use std::fmt::Display;

use chrono::offset::LocalResult;
use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, Weekday};
use chrono::{Duration, Local, Utc};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;
use tokio::time::sleep;
use user_idle::UserIdle;

use super::models::*;
use crate::config::{AppConfig, SharedConfig};
use crate::core::schedule::{AttentionSettings, ScheduleSettings};

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

/// The main scheduler struct.
pub struct Scheduler {
    app_handle: AppHandle,
    state: SchedulerState,
    cmd_rx: mpsc::Receiver<Command>,

    // State related to break progression
    mini_break_counter: u8,
    last_break_time: Option<DateTime<Utc>>,
}

impl Scheduler {
    /// Creates a new Scheduler instance.
    pub fn new(app_handle: AppHandle, cmd_rx: mpsc::Receiver<Command>) -> Self {
        Self {
            app_handle,
            state: SchedulerState::Running,
            cmd_rx,
            mini_break_counter: 0,
            last_break_time: None,
        }
    }

    pub async fn run(&mut self) {
        log::info!("Scheduler started in {:?} state.", self.state);

        loop {
            match self.state {
                SchedulerState::Running => {
                    let next_event = self.calculate_next_event().await;

                    if let Some(event) = next_event {
                        let duration_to_wait = event.time - Utc::now();
                        if duration_to_wait > Duration::zero() {
                            log::info!(
                                "Next event: {:?} in {:.1} seconds",
                                event.kind,
                                duration_to_wait.num_seconds()
                            );
                            tokio::select! {
                                _ = sleep(duration_to_wait.to_std().unwrap()) => {
                                    self.handle_event(event).await; // Handle the event when the time comes
                                    self.reset_timers(); // Reset timers after handling the event
                                }
                                Some(cmd) = self.cmd_rx.recv() => {
                                    if self.handle_command(cmd).await {
                                        break; // Shutdown command received
                                    }
                                }
                            }
                        } else {
                            // Event was in the past, handle immediately and recalculate
                            log::warn!(
                                "Scheduled event {:?} was in the past. Handling immediately.",
                                event.kind
                            );
                            self.handle_event(event).await;
                        }
                    } else {
                        log::info!("No upcoming events. Waiting for command.");
                        // No events scheduled, wait for a command indefinitely
                        if let Some(cmd) = self.cmd_rx.recv().await {
                            if self.handle_command(cmd).await {
                                break; // Shutdown command received
                            }
                        }
                    }
                }
                SchedulerState::Paused => {
                    log::info!(
                        "Scheduler is in {:?} state. Waiting for command to resume.",
                        self.state
                    );
                    if let Some(cmd) = self.cmd_rx.recv().await {
                        if self.handle_command(cmd).await {
                            break; // Shutdown command received
                        }
                    }
                }
            }
        }
        log::info!("Scheduler shutting down.");
    }

    async fn handle_command(&mut self, cmd: Command) -> bool {
        log::debug!("Handling command: {cmd:?}");
        match cmd {
            Command::UpdateConfig(new_config) => {
                let config = self.app_handle.state::<SharedConfig>();
                let mut config_guard = config.write().await;
                *config_guard = new_config;
                // Don't reset counters on simple updates, but a full recalculation will happen naturally.
            }
            Command::Pause(reason) => {
                log::info!("Pausing scheduler due to: {reason:?}");
                self.state = SchedulerState::Paused;
                match reason {
                    PauseReason::UserIdle | PauseReason::Dnd => {
                        // Reset timers when paused due to user idle or DND
                        self.reset_timers();
                    }
                    _ => {
                        // Other reasons may not require timer reset
                    }
                }
            }
            Command::Resume(reason) => {
                log::info!(
                    "Resuming scheduler from {:?} state due to: {:?}",
                    self.state,
                    reason
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
            Command::Shutdown => {
                log::info!("Received shutdown command. Stopping scheduler.");
                return true; // Indicate that the scheduler should shut down
            }
        }
        false
    }

    fn update_last_break_time(&mut self) {
        self.last_break_time = Some(Utc::now());
    }

    async fn handle_event(&mut self, event: ScheduledEvent) {
        log::info!("Executing event: {:?}", event.kind);
        // Emit the event to the Tauri frontend
        self.app_handle.emit("scheduler-event", event.kind).unwrap();

        match event.kind {
            EventKind::MiniBreak(_) | EventKind::LongBreak(_) => {
                self.last_break_time = Some(Utc::now());
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

    async fn calculate_next_event(&self) -> Option<ScheduledEvent> {
        let config = self.app_handle.state::<SharedConfig>();
        let config_guard = config.read().await;
        let now = Utc::now();
        let local_now = now.with_timezone(&Local);

        let active_schedule =
            self.get_active_schedule(&config_guard, local_now.time(), local_now.weekday())?;

        let mut potential_events: Vec<ScheduledEvent> = Vec::new();

        // 1. Calculate Break Events
        if active_schedule.mini_breaks.base.enabled || active_schedule.long_breaks.base.enabled {
            let is_long_break_due = self.is_long_break_due(active_schedule);

            // Determine the type of break to schedule
            let (break_kind, break_settings) = if is_long_break_due {
                (
                    EventKind::LongBreak(active_schedule.long_breaks.base.id),
                    &active_schedule.long_breaks.base,
                )
            } else {
                (
                    EventKind::MiniBreak(active_schedule.mini_breaks.base.id),
                    &active_schedule.mini_breaks.base,
                )
            };

            // Only schedule a break if its type is enabled
            if break_settings.enabled {
                let interval = Duration::seconds(active_schedule.mini_breaks.interval_s as i64);
                let break_time = self.last_break_time.unwrap_or_else(Utc::now) + interval;

                potential_events.push(ScheduledEvent {
                    time: break_time,
                    kind: break_kind,
                });

                // 2. Calculate Notification for the Break
                // Notification is enabled if the break has a positive notification time set
                if active_schedule.has_notification() {
                    let notification_time = break_time
                        - Duration::seconds(active_schedule.notification_before_s as i64);
                    let notification_kind = match break_kind {
                        EventKind::LongBreak(id) => NotificationKind::LongBreak(id),
                        EventKind::MiniBreak(id) => NotificationKind::MiniBreak(id),
                        _ => unreachable!(),
                    };
                    potential_events.push(ScheduledEvent {
                        time: notification_time,
                        kind: EventKind::Notification(notification_kind),
                    });
                }
            }
        }

        // 3. Calculate Attention Events
        for attention in &config_guard.attentions {
            if attention.enabled {
                if let Some(next_attention_time) =
                    self.get_next_attention_time(attention, local_now)
                {
                    potential_events.push(ScheduledEvent {
                        time: next_attention_time,
                        kind: EventKind::Attention(attention.id),
                    });
                }
            }
        }

        // 4. Find the earliest, highest-priority event
        potential_events
            .into_iter()
            .filter(|e| e.time > now) // Only consider future events
            .min_by(|a, b| a.time.cmp(&b.time).then_with(|| a.kind.cmp(&b.kind)))
    }

    fn get_active_schedule<'a>(
        &self,
        config: &'a AppConfig,
        now_time: NaiveTime,
        now_day: Weekday,
    ) -> Option<&'a ScheduleSettings> {
        config.schedules.iter().find(|s| {
            s.enabled && s.days_of_week.contains(&now_day) && s.time_range.contains(&now_time)
        })
    }
    fn is_long_break_due(&self, schedule: &ScheduleSettings) -> bool {
        schedule.long_breaks.base.enabled
            && self.mini_break_counter >= schedule.long_breaks.after_mini_breaks
    }

    fn get_next_attention_time(
        &self,
        attention: &AttentionSettings,
        now: DateTime<Local>,
    ) -> Option<DateTime<Utc>> {
        let now_date = now.date_naive();
        let now_time = now.time();
        let to_utc = |dt_local: DateTime<Local>| -> Option<DateTime<Utc>> {
            log::debug!(
                "Found potential attention time: {} (local)",
                dt_local.to_rfc2822()
            );
            Some(dt_local.with_timezone(&Utc))
        };
        let build_datetime = |date: NaiveDate, time: NaiveTime| -> Option<DateTime<Local>> {
            match date.and_time(time).and_local_timezone(Local) {
                LocalResult::Single(dt) => Some(dt),
                LocalResult::Ambiguous(dt1, _) => {
                    log::warn!(
                        "Ambiguous local time encountered for {time:?} on {date}. Using first option."
                    );
                    Some(dt1)
                }
                LocalResult::None => {
                    log::error!("No valid local time found for {time:?} on {date:?}.");
                    None
                }
            }
        };

        // Check if the attention is within the current time range
        if attention.days_of_week.contains(&now.weekday()) {
            if let Some(next_time_today) = attention.times.earliest_after(&now_time) {
                if let Some(dt_local) = build_datetime(now_date, next_time_today) {
                    return to_utc(dt_local);
                }
            }
        }

        // Find the first occurrence in the next 7 days
        for i in 1..=7 {
            let next_date = now_date + chrono::Duration::days(i);
            if attention.days_of_week.contains(&next_date.weekday()) {
                if let Some(first_time) = attention.times.first() {
                    if let Some(dt_local) = build_datetime(next_date, first_time) {
                        return to_utc(dt_local);
                    }
                }
            }
        }

        // No valid attention time found in the next 7 days
        log::warn!(
            "No scheduled time found for attention '{}' in the next 7 days.",
            attention.name
        );
        None
    }

    fn reset_timers(&mut self) {
        log::info!("Resetting break timers.");
        self.last_break_time = None;
    }
}

async fn spawn_idle_monitor_task(cmd_tx: mpsc::Sender<Command>, app_handle: AppHandle) {
    log::info!("Spawning user idle monitor task...");
    tokio::spawn(async move {
        let mut was_idle = false;
        // Check interval for user idle status
        let check_interval = std::time::Duration::from_secs(10);

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
                                "Failed to send UserIdle command: {e}. Monitor task shutting down."
                            );
                            break; // Channel closed, task cannot continue, exit loop
                        }
                        was_idle = true;
                    } else if !is_idle && was_idle {
                        // From Idle to Active
                        log::info!("User became active. Notifying scheduler.");
                        if let Err(e) = cmd_tx.send(Command::Resume(PauseReason::UserIdle)).await {
                            log::error!(
                                "Failed to send UserActive command: {e}. Monitor task shutting down."
                            );
                            break; // Channel closed, task cannot continue, exit loop
                        }
                        was_idle = false;
                    }
                }
                Err(e) => {
                    log::error!("Failed to get user idle time: {e:?}");
                }
            }

            sleep(check_interval).await;
        }
    });
}

pub async fn init_scheduler(app_handle: AppHandle) -> mpsc::Sender<Command> {
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    // Scheduler instance
    let mut scheduler = Scheduler::new(app_handle.clone(), cmd_rx);
    tokio::spawn(async move {
        scheduler.run().await;
    });

    spawn_idle_monitor_task(cmd_tx.clone(), app_handle).await;

    cmd_tx
}
