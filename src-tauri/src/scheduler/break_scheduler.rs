use chrono::{DateTime, Datelike, Duration, Local, Utc};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{mpsc, watch};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use super::models::{
    BreakInfo, Command, PauseReason, SchedulerEvent, SchedulerEventInfo, SchedulerStatus,
};
use crate::config::{AppConfig, SharedConfig};

/// The state of the break scheduler
#[derive(Debug, PartialEq, Clone)]
enum BreakSchedulerState {
    Running,
    Paused(PauseReason),
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

    current_break_info: Option<BreakInfo>,

    // Cancellation token for interrupting waits
    current_wait_cancel: Option<CancellationToken>,
}

impl BreakScheduler {
    pub fn new(app_handle: AppHandle, shutdown_rx: watch::Receiver<()>) -> Self {
        Self {
            app_handle,
            shutdown_rx,
            state: BreakSchedulerState::Running,
            mini_break_counter: 0,
            last_break_time: None,
            postponed_until: None,
            current_break_info: None,
            current_wait_cancel: None,
        }
    }

    pub async fn run(&mut self, mut cmd_rx: mpsc::Receiver<Command>) {
        tracing::info!("BreakScheduler started");

        loop {
            // Check for incoming commands first
            if let Ok(cmd) = cmd_rx.try_recv() {
                self.handle_command(cmd).await;
                continue;
            }

            match &self.state {
                BreakSchedulerState::Running => {
                    let break_result = {
                        let config = self.app_handle.state::<SharedConfig>();
                        let config_guard = config.read().await;
                        self.calculate_next_break(&config_guard)
                    };

                    if let Some(break_info) = break_result {
                        self.emit_status(&break_info);
                        self.wait_and_execute_break(break_info, &mut cmd_rx).await;
                    } else {
                        tracing::debug!("No active schedule, waiting for commands");
                        self.emit_paused_status(false);

                        // Wait for command
                        if let Some(cmd) = cmd_rx.recv().await {
                            self.handle_command(cmd).await;
                        } else {
                            break; // Channel closed
                        }
                    }
                }
                BreakSchedulerState::Paused(_) => {
                    tracing::debug!("BreakScheduler paused, waiting for resume");
                    self.emit_paused_status(true);

                    // Wait for command
                    if let Some(cmd) = cmd_rx.recv().await {
                        self.handle_command(cmd).await;
                    } else {
                        break; // Channel closed
                    }
                }
            }
        }

        tracing::info!("BreakScheduler shutting down");
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

    /// Wait for and execute a break, handling notifications
    async fn wait_and_execute_break(
        &mut self,
        break_info: BreakInfo,
        cmd_rx: &mut mpsc::Receiver<Command>,
    ) {
        self.current_break_info = Some(break_info.clone());
        let cancel_token = CancellationToken::new();
        self.current_wait_cancel = Some(cancel_token.clone());

        let now = Utc::now();
        let duration_to_break = break_info.break_time - now;

        if duration_to_break <= Duration::zero() {
            // Break time already passed, execute immediately
            tracing::warn!("Break time already passed, executing immediately");
            self.execute_break(break_info.event);
            self.current_wait_cancel = None;
            return;
        }

        tracing::info!(
            "Next break: {} in {} seconds",
            break_info.event,
            duration_to_break.num_seconds()
        );

        // Phase 1: Wait until notification time (if applicable)
        if let Some(notif_time) = break_info.notification_time
            && let duration_to_notif = notif_time - now
            && duration_to_notif > Duration::zero()
        {
            tracing::debug!(
                "Waiting {} seconds until notification",
                duration_to_notif.num_seconds()
            );

            tokio::select! {
                biased;
                _ = self.shutdown_rx.changed() => {
                    tracing::info!("BreakScheduler received shutdown (during break wait)");
                    self.current_wait_cancel = None;
                    return;
                }
                () = cancel_token.cancelled() => {
                    tracing::info!("Break wait cancelled during notification phase");
                    self.current_wait_cancel = None;
                    return;
                }
                () = sleep(duration_to_notif.to_std().unwrap_or(std::time::Duration::ZERO)) => {
                    self.send_notification(&break_info.event).await;
                }
                Some(cmd) = cmd_rx.recv() => {
                    self.handle_command(cmd).await;
                    self.current_wait_cancel = None;
                    return;
                }
            }
        }

        // Phase 2: Wait until break time
        let now = Utc::now();
        let remaining_duration = break_info.break_time - now;

        if remaining_duration > Duration::zero() {
            tracing::debug!(
                "Waiting {} seconds until break",
                remaining_duration.num_seconds()
            );

            tokio::select! {
                biased;
                _ = self.shutdown_rx.changed() => {
                    tracing::info!("BreakScheduler received shutdown (during break wait)");
                    self.current_wait_cancel = None;
                    return;
                }
                () = cancel_token.cancelled() => {
                    tracing::info!("Break wait cancelled during break phase");
                    self.current_wait_cancel = None;
                    return;
                }
                () = sleep(remaining_duration.to_std().unwrap_or(std::time::Duration::ZERO)) => {
                    self.execute_break(break_info.event);
                }
                Some(cmd) = cmd_rx.recv() => {
                    self.handle_command(cmd).await;
                    self.current_wait_cancel = None;
                    return;
                }
            }
        } else {
            self.execute_break(break_info.event);
        }

        self.current_break_info = None;
        self.current_wait_cancel = None;
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

        if let Err(e) = crate::platform::notifications::send_break_notification(
            &self.app_handle,
            break_type,
            notification_before_s,
        ) {
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

    /// Execute a break and update state
    fn execute_break(&mut self, event: SchedulerEvent) {
        tracing::info!("Executing break: {event}");

        // Emit event to frontend
        if let Err(e) = self.app_handle.emit("scheduler-event", &event) {
            tracing::error!("Failed to emit scheduler-event: {e}");
        }

        self.update_state_after_break(event);
    }

    /// Handle incoming commands
    async fn handle_command(&mut self, cmd: Command) {
        tracing::debug!("BreakScheduler handling command: {cmd:?}");

        match cmd {
            Command::Pause(reason) => {
                tracing::info!("Pausing BreakScheduler: {reason}");
                self.state = BreakSchedulerState::Paused(reason);
                self.cancel_current_wait();

                // Reset timers for certain pause reasons
                match reason {
                    PauseReason::UserIdle | PauseReason::Dnd | PauseReason::AppExclusion => {
                        self.reset_timers();
                    }
                    PauseReason::Manual => {}
                }

                self.emit_paused_status(true);
            }
            Command::Resume(_reason) => {
                tracing::info!("Resuming BreakScheduler");
                self.state = BreakSchedulerState::Running;
                self.update_break_timers(false);
                // Status will be emitted in next loop iteration
            }
            Command::Postpone => {
                tracing::info!("Postponing current break");
                let postpone_duration_s = {
                    let config = self.app_handle.state::<SharedConfig>();
                    let config_guard = config.read().await;
                    let now_local = Utc::now().with_timezone(&Local);
                    get_active_schedule(&config_guard, now_local.time(), now_local.weekday())
                        .map_or(300, |s| s.mini_breaks.base.postponed_s)
                };

                self.postponed_until =
                    Some(Utc::now() + Duration::seconds(i64::from(postpone_duration_s)));
                self.cancel_current_wait();

                // Close break window
                if let Err(e) = self.app_handle.emit("break-finished", "") {
                    tracing::error!("Failed to emit break-finished event: {e}");
                }
            }
            Command::SkipBreak => {
                tracing::info!("Skipping current break");
                if let Some(break_info) = &self.current_break_info {
                    self.update_state_after_break(break_info.event);
                } else {
                    self.update_break_timers(true);
                }
                self.cancel_current_wait();

                // Close break window
                if let Err(e) = self.app_handle.emit("break-finished", "") {
                    tracing::error!("Failed to emit break-finished event: {e}");
                }
            }
            Command::TriggerBreak(event) => {
                tracing::info!("Manually triggering break: {event}");
                self.cancel_current_wait();
                self.execute_break(event);
            }
            Command::UpdateConfig(new_config) => {
                tracing::debug!("Updating config");
                {
                    let config = self.app_handle.state::<SharedConfig>();
                    let mut config_guard = config.write().await;
                    *config_guard = new_config;
                }

                self.cancel_current_wait();
            }
            Command::RequestStatus => {
                tracing::debug!("Status request received");
                let config = self.app_handle.state::<SharedConfig>();
                let config_guard = config.read().await;

                if let BreakSchedulerState::Paused(_) = self.state {
                    self.emit_paused_status(true);
                } else if let Some(break_info) = self.calculate_next_break(&config_guard) {
                    self.emit_status(&break_info);
                } else {
                    self.emit_paused_status(false);
                }
            }
        }
    }

    /// Cancel any ongoing break wait
    fn cancel_current_wait(&mut self) {
        if let Some(token) = &self.current_wait_cancel {
            token.cancel();
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
}

/// Helper function to get active schedule
fn get_active_schedule(
    config: &AppConfig,
    now_time: chrono::NaiveTime,
    now_day: chrono::Weekday,
) -> Option<&crate::core::schedule::ScheduleSettings> {
    config.schedules.iter().find(|s| {
        s.enabled && s.days_of_week.contains(&now_day) && s.time_range.contains(&now_time)
    })
}
