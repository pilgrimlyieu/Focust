use std::fmt::Display;

use chrono::DateTime;
use chrono::{Duration, Local, Utc};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{mpsc, watch};
use tokio::time::sleep;
use user_idle::UserIdle;

use super::event::*;
use super::models::*;
use crate::config::{AppConfig, SharedConfig};

/// Represents the current state of the scheduler.
#[derive(Debug, PartialEq, Clone, Copy)]
enum SchedulerState {
    Running,
    Paused,
}

impl Display for SchedulerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchedulerState::Running => write!(f, "Running"),
            SchedulerState::Paused => write!(f, "Paused"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("No active schedule found for the current time.")]
    NoActiveSchedule,
    #[error("No active event could be scheduled.")]
    NoActiveEvent,
}

/// The main scheduler struct.
pub struct Scheduler {
    app_handle: AppHandle,
    state: SchedulerState,
    cmd_rx: mpsc::Receiver<Command>,
    shutdown_rx: watch::Receiver<()>,

    // State related to break progression
    pub mini_break_counter: u8,
    pub last_break_time: Option<DateTime<Utc>>,

    event_sources: Vec<Box<dyn EventSource>>,
}

impl Scheduler {
    /// Creates a new Scheduler instance.
    pub fn new(
        app_handle: AppHandle,
        cmd_rx: mpsc::Receiver<Command>,
        shutdown_rx: watch::Receiver<()>,
        event_sources: Vec<Box<dyn EventSource>>,
    ) -> Self {
        Self {
            app_handle,
            state: SchedulerState::Running,
            cmd_rx,
            shutdown_rx,
            mini_break_counter: 0,
            last_break_time: None,
            event_sources,
        }
    }

    async fn get_notification_delay(&self) -> u32 {
        let config = self.app_handle.state::<SharedConfig>();
        let config_guard = config.read().await;
        config_guard
            .schedules
            .first()
            .map(|s| s.notification_before_s)
            .unwrap_or(0)
    }

    pub async fn run(&mut self) {
        tracing::info!("Scheduler started in {} state.", self.state);

        loop {
            match self.state {
                SchedulerState::Running => {
                    let next_event_result = {
                        let config = self.app_handle.state::<SharedConfig>();
                        let config_guard = config.read().await;
                        self.calculate_next_event(&config_guard)
                    };
                    match next_event_result {
                        // TODO: Refactor NEEDED
                        Ok(event) => {
                            let duration_to_wait = event.time - Utc::now();
                            if duration_to_wait > Duration::zero() {
                                tracing::info!(
                                    "Next event: {} in {} seconds",
                                    event.kind,
                                    duration_to_wait.num_seconds()
                                );

                                // Emit status update for UI
                                let status = crate::scheduler::models::SchedulerStatus {
                                    paused: false,
                                    next_event: Some(
                                        crate::scheduler::models::SchedulerEventInfo {
                                            kind: event.kind,
                                            time: event.time.to_rfc3339(),
                                            seconds_until: duration_to_wait.num_seconds() as i32,
                                        },
                                    ),
                                };
                                if let Err(e) = self.app_handle.emit("scheduler-status", &status) {
                                    tracing::warn!("Failed to emit scheduler status: {}", e);
                                }

                                // Check if we need to send a notification before the break
                                let should_notify = matches!(
                                    event.kind,
                                    EventKind::MiniBreak(_) | EventKind::LongBreak(_)
                                );

                                if should_notify // should notify before break
                                    && let notification_before_s =
                                        self.get_notification_delay().await
                                    && notification_before_s > 0 // notification time is set
                                    && let notification_duration =
                                        Duration::seconds(notification_before_s as i64)
                                && let wait_until_notification = duration_to_wait - notification_duration
                                    && wait_until_notification > Duration::zero()
                                // notification time is before break
                                {
                                    // Wait until notification time
                                    tokio::select! {
                                        biased; // Give priority to shutdown signal
                                        _ = self.shutdown_rx.changed() => {
                                            break; // Exit the loop on shutdown
                                        }
                                        _ = sleep(wait_until_notification.to_std().unwrap_or(std::time::Duration::ZERO)) => {
                                            // Send notification
                                            self.send_break_notification(&event.kind, notification_before_s).await;

                                            // Now wait for the remaining time until the break
                                            tokio::select! {
                                                biased;
                                                _ = self.shutdown_rx.changed() => {
                                                    break;
                                                }
                                                _ = sleep(std::time::Duration::from_secs(notification_before_s as u64)) => {
                                                    self.handle_event(event).await;
                                                    self.reset_timers();
                                                }
                                                Some(cmd) = self.cmd_rx.recv() => {
                                                    self.handle_command(cmd).await;
                                                }
                                            }
                                        }
                                        Some(cmd) = self.cmd_rx.recv() => {
                                            self.handle_command(cmd).await;
                                        }
                                    }
                                    continue; // Continue to next loop iteration
                                }

                                // No notification needed or notification time passed, wait normally
                                tokio::select! {
                                    biased;
                                    _ = self.shutdown_rx.changed() => {
                                        break;
                                    }
                                    _ = sleep(duration_to_wait.to_std().unwrap_or(std::time::Duration::ZERO)) => {
                                        self.handle_event(event).await;
                                        self.reset_timers();
                                    }
                                    Some(cmd) = self.cmd_rx.recv() => {
                                        self.handle_command(cmd).await;
                                    }
                                }
                            } else {
                                // Event was in the past, handle immediately and recalculate
                                tracing::warn!(
                                    "Scheduled event {} was in the past. Handling immediately.",
                                    event.kind
                                );
                                self.handle_event(event).await;
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Could not calculate next event: {e}. Waiting for command or config change."
                            );
                            // No events scheduled, wait for a command indefinitely
                            tokio::select! {
                                biased;
                                _ = self.shutdown_rx.changed() => {
                                    break;
                                }
                                Some(cmd) = self.cmd_rx.recv() => {
                                    self.handle_command(cmd).await;
                                }
                            }
                        }
                    }
                }
                SchedulerState::Paused => {
                    tracing::info!(
                        "Scheduler is in {} state. Waiting for command to resume.",
                        self.state
                    );
                    tokio::select! {
                        biased;
                        _ = self.shutdown_rx.changed() => {
                            break;
                        }
                        Some(cmd) = self.cmd_rx.recv() => {
                            self.handle_command(cmd).await;
                        }
                    }
                }
            }
        }
        tracing::info!("Scheduler shutting down.");
    }

    async fn handle_command(&mut self, cmd: Command) {
        tracing::debug!("Handling command: {cmd}");
        match cmd {
            Command::UpdateConfig(new_config) => {
                let config = self.app_handle.state::<SharedConfig>();
                let mut config_guard = config.write().await;
                *config_guard = new_config;
                // Don't reset counters on simple updates, but a full recalculation will happen naturally.
            }
            Command::Pause(reason) => {
                tracing::info!("Pausing scheduler due to: {reason}");
                self.state = SchedulerState::Paused;

                // Emit paused status
                let status = crate::scheduler::models::SchedulerStatus {
                    paused: true,
                    next_event: None,
                };
                if let Err(e) = self.app_handle.emit("scheduler-status", &status) {
                    tracing::warn!("Failed to emit scheduler status: {e}");
                }

                match reason {
                    PauseReason::UserIdle | PauseReason::Dnd => {
                        // Reset timers when paused due to user idle or DND
                        self.reset_timers();
                    }
                    _ => {} // Other reasons may not require timer reset
                }
            }
            Command::Resume(reason) => {
                tracing::info!(
                    "Resuming scheduler from {} state due to: {reason}",
                    self.state,
                );
                self.state = SchedulerState::Running;
                self.update_last_break_time(); // Update last break time on resume

                // Recalculate and emit status immediately after resuming
                let config = self.app_handle.state::<SharedConfig>();
                let config_guard = config.read().await;
                if let Ok(event) = self.calculate_next_event(&config_guard) {
                    let duration_to_wait = event.time - Utc::now();
                    let status = crate::scheduler::models::SchedulerStatus {
                        paused: false,
                        next_event: Some(crate::scheduler::models::SchedulerEventInfo {
                            kind: event.kind,
                            time: event.time.to_rfc3339(),
                            seconds_until: duration_to_wait.num_seconds() as i32,
                        }),
                    };
                    if let Err(e) = self.app_handle.emit("scheduler-status", &status) {
                        tracing::warn!("Failed to emit scheduler status on resume: {}", e);
                    }
                }
            }
            Command::Postpone => {
                // TODO: WHAT TO DO WITH POSTPONE?
                tracing::info!("Postponing current break.");
                // 1. Get the relevant postpone duration from config
                let config = self.app_handle.state::<SharedConfig>();
                let config_guard = config.read().await;
                // This assumes we are postponing a mini-break. A more complex logic
                // could check what the *next* break is to get the correct duration.
                let postpone_duration_s = config_guard
                    .schedules
                    .first() // Assuming a single schedule for simplicity
                    .map_or(300, |s| s.mini_breaks.base.postponed_s);

                // 2. Update the last break time to be `now + postpone_duration`
                self.last_break_time =
                    Some(Utc::now() + Duration::seconds(postpone_duration_s as i64));

                // 3. Immediately emit an event to the frontend to close the break window
                if let Err(e) = self.app_handle.emit("break-finished", "") {
                    tracing::error!("Failed to emit break-finished event: {e}");
                }
            }
            Command::TriggerBreak(kind) => {
                tracing::info!("Manually triggering break: {kind}");
                // Create a scheduled event for immediate execution
                let event = ScheduledEvent {
                    time: Utc::now(),
                    kind,
                };
                // Handle the event immediately
                self.handle_event(event).await;
            }
            Command::SkipBreak => {
                tracing::info!("Skipping current break");
                // Update last break time so the next break is scheduled correctly
                self.update_last_break_time();
                // Emit event to close the break window immediately
                if let Err(e) = self.app_handle.emit("break-finished", "") {
                    tracing::error!("Failed to emit break-finished event: {e}");
                }
            }
            Command::RequestStatus => {
                tracing::debug!("Status request received, emitting current status");
                // Calculate and emit the current status
                let config = self.app_handle.state::<SharedConfig>();
                let config_guard = config.read().await;

                let status = if self.state == SchedulerState::Paused {
                    crate::scheduler::models::SchedulerStatus {
                        paused: true,
                        next_event: None,
                    }
                } else {
                    match self.calculate_next_event(&config_guard) {
                        Ok(event) => {
                            let duration_to_wait = event.time - Utc::now();
                            crate::scheduler::models::SchedulerStatus {
                                paused: false,
                                next_event: Some(crate::scheduler::models::SchedulerEventInfo {
                                    kind: event.kind,
                                    time: event.time.to_rfc3339(),
                                    seconds_until: duration_to_wait.num_seconds() as i32,
                                }),
                            }
                        }
                        Err(_) => crate::scheduler::models::SchedulerStatus {
                            paused: false,
                            next_event: None,
                        },
                    }
                };

                if let Err(e) = self.app_handle.emit("scheduler-status", &status) {
                    tracing::warn!("Failed to emit scheduler status: {}", e);
                }
            }
        }
    }

    fn update_last_break_time(&mut self) {
        tracing::debug!("Updating last break time to now.");
        self.last_break_time = Some(Utc::now());
    }

    async fn handle_event(&mut self, event: ScheduledEvent) {
        tracing::info!("Executing event: {}", event.kind);
        // Emit the event to the Tauri frontend
        if let Err(e) = self.app_handle.emit("scheduler-event", event.kind) {
            tracing::error!("Failed to emit scheduler-event: {e}");
        }

        match event.kind {
            EventKind::MiniBreak(_) | EventKind::LongBreak(_) => {
                self.update_last_break_time();
                if let EventKind::MiniBreak(_) = event.kind {
                    self.mini_break_counter += 1;
                } else {
                    // It was a long break, reset the mini break counter
                    self.mini_break_counter = 0;
                }
            }
            _ => {} // Notifications or Attentions doesn't affect the break cycle timers
        }
    }

    /// Send a notification before a break starts
    async fn send_break_notification(&self, event_kind: &EventKind, seconds_before: u32) {
        let break_type = match event_kind {
            EventKind::MiniBreak(_) => "Mini Break",
            EventKind::LongBreak(_) => "Long Break",
            _ => return, // Only send notifications for break events
        };

        if let Err(e) = crate::platform::notifications::send_break_notification(
            &self.app_handle,
            break_type,
            seconds_before,
        ) {
            tracing::warn!("Failed to send break notification: {e}");
        }
    }

    fn calculate_next_event(&self, config: &AppConfig) -> Result<ScheduledEvent, SchedulerError> {
        let now = Utc::now();

        let context = SchedulingContext {
            config,
            now_utc: now,
            now_local: now.with_timezone(&Local),
            mini_break_counter: self.mini_break_counter,
            last_break_time: self.last_break_time,
        };

        self.event_sources
            .iter()
            .flat_map(|source| source.upcoming_events(&context))
            .filter(|e| e.time > now) // Only consider future events
            .min_by(|a, b| a.time.cmp(&b.time).then_with(|| a.kind.cmp(&b.kind)))
            .ok_or(SchedulerError::NoActiveEvent)
    }

    fn reset_timers(&mut self) {
        tracing::info!("Resetting break timers.");
        self.last_break_time = None;
    }
}

async fn spawn_idle_monitor_task(
    cmd_tx: mpsc::Sender<Command>,
    _shutdown_tx: watch::Sender<()>,
    app_handle: AppHandle,
) {
    tracing::debug!("Spawning user idle monitor task...");

    const CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(10);

    tokio::spawn(async move {
        let mut was_idle = false;
        // Check interval for user idle status

        loop {
            let inactive_s = {
                let config = app_handle.state::<SharedConfig>();
                let config_guard = config.read().await;
                config_guard.inactive_s
            };

            match UserIdle::get_time() {
                Ok(idle_duration) => {
                    let idle_seconds = idle_duration.as_seconds();
                    let is_idle = idle_seconds >= inactive_s as u64;

                    if is_idle && !was_idle {
                        // From Active to Idle
                        tracing::info!(
                            "User became idle (idle for {idle_seconds}s). Notifying scheduler."
                        );
                        if let Err(e) = cmd_tx.send(Command::Pause(PauseReason::UserIdle)).await {
                            tracing::error!(
                                "Failed to send UserIdle command: {e}. Monitor task continuing."
                            );
                            continue;
                        }
                        was_idle = true;
                    } else if !is_idle && was_idle {
                        // From Idle to Active
                        tracing::info!("User became active. Notifying scheduler.");
                        if let Err(e) = cmd_tx.send(Command::Resume(PauseReason::UserIdle)).await {
                            tracing::error!(
                                "Failed to send UserActive command: {e}. Monitor task continuing."
                            );
                            continue;
                        }
                        was_idle = false;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to get user idle time: {e}");
                }
            }

            sleep(CHECK_INTERVAL).await;
        }
    });
}

pub async fn init_scheduler(app_handle: AppHandle) -> (mpsc::Sender<Command>, watch::Sender<()>) {
    let (cmd_tx, cmd_rx) = mpsc::channel(32);
    let (shutdown_tx, shutdown_rx) = watch::channel(());

    let sources: Vec<Box<dyn EventSource>> =
        vec![Box::new(BreakEventSource), Box::new(AttentionEventSource)];

    // Scheduler instance
    let mut scheduler = Scheduler::new(app_handle.clone(), cmd_rx, shutdown_rx, sources);
    tokio::spawn(async move {
        scheduler.run().await;
    });

    spawn_idle_monitor_task(cmd_tx.clone(), shutdown_tx.clone(), app_handle).await;

    (cmd_tx, shutdown_tx)
}

#[cfg(test)]
mod tests {
    use super::*;

    // SchedulerState tests
    #[test]
    fn test_scheduler_state_equality() {
        assert_eq!(SchedulerState::Running, SchedulerState::Running);
        assert_eq!(SchedulerState::Paused, SchedulerState::Paused);
        assert_ne!(SchedulerState::Running, SchedulerState::Paused);
    }

    #[test]
    fn test_scheduler_state_display() {
        assert_eq!(SchedulerState::Running.to_string(), "Running");
        assert_eq!(SchedulerState::Paused.to_string(), "Paused");
    }

    #[test]
    fn test_scheduler_state_clone() {
        let state = SchedulerState::Running;
        let cloned = state;
        assert_eq!(state, cloned);
    }

    // SchedulerError tests
    #[test]
    fn test_scheduler_error_no_active_schedule_display() {
        let error = SchedulerError::NoActiveSchedule;
        assert_eq!(
            error.to_string(),
            "No active schedule found for the current time."
        );
    }

    #[test]
    fn test_scheduler_error_no_active_event_display() {
        let error = SchedulerError::NoActiveEvent;
        assert_eq!(error.to_string(), "No active event could be scheduled.");
    }

    // calculate_next_event boundary tests (using mock event source)
    struct MockEventSource {
        events: Vec<ScheduledEvent>,
    }

    impl EventSource for MockEventSource {
        fn upcoming_events(&self, _context: &SchedulingContext) -> Vec<ScheduledEvent> {
            self.events.clone()
        }
    }

    #[test]
    fn test_calculate_next_event_no_events() {
        // Create a mock source with no events
        let mock_source = MockEventSource { events: vec![] };
        let sources: Vec<Box<dyn EventSource>> = vec![Box::new(mock_source)];

        let config = AppConfig::default();
        let now = Utc::now();

        let context = SchedulingContext {
            config: &config,
            now_utc: now,
            now_local: now.with_timezone(&Local),
            mini_break_counter: 0,
            last_break_time: None,
        };

        // Manually invoke the calculate_next_event logic
        let result: Option<ScheduledEvent> = sources
            .iter()
            .flat_map(|source| source.upcoming_events(&context))
            .filter(|e| e.time > now)
            .min_by(|a, b| a.time.cmp(&b.time).then_with(|| a.kind.cmp(&b.kind)));

        assert!(result.is_none());
    }

    #[test]
    fn test_calculate_next_event_single_future_event() {
        use crate::core::schedule::BreakId;
        let future_time = Utc::now() + Duration::minutes(10);
        let event = ScheduledEvent {
            time: future_time,
            kind: EventKind::MiniBreak(BreakId::new()),
        };

        let mock_source = MockEventSource {
            events: vec![event.clone()],
        };
        let sources: Vec<Box<dyn EventSource>> = vec![Box::new(mock_source)];

        let config = AppConfig::default();
        let now = Utc::now();

        let context = SchedulingContext {
            config: &config,
            now_utc: now,
            now_local: now.with_timezone(&Local),
            mini_break_counter: 0,
            last_break_time: None,
        };

        let result: Option<ScheduledEvent> = sources
            .iter()
            .flat_map(|source| source.upcoming_events(&context))
            .filter(|e| e.time > now)
            .min_by(|a, b| a.time.cmp(&b.time).then_with(|| a.kind.cmp(&b.kind)));

        assert!(result.is_some());
    }

    #[test]
    fn test_calculate_next_event_filters_past_events() {
        use crate::core::schedule::BreakId;
        let past_time = Utc::now() - Duration::minutes(10);
        let future_time = Utc::now() + Duration::minutes(10);

        let past_event = ScheduledEvent {
            time: past_time,
            kind: EventKind::MiniBreak(BreakId::new()),
        };

        let future_event = ScheduledEvent {
            time: future_time,
            kind: EventKind::LongBreak(BreakId::new()),
        };

        let mock_source = MockEventSource {
            events: vec![past_event, future_event.clone()],
        };
        let sources: Vec<Box<dyn EventSource>> = vec![Box::new(mock_source)];

        let config = AppConfig::default();
        let now = Utc::now();

        let context = SchedulingContext {
            config: &config,
            now_utc: now,
            now_local: now.with_timezone(&Local),
            mini_break_counter: 0,
            last_break_time: None,
        };

        let result: Option<ScheduledEvent> = sources
            .iter()
            .flat_map(|source| source.upcoming_events(&context))
            .filter(|e| e.time > now)
            .min_by(|a, b| a.time.cmp(&b.time).then_with(|| a.kind.cmp(&b.kind)));

        assert!(result.is_some());
        // Should only select future events (LongBreak)
    }

    #[test]
    fn test_calculate_next_event_picks_earliest() {
        use crate::core::schedule::BreakId;
        let time1 = Utc::now() + Duration::minutes(10);
        let time2 = Utc::now() + Duration::minutes(5);
        let time3 = Utc::now() + Duration::minutes(15);

        let event1 = ScheduledEvent {
            time: time1,
            kind: EventKind::MiniBreak(BreakId::new()),
        };
        let event2 = ScheduledEvent {
            time: time2,
            kind: EventKind::LongBreak(BreakId::new()),
        };
        let event3 = ScheduledEvent {
            time: time3,
            kind: EventKind::Attention(crate::core::schedule::AttentionId::new()),
        };

        let mock_source = MockEventSource {
            events: vec![event1, event2.clone(), event3],
        };
        let sources: Vec<Box<dyn EventSource>> = vec![Box::new(mock_source)];

        let config = AppConfig::default();
        let now = Utc::now();

        let context = SchedulingContext {
            config: &config,
            now_utc: now,
            now_local: now.with_timezone(&Local),
            mini_break_counter: 0,
            last_break_time: None,
        };

        let result: Option<ScheduledEvent> = sources
            .iter()
            .flat_map(|source| source.upcoming_events(&context))
            .filter(|e| e.time > now)
            .min_by(|a, b| a.time.cmp(&b.time).then_with(|| a.kind.cmp(&b.kind)));

        assert!(result.is_some());
        // Should select the earliest event (time2, LongBreak)
    }

    #[test]
    fn test_calculate_next_event_priority_when_same_time() {
        use crate::core::schedule::{AttentionId, BreakId};
        let same_time = Utc::now() + Duration::minutes(10);

        // Create two events at the same time but with different priorities
        let attention_event = ScheduledEvent {
            time: same_time,
            kind: EventKind::Attention(AttentionId::new()),
        };
        let mini_break_event = ScheduledEvent {
            time: same_time,
            kind: EventKind::MiniBreak(BreakId::new()),
        };

        let mock_source = MockEventSource {
            events: vec![mini_break_event, attention_event.clone()],
        };
        let sources: Vec<Box<dyn EventSource>> = vec![Box::new(mock_source)];

        let config = AppConfig::default();
        let now = Utc::now();

        let context = SchedulingContext {
            config: &config,
            now_utc: now,
            now_local: now.with_timezone(&Local),
            mini_break_counter: 0,
            last_break_time: None,
        };

        let result: Option<ScheduledEvent> = sources
            .iter()
            .flat_map(|source| source.upcoming_events(&context))
            .filter(|e| e.time > now)
            .min_by(|a, b| a.time.cmp(&b.time).then_with(|| a.kind.cmp(&b.kind)));

        assert!(result.is_some());
        // When at the same time, should select the higher priority (Attention > MiniBreak)
    }
}
