use std::fmt::Display;

use chrono::DateTime;
use chrono::{Duration, Local, Utc};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{mpsc, watch};
use tokio::time::sleep;
use user_idle::UserIdle;

use super::event::*;
use super::models::*;
use crate::config::SharedConfig;

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
    mini_break_counter: u8,
    last_break_time: Option<DateTime<Utc>>,

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

    pub async fn run(&mut self) {
        log::info!("Scheduler started in {} state.", self.state);

        loop {
            match self.state {
                SchedulerState::Running => {
                    match self.calculate_next_event().await {
                        Ok(event) => {
                            let duration_to_wait = event.time - Utc::now();
                            if duration_to_wait > Duration::zero() {
                                log::info!(
                                    "Next event: {} in {} seconds",
                                    event.kind,
                                    duration_to_wait.num_seconds()
                                );
                                tokio::select! {
                                    biased;
                                    _ = self.shutdown_rx.changed() => {
                                        break;
                                    }
                                    _ = sleep(duration_to_wait.to_std().unwrap()) => {
                                        self.handle_event(event).await; // Handle the event when the time comes
                                        self.reset_timers(); // Reset timers after handling the event
                                    }
                                    Some(cmd) = self.cmd_rx.recv() => {
                                        self.handle_command(cmd).await;
                                    }
                                }
                            } else {
                                // Event was in the past, handle immediately and recalculate
                                log::warn!(
                                    "Scheduled event {} was in the past. Handling immediately.",
                                    event.kind
                                );
                                self.handle_event(event).await;
                            }
                        }
                        Err(e) => {
                            log::warn!(
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
                    log::info!(
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
        log::info!("Scheduler shutting down.");
    }

    async fn handle_command(&mut self, cmd: Command) {
        log::debug!("Handling command: {cmd}");
        match cmd {
            Command::UpdateConfig(new_config) => {
                let config = self.app_handle.state::<SharedConfig>();
                let mut config_guard = config.write().await;
                *config_guard = new_config;
                // Don't reset counters on simple updates, but a full recalculation will happen naturally.
            }
            Command::Pause(reason) => {
                log::info!("Pausing scheduler due to: {reason}");
                self.state = SchedulerState::Paused;
                match reason {
                    PauseReason::UserIdle | PauseReason::Dnd => {
                        // Reset timers when paused due to user idle or DND
                        self.reset_timers();
                    }
                    _ => {} // Other reasons may not require timer reset
                }
            }
            Command::Resume(reason) => {
                log::info!(
                    "Resuming scheduler from {} state due to: {reason}",
                    self.state,
                );
                self.state = SchedulerState::Running;
                self.update_last_break_time(); // Update last break time on resume
            }
            Command::Postpone => {
                // TODO: WHAT TO DO WITH POSTPONE?
                log::info!("Postponing current break.");
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
                self.app_handle.emit("break-finished", "").unwrap();
            }
        }
    }

    fn update_last_break_time(&mut self) {
        log::debug!("Updating last break time to now.");
        self.last_break_time = Some(Utc::now());
    }

    async fn handle_event(&mut self, event: ScheduledEvent) {
        log::info!("Executing event: {}", event.kind);
        // Emit the event to the Tauri frontend
        self.app_handle.emit("scheduler-event", event.kind).unwrap();

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
            _ => {
                // Notifications or Attentions don't affect the break cycle timers
            }
        }
    }

    async fn calculate_next_event(&self) -> Result<ScheduledEvent, SchedulerError> {
        let config = self.app_handle.state::<SharedConfig>();
        let config_guard = config.read().await;
        let now = Utc::now();

        let context = SchedulingContext {
            config: &config_guard,
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
        log::info!("Resetting break timers.");
        self.last_break_time = None;
    }
}

async fn spawn_idle_monitor_task(cmd_tx: mpsc::Sender<Command>, app_handle: AppHandle) {
    log::debug!("Spawning user idle monitor task...");

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
                    let is_idle = idle_seconds >= inactive_s;

                    if is_idle && !was_idle {
                        // From Active to Idle
                        log::info!(
                            "User became idle (idle for {idle_seconds}s). Notifying scheduler."
                        );
                        if let Err(e) = cmd_tx.send(Command::Pause(PauseReason::UserIdle)).await {
                            log::error!(
                                "Failed to send UserIdle command: {e}. Monitor task continuing."
                            );
                            continue;
                        }
                        was_idle = true;
                    } else if !is_idle && was_idle {
                        // From Idle to Active
                        log::info!("User became active. Notifying scheduler.");
                        if let Err(e) = cmd_tx.send(Command::Resume(PauseReason::UserIdle)).await {
                            log::error!(
                                "Failed to send UserActive command: {e}. Monitor task continuing."
                            );
                            continue;
                        }
                        was_idle = false;
                    }
                }
                Err(e) => {
                    log::error!("Failed to get user idle time: {e}");
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

    spawn_idle_monitor_task(cmd_tx.clone(), app_handle).await;

    (cmd_tx, shutdown_tx)
}
