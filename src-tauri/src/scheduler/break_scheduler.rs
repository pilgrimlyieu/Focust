use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;

use chrono::{DateTime, Datelike, Duration, Local, Utc};
use futures::future::pending;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{mpsc, watch};
use tokio::time::sleep;

use super::models::{
    BreakInfo, Command, PauseReason, SchedulerEvent, SchedulerEventInfo, SchedulerStatus,
};
use crate::config::{AppConfig, SharedConfig};
use crate::core::schedule::ScheduleSettings;
use crate::platform::create_prompt_windows;
use crate::platform::send_break_notification;

/// The state of the break scheduler
#[derive(Debug, Clone)]
enum BreakSchedulerState {
    /// Paused
    Paused(PauseReason),
    /// Running, but no active schedule (e.g., non-working hours)
    Idle,
    /// Waiting for break notification to be sent
    WaitingForNotification(BreakInfo),
    /// Waiting for break to start (notification has been sent or not needed)
    WaitingForBreak(BreakInfo),
    /// In break (break window is open)
    InBreak(BreakInfo),
}

impl Display for BreakSchedulerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state_str = match self {
            BreakSchedulerState::Paused(reason) => format!("Paused({reason})"),
            BreakSchedulerState::Idle => "Idle".to_string(),
            BreakSchedulerState::WaitingForNotification(_) => "WaitingForNotification".to_string(),
            BreakSchedulerState::WaitingForBreak(_) => "WaitingForBreak".to_string(),
            BreakSchedulerState::InBreak(info) => format!("InBreak({})", info.event),
        };
        write!(f, "{state_str}")
    }
}

/// Main break scheduler responsible for managing mini and long breaks
pub struct BreakScheduler {
    app_handle: AppHandle,
    shutdown_rx: watch::Receiver<()>,
    state: BreakSchedulerState,

    // Break cycle state
    mini_break_counter: u8,
    last_break_time: Option<DateTime<Utc>>,
}

impl BreakScheduler {
    pub fn new(app_handle: AppHandle, shutdown_rx: watch::Receiver<()>) -> Self {
        Self {
            app_handle,
            shutdown_rx,
            state: BreakSchedulerState::Paused(PauseReason::Manual),
            mini_break_counter: 0,
            last_break_time: None,
        }
    }

    /// Main run loop
    pub async fn run(&mut self, mut cmd_rx: mpsc::Receiver<Command>) {
        tracing::info!("BreakScheduler started");

        self.transition_to_calculating().await;
        loop {
            let timer_duration = self.get_duration_for_current_state();
            let mut sleep_fut: Pin<Box<dyn Future<Output = ()> + Send>> =
                if let Some(duration) = timer_duration {
                    let std_duration = duration.to_std().unwrap_or(std::time::Duration::ZERO);
                    Box::pin(sleep(std_duration))
                } else {
                    Box::pin(pending()) // This future never completes
                };

            tokio::select! {
                biased;
                _ = self.shutdown_rx.changed() => {
                    tracing::info!("BreakScheduler received shutdown");
                    break;
                }
                Some(cmd) = cmd_rx.recv() => {
                    self.handle_command(cmd).await;
                }
                () = &mut sleep_fut => {
                    if timer_duration.is_some() {
                        self.on_timer_fired().await;
                    }
                }
                else => {
                    tracing::info!("Command channel closed, shutting down");
                    break;
                }
            }
        }
        tracing::info!("BreakScheduler shutting down");
    }

    /// Get the duration of next timer based on current state
    fn get_duration_for_current_state(&self) -> Option<Duration> {
        let now = Utc::now();
        match &self.state {
            BreakSchedulerState::WaitingForNotification(info) => {
                info.notification_time.map(|notif_time| notif_time - now)
            }
            BreakSchedulerState::WaitingForBreak(info) => Some(info.break_time - now),
            BreakSchedulerState::Paused(_)
            | BreakSchedulerState::Idle
            | BreakSchedulerState::InBreak(_) => None,
        }
    }

    /// Handle timer fired event based on current state
    async fn on_timer_fired(&mut self) {
        match self.state.clone() {
            BreakSchedulerState::WaitingForNotification(info) => {
                tracing::debug!("Timer fired: sending notification");
                self.send_notification(&info.event).await;
                self.state = BreakSchedulerState::WaitingForBreak(info);
            }
            BreakSchedulerState::WaitingForBreak(info) => {
                tracing::debug!("Timer fired: executing break");
                self.execute_break(info).await;
            }
            _ => {
                tracing::warn!("Timer fired in unexpected state: {}", self.state);
            }
        }
    }

    /// Send a notification before a break
    async fn send_notification(&self, event: &SchedulerEvent) {
        let break_type = match event {
            SchedulerEvent::MiniBreak(_) => "MiniBreak",
            SchedulerEvent::LongBreak(_) => "LongBreak",
            SchedulerEvent::Attention(_) => return,
        };

        let notification_before_s = {
            let config = self.app_handle.state::<SharedConfig>();
            let config_guard = config.read().await;
            let now_local = Utc::now().with_timezone(&Local);
            let active_schedule =
                get_active_schedule(&config_guard, now_local.time(), now_local.weekday());
            active_schedule.map_or(0, |s| s.notification_before_s)
        };

        send_break_notification(&self.app_handle, break_type, notification_before_s)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to send break notification: {e}");
            });
    }

    /// Update state after a break has been executed
    fn update_state_after_break(&mut self, event: SchedulerEvent) {
        self.update_last_break_time();

        match event {
            SchedulerEvent::MiniBreak(_) => {
                self.mini_break_counter += 1;
            }
            SchedulerEvent::LongBreak(_) => {
                self.mini_break_counter = 0;
            }
            SchedulerEvent::Attention(_) => {}
        }
    }

    /// Execute a break: create window and play audio, then wait for completion
    /// Close all break windows
    fn close_break_windows(&self) {
        let windows = self.app_handle.webview_windows();
        for (label, window) in windows {
            if label.starts_with("break-") {
                tracing::debug!("Closing break window: {label}");
                let _ = window.close();
            }
        }
    }

    /// Get postpone duration based on current break type
    async fn get_postpone_duration_s(&self) -> u32 {
        let config = self.app_handle.state::<SharedConfig>();
        let config_guard = config.read().await;
        let now_local = Utc::now().with_timezone(&Local);
        let active_schedule =
            get_active_schedule(&config_guard, now_local.time(), now_local.weekday());
        active_schedule.map_or(300, |s| {
            match &self.state {
                BreakSchedulerState::WaitingForBreak(info)
                | BreakSchedulerState::WaitingForNotification(info)
                | BreakSchedulerState::InBreak(info) => match info.event {
                    SchedulerEvent::MiniBreak(_) => s.mini_breaks.base.postponed_s,
                    SchedulerEvent::LongBreak(_) => s.long_breaks.base.postponed_s,
                    SchedulerEvent::Attention(_) => unreachable!(),
                },
                _ => s.mini_breaks.base.postponed_s, // fallback to mini break postpone
            }
        })
    }

    /// Get maximum postpone count based on current break type
    async fn get_max_postpone_count(&self) -> u8 {
        let config = self.app_handle.state::<SharedConfig>();
        let config_guard = config.read().await;
        let now_local = Utc::now().with_timezone(&Local);
        let active_schedule =
            get_active_schedule(&config_guard, now_local.time(), now_local.weekday());
        active_schedule.map_or(2, |s| {
            match &self.state {
                BreakSchedulerState::WaitingForBreak(info)
                | BreakSchedulerState::WaitingForNotification(info)
                | BreakSchedulerState::InBreak(info) => match info.event {
                    SchedulerEvent::MiniBreak(_) => s.mini_breaks.base.max_postpone_count,
                    SchedulerEvent::LongBreak(_) => s.long_breaks.base.max_postpone_count,
                    SchedulerEvent::Attention(_) => 0, // unreachable!(),
                },
                _ => s.mini_breaks.base.max_postpone_count, // fallback
            }
        })
    }

    /// Emit current status to frontend
    fn emit_current_status(&self) {
        match &self.state {
            BreakSchedulerState::Paused(_) => {
                self.emit_paused_status(true);
            }
            BreakSchedulerState::Idle | BreakSchedulerState::InBreak(_) => {
                self.emit_idle_status();
            }
            BreakSchedulerState::WaitingForNotification(info)
            | BreakSchedulerState::WaitingForBreak(info) => {
                self.emit_status(info);
            }
        }
    }

    /// Handle incoming commands
    /// Returns true if the command requires interrupting the current wait
    #[allow(clippy::too_many_lines)]
    async fn handle_command(&mut self, cmd: Command) {
        tracing::debug!("BreakScheduler handling command: {cmd}");
        match cmd {
            Command::Pause(reason) => {
                tracing::info!("Pausing BreakScheduler: {reason}");
                self.state = BreakSchedulerState::Paused(reason);

                // Reset timers for certain pause reasons
                match reason {
                    PauseReason::UserIdle | PauseReason::Dnd | PauseReason::AppExclusion => {
                        self.reset_last_break_time();
                    }
                    PauseReason::Manual => {}
                }
                self.close_break_windows();
                // Note: SchedulerManager already emitted paused status, no need to emit again
            }
            Command::Resume(_reason) => {
                tracing::info!("Resuming BreakScheduler");
                if let BreakSchedulerState::Paused(_) = self.state {
                    self.update_last_break_time();
                    self.transition_to_calculating().await;
                }
            }
            Command::PostponeBreak => {
                // Check postpone limit first
                let max_count = self.get_max_postpone_count().await;

                let current_count = match &self.state {
                    BreakSchedulerState::WaitingForNotification(info)
                    | BreakSchedulerState::WaitingForBreak(info)
                    | BreakSchedulerState::InBreak(info) => info.postpone_count,
                    _ => {
                        tracing::warn!("Cannot postpone in current state: {}", self.state);
                        return;
                    }
                };

                // Check if limit reached
                if current_count >= max_count {
                    tracing::warn!(
                        "Max postpone count ({max_count}) reached, cannot postpone further",
                    );
                    // Emit event to notify frontend
                    let _ = self.app_handle.emit("postpone-limit-reached", ());
                    return;
                }

                let postpone_s = self.get_postpone_duration_s().await;
                let postpone_duration = Duration::seconds(i64::from(postpone_s));

                match &self.state {
                    BreakSchedulerState::WaitingForNotification(info)
                    | BreakSchedulerState::WaitingForBreak(info) => {
                        // Scenario 1: Break not yet triggered - delay the scheduled break time
                        tracing::info!(
                            "Postponing upcoming break (postpone_count: {})",
                            info.postpone_count + 1
                        );

                        let mut new_info = info.clone();
                        new_info.postpone_count += 1;
                        new_info.break_time += postpone_duration;
                        // Remove notification time since we're postponing
                        new_info.notification_time = None;

                        self.state = BreakSchedulerState::WaitingForBreak(new_info.clone());
                        self.emit_status(&new_info);
                    }

                    BreakSchedulerState::InBreak(info) => {
                        // Scenario 2: Break already triggered - close window and reschedule
                        tracing::info!(
                            "Postponing active break, will retry in {postpone_s}s (postpone_count: {})",
                            info.postpone_count + 1
                        );

                        let mut new_info = info.clone();
                        new_info.postpone_count += 1;
                        new_info.break_time = Utc::now() + postpone_duration;
                        new_info.notification_time = None;

                        self.close_break_windows();
                        self.state = BreakSchedulerState::WaitingForBreak(new_info.clone());
                        self.emit_status(&new_info);
                        // Do NOT update last break time. This break hasn't been completed, just postponed.
                    }

                    _ => unreachable!("Cannot postpone in {} state.", self.state),
                }
            }
            Command::SkipBreak => {
                tracing::info!("Skipping current break");
                match &self.state {
                    BreakSchedulerState::WaitingForNotification(info)
                    | BreakSchedulerState::WaitingForBreak(info)
                    | BreakSchedulerState::InBreak(info) => {
                        self.update_state_after_break(info.event);
                    }
                    _ => {
                        self.update_last_break_time();
                    }
                }
                self.close_break_windows();
                self.transition_to_calculating().await;
            }
            Command::PromptFinished(event) => {
                if let BreakSchedulerState::InBreak(info) = &self.state {
                    if event == info.event {
                        tracing::info!("Break finished normally: {event}");
                        self.update_state_after_break(event);
                        self.transition_to_calculating().await;
                    } else {
                        tracing::warn!(
                            "Received PromptFinished for different event: expected {}, got {event}",
                            info.event
                        );
                    }
                } else {
                    tracing::warn!("Unexpected PromptFinished command in state: {}", self.state);
                }
            }
            Command::TriggerEvent(event) => {
                tracing::info!("Manually triggering break: {event}");
                // Create a new BreakInfo for manual trigger
                let test_info = BreakInfo {
                    break_time: Utc::now(),
                    notification_time: None,
                    event,
                    postpone_count: 0,
                };
                self.execute_break(test_info).await;
            }
            Command::UpdateConfig(new_config) => {
                tracing::debug!("Updating config");
                {
                    let config = self.app_handle.state::<SharedConfig>();
                    let mut config_guard = config.write().await;
                    *config_guard = new_config;
                }
                self.transition_to_calculating().await;
            }
            Command::RequestBreakStatus => {
                tracing::debug!("Status request received");
                self.emit_current_status();
            }
        }
    }

    /// Transition to calculating next break
    async fn transition_to_calculating(&mut self) {
        let break_info = {
            let config = self.app_handle.state::<SharedConfig>();
            let config_guard = config.read().await;
            self.calculate_next_break(&config_guard)
        };

        if let Some(break_info) = break_info {
            let now = Utc::now();

            if break_info.break_time <= now {
                tracing::warn!("Break time already passed, executing immediately");
                Box::pin(self.execute_break(break_info)).await;
            } else if let Some(notif_time) = break_info.notification_time {
                if notif_time <= now {
                    tracing::debug!("Notification time passed, sending immediately");
                    self.send_notification(&break_info.event).await;
                    self.state = BreakSchedulerState::WaitingForBreak(break_info.clone());
                    self.emit_status(&break_info);
                } else {
                    tracing::info!("Transitioning to WaitingForNotification");
                    self.state = BreakSchedulerState::WaitingForNotification(break_info.clone());
                    self.emit_status(&break_info);
                }
            } else {
                tracing::info!("Transitioning to WaitingForBreak");
                self.state = BreakSchedulerState::WaitingForBreak(break_info.clone());
                self.emit_status(&break_info);
            }
        } else {
            tracing::info!("Transitioning to Idle (no active schedule)");
            self.state = BreakSchedulerState::Idle;
            self.emit_idle_status();
        }
    }

    /// Calculate the next break based on current state and configuration
    fn calculate_next_break(&self, config: &AppConfig) -> Option<BreakInfo> {
        let now = Utc::now();
        let now_local = now.with_timezone(&Local);

        // Check if we're in an active schedule
        let active_schedule = get_active_schedule(config, now_local.time(), now_local.weekday())?;

        // Determine if it's time for a long break
        let is_long_break_due = active_schedule.long_breaks.base.enabled
            && self.mini_break_counter >= active_schedule.long_breaks.after_mini_breaks;

        let (event, break_settings) = if is_long_break_due {
            (
                SchedulerEvent::LongBreak(active_schedule.long_breaks.base.id),
                &active_schedule.long_breaks.base,
            )
        } else {
            (
                SchedulerEvent::MiniBreak(active_schedule.mini_breaks.base.id),
                &active_schedule.mini_breaks.base,
            )
        };

        if !break_settings.enabled {
            return None;
        }

        // Calculate break time
        let interval = Duration::seconds(i64::from(active_schedule.mini_breaks.interval_s));
        let base_time = self.last_break_time.unwrap_or(now);
        let break_time = base_time + interval;

        // Calculate notification time if enabled
        let notification_time = active_schedule
            .has_notification()
            .then(|| {
                let notif_time = break_time
                    - Duration::seconds(i64::from(active_schedule.notification_before_s));
                (notif_time > now).then_some(notif_time)
            })
            .flatten();

        Some(BreakInfo {
            break_time,
            notification_time,
            event,
            postpone_count: 0,
        })
    }

    /// Execute a break: create window and play audio, then wait for completion
    async fn execute_break(&mut self, info: BreakInfo) {
        tracing::info!("Executing break: {}", info.event);
        let event = info.event;
        let postpone_count = info.postpone_count;
        self.state = BreakSchedulerState::InBreak(info);
        if let Err(e) = create_prompt_windows(&self.app_handle, event, postpone_count).await {
            tracing::error!("Failed to create break windows: {e}");
            self.update_state_after_break(event);
            Box::pin(self.transition_to_calculating()).await;
        }
    }

    /// Reset break timers
    fn reset_last_break_time(&mut self) {
        self.last_break_time = None;
    }

    /// Update break timers after a break completes
    fn update_last_break_time(&mut self) {
        self.last_break_time = Some(Utc::now());
    }

    /// Emit current status to frontend
    fn emit_status(&self, break_info: &BreakInfo) {
        let duration_to_wait = break_info.break_time - Utc::now();
        let status = SchedulerStatus {
            paused: false,
            next_event: Some(SchedulerEventInfo {
                kind: break_info.event,
                time: break_info.break_time.to_rfc3339(),
                seconds_until: duration_to_wait.num_seconds() as i32,
            }),
        };

        self.app_handle
            .emit("scheduler-status", &status)
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to emit scheduler status: {e}");
            });
    }

    /// Emit paused status to frontend
    fn emit_paused_status(&self, paused: bool) {
        let status = SchedulerStatus {
            paused,
            next_event: None,
        };

        self.app_handle
            .emit("scheduler-status", &status)
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to emit scheduler status: {e}");
            });
    }

    /// Emit idle status to frontend
    fn emit_idle_status(&self) {
        self.emit_paused_status(false);
    }
}

/// Helper function to get active schedule
fn get_active_schedule(
    config: &AppConfig,
    now_time: chrono::NaiveTime,
    now_day: chrono::Weekday,
) -> Option<&ScheduleSettings> {
    config.schedules.iter().find(|s| {
        s.enabled && s.days_of_week.contains(&now_day) && s.time_range.contains(&now_time)
    })
}
