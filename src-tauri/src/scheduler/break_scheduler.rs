use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;

use chrono::{DateTime, Datelike, Duration, Local, Utc};
use futures::future::pending;
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::{mpsc, watch};
use tokio::time::sleep;

use super::event_emitter::EventEmitter;
use super::models::{
    BreakInfo, Command, PauseReason, SchedulerEvent, SchedulerEventInfo, SchedulerStatus,
};
use super::shared_state::SharedState;
use crate::config::{AppConfig, SharedConfig};
#[cfg(not(test))]
use crate::platform::create_prompt_windows;
use crate::platform::send_break_notification;
use crate::scheduler::event::get_active_schedule;

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
    /// In break (prompt window is open)
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
pub struct BreakScheduler<E, R = tauri::Wry>
where
    E: EventEmitter,
    R: Runtime,
{
    app_handle: AppHandle<R>,
    event_emitter: E,
    shutdown_rx: watch::Receiver<()>,
    state: BreakSchedulerState,

    // Break cycle state
    mini_break_counter: u8,
    last_break_time: Option<DateTime<Utc>>,

    // Shared state for session management
    shared_state: SharedState,
}

impl<E, R> BreakScheduler<E, R>
where
    E: EventEmitter,
    R: Runtime,
{
    pub fn new(
        app_handle: AppHandle<R>,
        event_emitter: E,
        shutdown_rx: watch::Receiver<()>,
        shared_state: SharedState,
    ) -> Self {
        Self {
            app_handle,
            event_emitter,
            shutdown_rx,
            state: BreakSchedulerState::Idle,
            mini_break_counter: 0,
            last_break_time: None,
            shared_state,
        }
    }

    /// Get current state as string (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn get_state(&self) -> String {
        format!("{}", self.state)
    }

    /// Get mini break counter (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn get_mini_break_counter(&self) -> u8 {
        self.mini_break_counter
    }

    /// Get last break time (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn get_last_break_time(&self) -> Option<DateTime<Utc>> {
        self.last_break_time
    }

    /// Main run loop
    pub async fn run(&mut self, mut cmd_rx: mpsc::Receiver<Command>) {
        tracing::info!("BreakScheduler started");

        // Only transition to calculating if not paused
        // If paused, wait for Resume command to start scheduling
        if !matches!(self.state, BreakSchedulerState::Paused(_)) {
            self.transition_to_calculating().await;
        }

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

        // CRITICAL: Always clean up session state when closing break windows
        self.shared_state.write().end_break_session();
        tracing::debug!("Break session ended (windows closed), monitors will resume monitoring");
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
    async fn handle_command(&mut self, cmd: Command) {
        tracing::debug!("BreakScheduler handling command: {cmd}");
        match cmd {
            Command::Pause(reason) => {
                self.handle_pause_command(reason);
            }
            Command::Resume(_reason) => {
                self.handle_resume_command().await;
            }
            Command::PostponeBreak => {
                self.handle_postpone_break_command().await;
            }
            Command::SkipBreak => {
                self.handle_skip_break_command().await;
            }
            Command::PromptFinished(event) => {
                self.handle_prompt_finished_command(event).await;
            }
            Command::TriggerEvent(event) => {
                self.handle_trigger_event_command(event).await;
            }
            Command::UpdateConfig(new_config) => {
                self.handle_update_config_command(new_config).await;
            }
            Command::RequestBreakStatus => {
                self.handle_request_break_status_command();
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
        calculate_next_break_pure(config, now, self.mini_break_counter, self.last_break_time)
    }

    /// Execute a break: create window and play audio, then wait for completion
    #[allow(clippy::unused_async)]
    async fn execute_break(&mut self, info: BreakInfo) {
        tracing::info!("Executing break: {}", info.event);
        let event = info.event;
        #[cfg(not(test))]
        let postpone_count = info.postpone_count;
        self.state = BreakSchedulerState::InBreak(info);

        // CRITICAL: Mark break session start BEFORE creating windows
        // This prevents DND monitor from reacting to system DND triggered by the fullscreen window
        // Windows fullscreen can trigger system Focus Assist, causing unwanted scheduler pause
        self.shared_state.write().start_break_session();
        tracing::info!("Break session started, DND monitor will ignore DND changes during break");

        // Emit event to notify tests/frontend that break is starting
        if let Err(e) = self.event_emitter.emit("scheduler-event", event) {
            tracing::warn!("Failed to emit scheduler-event: {e}");
        }

        // In tests with MockRuntime, skip window creation as it's not supported
        // The test can still verify that we entered InBreak state via events
        #[cfg(not(test))]
        {
            if let Err(e) = create_prompt_windows(&self.app_handle, event, postpone_count).await {
                tracing::error!("Failed to create break windows: {e}");

                // Clean up session state on error
                self.shared_state.write().end_break_session();
                tracing::info!("Break session ended (error cleanup)");

                self.update_state_after_break(event);
                Box::pin(self.transition_to_calculating()).await;
            }
        }

        #[cfg(test)]
        {
            // In tests, just log that we would create windows
            tracing::debug!("Test mode: skipping window creation for event: {event}");
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
            mini_break_counter: self.mini_break_counter,
        };

        self.event_emitter
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
            mini_break_counter: self.mini_break_counter,
        };

        self.event_emitter
            .emit("scheduler-status", &status)
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to emit scheduler status: {e}");
            });
    }

    /// Emit idle status to frontend
    fn emit_idle_status(&self) {
        self.emit_paused_status(false);
    }

    /// Handle Pause command
    fn handle_pause_command(&mut self, reason: PauseReason) {
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
        self.emit_paused_status(true);
    }

    /// Handle Resume command
    async fn handle_resume_command(&mut self) {
        tracing::info!("Resuming BreakScheduler");
        if let BreakSchedulerState::Paused(_) = self.state {
            self.update_last_break_time();
            Box::pin(self.transition_to_calculating()).await;
        }
    }

    /// Handle `PostponeBreak` command
    async fn handle_postpone_break_command(&mut self) {
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
            tracing::warn!("Max postpone count ({max_count}) reached, cannot postpone further");
            // Emit event to notify frontend
            let _ = self.event_emitter.emit("postpone-limit-reached", ());
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

    /// Handle `SkipBreak` command
    async fn handle_skip_break_command(&mut self) {
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

    /// Handle `PromptFinished` command
    async fn handle_prompt_finished_command(&mut self, event: SchedulerEvent) {
        tracing::debug!("Handling PromptFinished command for event: {event}");

        if let BreakSchedulerState::InBreak(info) = &self.state {
            if event == info.event {
                tracing::info!("Break finished normally: {event}");

                // Clean up session state
                self.shared_state.write().end_break_session();
                tracing::info!("Break session ended, DND monitor will resume monitoring");

                self.update_state_after_break(event);
                tracing::debug!("Break state updated after break: {event}");

                self.transition_to_calculating().await;
                tracing::debug!("Transitioned to calculating next break");
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

    /// Handle `TriggerEvent` command
    async fn handle_trigger_event_command(&mut self, event: SchedulerEvent) {
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

    /// Handle `UpdateConfig` command
    async fn handle_update_config_command(&mut self, new_config: AppConfig) {
        tracing::debug!("Updating config");
        {
            let config = self.app_handle.state::<SharedConfig>();
            let mut config_guard = config.write().await;
            *config_guard = new_config;
        }
        self.transition_to_calculating().await;
    }

    /// Handle `RequestBreakStatus` command
    fn handle_request_break_status_command(&mut self) {
        tracing::debug!("Status request received");
        self.emit_current_status();
    }
}

/// Pure function version of `calculate_next_break` for testing
///
/// This function has no side effects and can be tested independently.
/// All inputs are explicit parameters.
pub(crate) fn calculate_next_break_pure(
    config: &AppConfig,
    now: DateTime<Utc>,
    mini_break_counter: u8,
    last_break_time: Option<DateTime<Utc>>,
) -> Option<BreakInfo> {
    let now_local = now.with_timezone(&Local);

    // Check if we're in an active schedule
    let active_schedule = get_active_schedule(config, now_local.time(), now_local.weekday())?;

    // Determine if it's time for a long break
    let is_long_break_due = active_schedule.long_breaks.base.enabled
        && mini_break_counter >= active_schedule.long_breaks.after_mini_breaks;

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
    let base_time = last_break_time.unwrap_or(now);
    let break_time = base_time + interval;

    // Calculate notification time if enabled
    let notification_time = active_schedule
        .has_notification()
        .then(|| {
            let notif_time =
                break_time - Duration::seconds(i64::from(active_schedule.notification_before_s));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::core::schedule::{MiniBreakSettings, ScheduleSettings};
    use crate::scheduler::test_helpers::*;

    use chrono::Weekday;

    mod get_active_schedule_tests {
        use super::*;
        use crate::scheduler::event::get_active_schedule;

        #[test]
        fn returns_schedule_within_time_range() {
            let mut config = AppConfig::default();
            config.schedules[0].time_range = time_range(9, 0, 17, 0);
            config.schedules[0].days_of_week = workdays();
            config.schedules[0].enabled = true;

            let time = naive_time(10, 30, 0);
            let day = Weekday::Mon;

            let result = get_active_schedule(&config, time, day);
            assert!(result.is_some());
        }

        #[test]
        fn returns_none_outside_time_range() {
            let mut config = AppConfig::default();
            config.schedules[0].time_range = time_range(9, 0, 17, 0);
            config.schedules[0].days_of_week = workdays();
            config.schedules[0].enabled = true;

            let time = naive_time(20, 0, 0);
            let day = Weekday::Mon;

            let result = get_active_schedule(&config, time, day);
            assert!(result.is_none());
        }

        #[test]
        fn returns_none_on_non_working_day() {
            let mut config = AppConfig::default();
            config.schedules[0].time_range = time_range(9, 0, 17, 0);
            config.schedules[0].days_of_week = workdays();
            config.schedules[0].enabled = true;

            let time = naive_time(10, 0, 0);
            let day = Weekday::Sat;

            let result = get_active_schedule(&config, time, day);
            assert!(result.is_none());
        }

        #[test]
        fn ignores_disabled_schedules() {
            let mut config = AppConfig::default();
            config.schedules[0].time_range = time_range(9, 0, 17, 0);
            config.schedules[0].days_of_week = workdays();
            config.schedules[0].enabled = false;

            let time = naive_time(10, 0, 0);
            let day = Weekday::Mon;

            let result = get_active_schedule(&config, time, day);
            assert!(result.is_none());
        }
    }

    mod calculate_next_break_tests {
        use super::*;

        #[test]
        fn calculates_first_break_from_now() {
            let mut config = AppConfig::default();
            config.schedules[0].days_of_week = all_weekdays();
            config.schedules[0].mini_breaks.interval_s = 60;
            config.schedules[0].notification_before_s = 10;

            let now = Utc::now();
            let result = calculate_next_break_pure(&config, now, 0, None);

            let break_info = result.unwrap();
            let expected_time = now + duration_s(60);
            assert_time_near(break_info.break_time, expected_time, duration_s(1));
        }

        #[test]
        fn calculates_next_break_after_previous() {
            let mut config = AppConfig::default();
            config.schedules[0].days_of_week = all_weekdays();
            config.schedules[0].mini_breaks.interval_s = 60;

            let now = Utc::now();
            let last_break = now - duration_s(30);

            let result = calculate_next_break_pure(&config, now, 0, Some(last_break));

            let break_info = result.unwrap();
            let expected_time = last_break + duration_s(60);
            assert_time_near(break_info.break_time, expected_time, duration_s(1));
        }

        #[test]
        fn schedules_long_break_after_threshold() {
            let mut config = AppConfig::default();
            config.schedules[0].days_of_week = all_weekdays();
            config.schedules[0].long_breaks.after_mini_breaks = 4;

            let now = Utc::now();

            // Counter is 3 - should schedule mini break
            let result = calculate_next_break_pure(&config, now, 3, None);
            assert!(matches!(
                result.unwrap().event,
                SchedulerEvent::MiniBreak(_)
            ));

            // Counter is 4 - should schedule long break
            let result = calculate_next_break_pure(&config, now, 4, None);
            assert!(matches!(
                result.unwrap().event,
                SchedulerEvent::LongBreak(_)
            ));
        }

        #[test]
        fn returns_none_when_mini_breaks_disabled() {
            let mut config = AppConfig::default();
            config.schedules[0].days_of_week = all_weekdays();
            config.schedules[0].mini_breaks.base.enabled = false;

            let now = Utc::now();
            let result = calculate_next_break_pure(&config, now, 0, None);

            assert!(result.is_none());
        }

        #[test]
        fn returns_none_outside_active_schedule() {
            let mut config = AppConfig::default();
            config.schedules[0].time_range = time_range(9, 0, 17, 0);
            config.schedules[0].days_of_week = workdays();

            // Saturday at 10:00
            let now = test_datetime(2025, 9, 6, 10, 0, 0);

            let result = calculate_next_break_pure(&config, now, 0, None);
            assert!(result.is_none());
        }

        #[test]
        fn includes_notification_time_when_enabled() {
            let mut config = AppConfig::default();
            config.schedules[0].days_of_week = all_weekdays();
            config.schedules[0].mini_breaks.interval_s = 60;
            config.schedules[0].notification_before_s = 10;

            let now = Utc::now();
            let result = calculate_next_break_pure(&config, now, 0, None);

            let break_info = result.unwrap();

            let notif_time = break_info.notification_time.unwrap();
            let expected_notif = break_info.break_time - duration_s(10);
            assert_time_near(notif_time, expected_notif, duration_s(1));
        }

        #[test]
        fn omits_notification_when_disabled() {
            let mut config = AppConfig::default();
            config.schedules[0].days_of_week = all_weekdays();
            config.schedules[0].mini_breaks.interval_s = 60;
            config.schedules[0].notification_before_s = 0;

            let now = Utc::now();
            let result = calculate_next_break_pure(&config, now, 0, None);

            let break_info = result.unwrap();
            assert!(break_info.notification_time.is_none());
        }

        #[test]
        fn omits_notification_when_too_late() {
            let mut config = AppConfig::default();
            config.schedules[0].days_of_week = all_weekdays();
            config.schedules[0].mini_breaks.interval_s = 20;
            config.schedules[0].notification_before_s = 30;

            let now = Utc::now();
            let result = calculate_next_break_pure(&config, now, 0, None);

            // Notification time would be before now, so should be omitted
            let break_info = result.unwrap();
            assert!(break_info.notification_time.is_none());
        }

        // ===================================================================
        // Time Scheduling Tests
        // ===================================================================

        /// Cross-midnight time range support
        /// NOTE: Current implementation of [`TimeRange::contains()`] does NOT support cross-midnight ranges.
        /// This test is ignored until the feature is implemented.
        /// Expected behavior: schedules spanning across midnight (e.g., 22:00-02:00) should work correctly
        #[test]
        #[ignore = "TimeRange::contains() does not support cross-midnight ranges yet"]
        fn handles_cross_midnight_time_range() {
            let config = TestConfigBuilder::new()
                .time_range(time_range(22, 0, 2, 0)) // 22:00-02:00 (night shift)
                .mini_break_interval_s(60)
                .build();

            // Test during active time (23:00)
            let now_23h = test_datetime(2025, 9, 3, 23, 0, 0);
            let result = calculate_next_break_pure(&config, now_23h, 0, None);
            assert!(
                result.is_some(),
                "Should schedule break at 23:00 (within range)"
            );

            // Test after midnight but still in range (01:00)
            let now_01h = test_datetime(2025, 9, 4, 1, 0, 0);
            let result = calculate_next_break_pure(&config, now_01h, 0, None);
            assert!(
                result.is_some(),
                "Should schedule break at 01:00 (within range)"
            );

            // Test outside range (10:00)
            let now_10h = test_datetime(2025, 9, 3, 10, 0, 0);
            let result = calculate_next_break_pure(&config, now_10h, 0, None);
            assert!(
                result.is_none(),
                "Should not schedule break at 10:00 (outside range)"
            );
        }

        /// Notification time of zero (disabled notifications)
        /// Verifies that when `notification_before_s` = 0, no notification time is calculated
        #[test]
        fn notification_time_zero_disables_notifications() {
            let config = TestConfigBuilder::new()
                .mini_break_interval_s(120)
                .notification_before_s(0)
                .build();

            let now = Utc::now();
            let result = calculate_next_break_pure(&config, now, 0, None);

            let break_info = result.unwrap();
            assert!(
                break_info.notification_time.is_none(),
                "Notification time should be None when notification_before_s = 0"
            );
        }

        /// Notification time exceeding interval
        /// Tests graceful handling when notification time is longer than break interval
        #[test]
        fn notification_time_exceeding_interval_omits_notification() {
            let config = TestConfigBuilder::new()
                .mini_break_interval_s(60) // 1 minute interval
                .notification_before_s(120) // 2 minutes notification (exceeds interval)
                .build();

            let now = Utc::now();
            let result = calculate_next_break_pure(&config, now, 0, None);

            let break_info = result.unwrap();
            assert!(
                break_info.notification_time.is_none(),
                "Notification should be omitted when notification_before_s > interval_s"
            );
        }

        /// Multiple schedule configurations switching
        /// Tests that only the first matching enabled schedule is used
        #[test]
        fn uses_first_matching_schedule() {
            // Schedule 1: 09:00-12:00, 30-minute intervals
            let schedule1 = ScheduleSettings {
                name: "Morning".to_string(),
                enabled: true,
                time_range: time_range(9, 0, 12, 0),
                days_of_week: workdays(),
                notification_before_s: 10,
                mini_breaks: MiniBreakSettings {
                    interval_s: 1800, // 30 minutes
                    ..Default::default()
                },
                ..Default::default()
            };

            // Schedule 2: 13:00-17:00, 20-minute intervals
            let schedule2 = ScheduleSettings {
                name: "Afternoon".to_string(),
                enabled: true,
                time_range: time_range(13, 0, 17, 0),
                days_of_week: workdays(),
                notification_before_s: 5,
                mini_breaks: MiniBreakSettings {
                    interval_s: 1200, // 20 minutes
                    ..Default::default()
                },
                ..Default::default()
            };

            let config = AppConfig {
                schedules: vec![schedule1, schedule2],
                ..Default::default()
            };

            // Test at 10:00 local time (should use schedule1)
            let now_morning = test_datetime_with_local(2025, 9, 3, 10, 0, 0);

            let result = calculate_next_break_pure(&config, now_morning, 0, None);
            let break_info = result.unwrap();
            let break_duration = (break_info.break_time - now_morning).num_seconds();
            // Should be around 1800 seconds (30 minutes) from schedule1
            assert_duration_near(break_duration, 1800, 5);

            // Test at 14:00 local time (should use schedule2)
            let now_afternoon = test_datetime_with_local(2025, 9, 3, 14, 0, 0);

            let result = calculate_next_break_pure(&config, now_afternoon, 0, None);
            let break_info = result.unwrap();
            let break_duration = (break_info.break_time - now_afternoon).num_seconds();
            // Should be around 1200 seconds (20 minutes) from schedule2
            assert_duration_near(break_duration, 1200, 5);

            // Test at 12:30 local time (between schedules, should be None)
            let now_between = test_datetime_with_local(2025, 9, 3, 12, 30, 0);

            let result = calculate_next_break_pure(&config, now_between, 0, None);
            assert!(
                result.is_none(),
                "Should not find a schedule at 12:30 local time"
            );
        }

        /// Additional test: Long break triggering with different thresholds
        #[test]
        fn long_break_triggers_at_correct_threshold() {
            let config = TestConfigBuilder::new()
                .mini_break_interval_s(60)
                .long_break_after_mini_breaks(3) // Trigger after 3 mini breaks
                .build();

            let now = Utc::now();

            // Counter = 0,1,2 should give mini breaks
            for counter in 0..=2 {
                let result = calculate_next_break_pure(&config, now, counter, None);
                assert!(
                    matches!(result.unwrap().event, SchedulerEvent::MiniBreak(_)),
                    "Counter {counter} should trigger mini break"
                );
            }

            // Counter = 3 should give long break
            let result = calculate_next_break_pure(&config, now, 3, None);
            assert!(
                matches!(result.unwrap().event, SchedulerEvent::LongBreak(_)),
                "Counter 3 should trigger long break"
            );
        }

        /// Test: Disabled long breaks should never trigger
        #[test]
        fn disabled_long_breaks_never_trigger() {
            let config = TestConfigBuilder::new()
                .mini_break_interval_s(60)
                .long_break_after_mini_breaks(2)
                .long_breaks_enabled(false)
                .build();

            let now = Utc::now();

            // Even with counter >= threshold, should still give mini break
            let result = calculate_next_break_pure(&config, now, 5, None);
            assert!(
                matches!(result.unwrap().event, SchedulerEvent::MiniBreak(_)),
                "Should trigger mini break when long breaks are disabled"
            );
        }
    }
}
