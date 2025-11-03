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
use crate::cmd::window::create_break_windows;
use crate::config::{AppConfig, SharedConfig};
use crate::core::schedule::ScheduleSettings;
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
    InBreak(SchedulerEvent),
}

impl Display for BreakSchedulerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state_str = match self {
            BreakSchedulerState::Paused(reason) => format!("Paused({reason})"),
            BreakSchedulerState::Idle => "Idle".to_string(),
            BreakSchedulerState::WaitingForNotification(_) => "WaitingForNotification".to_string(),
            BreakSchedulerState::WaitingForBreak(_) => "WaitingForBreak".to_string(),
            BreakSchedulerState::InBreak(event) => format!("InBreak({event})"),
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

    // Postpone state
    postponed_until: Option<DateTime<Utc>>,
}

impl BreakScheduler {
    pub fn new(app_handle: AppHandle, shutdown_rx: watch::Receiver<()>) -> Self {
        Self {
            app_handle,
            shutdown_rx,
            state: BreakSchedulerState::Paused(PauseReason::Manual),
            mini_break_counter: 0,
            last_break_time: None,
            postponed_until: None,
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
                self.execute_break(info.event).await;
            }
            _ => {
                tracing::warn!("Timer fired in unexpected state: {}", self.state);
            }
        }
    }

    /// Send a notification before a break
    async fn send_notification(&self, event: &SchedulerEvent) {
        let break_type = match event {
            SchedulerEvent::MiniBreak(_) => "Mini Break",
            SchedulerEvent::LongBreak(_) => "Long Break",
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

        if let Err(e) = send_break_notification(&self.app_handle, break_type, notification_before_s)
        {
            tracing::warn!("Failed to send break notification: {e}");
        }
    }

    /// Update state after a break has been executed
    fn update_state_after_break(&mut self, event: SchedulerEvent) {
        self.update_break_timers(true);

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
                | BreakSchedulerState::WaitingForNotification(info) => match info.event {
                    SchedulerEvent::MiniBreak(_) => s.mini_breaks.base.postponed_s,
                    SchedulerEvent::LongBreak(_) => s.long_breaks.base.postponed_s,
                    SchedulerEvent::Attention(_) => unreachable!(),
                },
                BreakSchedulerState::InBreak(event) => match event {
                    SchedulerEvent::MiniBreak(_) => s.mini_breaks.base.postponed_s,
                    SchedulerEvent::LongBreak(_) => s.long_breaks.base.postponed_s,
                    SchedulerEvent::Attention(_) => unreachable!(),
                },
                _ => s.mini_breaks.base.postponed_s, // fallback to mini break postpone
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
                        self.reset_timers();
                    }
                    PauseReason::Manual => {}
                }
                self.close_break_windows();
                self.emit_paused_status(true);
            }
            Command::Resume(_reason) => {
                tracing::info!("Resuming BreakScheduler");
                if let BreakSchedulerState::Paused(_) = self.state {
                    self.update_break_timers(false);
                    self.transition_to_calculating().await;
                }
            }
            Command::Postpone => {
                tracing::info!("Postponing current break");
                let postpone_s = self.get_postpone_duration_s().await;
                self.postponed_until = Some(Utc::now() + Duration::seconds(i64::from(postpone_s)));
                self.close_break_windows();
                self.transition_to_calculating().await;
            }
            Command::SkipBreak => {
                tracing::info!("Skipping current break");
                match &self.state {
                    BreakSchedulerState::WaitingForNotification(info)
                    | BreakSchedulerState::WaitingForBreak(info) => {
                        self.update_state_after_break(info.event);
                    }
                    BreakSchedulerState::InBreak(event) => {
                        self.update_state_after_break(*event);
                    }
                    _ => {
                        self.update_break_timers(true);
                    }
                }
                self.close_break_windows();
                self.transition_to_calculating().await;
            }
            Command::BreakFinished(event) => {
                if let BreakSchedulerState::InBreak(current_event) = self.state {
                    if event == current_event {
                        tracing::info!("Break finished normally: {event}");
                        self.update_state_after_break(event);
                        self.transition_to_calculating().await;
                    } else {
                        tracing::warn!(
                            "Received BreakFinished for different event: expected {current_event}, got {event}"
                        );
                    }
                } else {
                    tracing::warn!("Unexpected BreakFinished command in state: {}", self.state);
                }
            }
            Command::TriggerEvent(event) => {
                tracing::info!("Manually triggering break: {event}");
                self.execute_break(event).await;
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
            Command::RequestStatus => {
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
                Box::pin(self.execute_break(break_info.event)).await;
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
        let base_time = self.postponed_until.or(self.last_break_time).unwrap_or(now);
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
        })
    }

    /// Execute a break: create window and play audio, then wait for completion
    async fn execute_break(&mut self, event: SchedulerEvent) {
        tracing::info!("Executing break: {event}");
        self.state = BreakSchedulerState::InBreak(event);
        if let Err(e) = create_break_windows(&self.app_handle, event).await {
            tracing::error!("Failed to create break windows: {e}");
            self.update_state_after_break(event);
            Box::pin(self.transition_to_calculating()).await;
        }
    }

    /// Reset break timers
    fn reset_timers(&mut self) {
        tracing::debug!("Resetting break timers");
        self.last_break_time = None;
        self.postponed_until = None;
    }

    /// Update break timers
    fn update_break_timers(&mut self, clear_postpone: bool) {
        self.last_break_time = Some(Utc::now());
        if clear_postpone {
            self.postponed_until = None;
        }
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

        if let Err(e) = self.app_handle.emit("scheduler-status", &status) {
            tracing::warn!("Failed to emit scheduler status: {e}");
        }
    }

    /// Emit paused status to frontend
    fn emit_paused_status(&self, paused: bool) {
        let status = SchedulerStatus {
            paused,
            next_event: None,
        };

        if let Err(e) = self.app_handle.emit("scheduler-status", &status) {
            tracing::warn!("Failed to emit scheduler status: {e}");
        }
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
