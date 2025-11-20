use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;

use chrono::offset::LocalResult;
use chrono::{DateTime, Duration, Local, Utc};
use chrono::{Datelike, NaiveDate, NaiveTime};
use futures::future::pending;
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::{mpsc, watch};
use tokio::time::sleep;

use super::event_emitter::EventEmitter;
use super::models::{Command, PauseReason, SchedulerEvent};
use super::shared_state::SharedState;
use crate::core::schedule::AttentionSettings;
use crate::platform::create_prompt_windows;
use crate::{config::SharedConfig, core::schedule::AttentionId};

/// Information about a scheduled attention
#[derive(Debug, Clone)]
struct AttentionInfo {
    attention_id: AttentionId,
    attention_time: DateTime<Utc>,
}

/// The state of the attention timer
#[derive(Debug, Clone)]
enum AttentionTimerState {
    /// Paused
    Paused(PauseReason),
    /// No enabled attentions
    Idle,
    /// Waiting for attention to trigger
    WaitingForAttention(AttentionInfo),
}

impl Display for AttentionTimerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttentionTimerState::Paused(reason) => write!(f, "Paused({reason})"),
            AttentionTimerState::Idle => write!(f, "Idle"),
            AttentionTimerState::WaitingForAttention(info) => {
                write!(f, "WaitingForAttention({})", info.attention_id)
            }
        }
    }
}

/// A simple timer for attention reminders
/// Attention timer can be paused/resumed like breaks
pub struct AttentionTimer<E, R = tauri::Wry>
where
    E: EventEmitter,
    R: Runtime,
{
    app_handle: AppHandle<R>,
    #[expect(unused)]
    event_emitter: E,
    shutdown_rx: watch::Receiver<()>,
    state: AttentionTimerState,
    shared_state: SharedState,
}

impl<E, R> AttentionTimer<E, R>
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
            state: AttentionTimerState::Idle,
            shared_state,
        }
    }

    /// Get current state as string (for testing)
    #[cfg(test)]
    #[expect(dead_code)]
    pub fn get_state(&self) -> String {
        format!("{}", self.state)
    }

    /// Get next attention info (for testing)
    #[cfg(test)]
    #[expect(dead_code)]
    pub fn get_next_attention_info(&self) -> Option<(AttentionId, DateTime<Utc>)> {
        match &self.state {
            AttentionTimerState::WaitingForAttention(info) => {
                Some((info.attention_id, info.attention_time))
            }
            _ => None,
        }
    }

    pub async fn run(&mut self, mut cmd_rx: mpsc::Receiver<Command>) {
        tracing::info!("AttentionTimer started");

        // Only transition to calculating if not paused
        if !matches!(self.state, AttentionTimerState::Paused(_)) {
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
                    tracing::info!("AttentionTimer received shutdown");
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

        tracing::info!("AttentionTimer shutting down");
    }

    /// Get the duration of next timer based on current state
    fn get_duration_for_current_state(&self) -> Option<Duration> {
        let now = Utc::now();
        match &self.state {
            AttentionTimerState::WaitingForAttention(info) => Some(info.attention_time - now),
            AttentionTimerState::Paused(_) | AttentionTimerState::Idle => None,
        }
    }

    /// Handle timer fired event based on current state
    async fn on_timer_fired(&mut self) {
        match self.state.clone() {
            AttentionTimerState::WaitingForAttention(info) => {
                tracing::debug!("Timer fired: executing attention");
                self.execute_attention(info.attention_id).await;
            }
            _ => {
                tracing::warn!("Timer fired in unexpected state: {}", self.state);
            }
        }
    }

    /// Execute an attention reminder: create window and play audio
    async fn execute_attention(&mut self, attention_id: AttentionId) {
        tracing::info!("Executing attention: {attention_id}");

        // Mark attention session as started
        self.shared_state.write().start_attention_session();

        let event = SchedulerEvent::Attention(attention_id);

        let app_handle = self.app_handle.clone();
        tauri::async_runtime::spawn(async move {
            create_prompt_windows(&app_handle, event, 0)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to create attention windows: {e}");
                });
        });

        // Recalculate next attention
        self.transition_to_calculating().await;
    }

    /// Handle incoming commands
    async fn handle_command(&mut self, cmd: Command) {
        tracing::debug!("AttentionTimer handling command: {cmd}");

        match cmd {
            Command::Pause(reason) => {
                self.handle_pause_command(reason);
            }
            Command::Resume(_reason) => {
                self.handle_resume_command().await;
            }
            Command::UpdateConfig(new_config) => {
                self.handle_update_config_command(new_config).await;
            }
            Command::TriggerEvent(SchedulerEvent::Attention(attention_id)) => {
                self.handle_trigger_event_command(attention_id).await;
            }
            Command::PromptFinished(SchedulerEvent::Attention(_)) => {
                self.handle_prompt_finished_command().await;
            }
            // AttentionTimer ignores other commands (they're for BreakScheduler)
            _ => {}
        }
    }

    /// Transition to calculating next attention
    async fn transition_to_calculating(&mut self) {
        let attention_info = {
            let config = self.app_handle.state::<SharedConfig>();
            let config_guard = config.read().await;
            calculate_next_attention(&config_guard.attentions).map(|(id, time)| AttentionInfo {
                attention_id: id,
                attention_time: time,
            })
        };

        if let Some(info) = attention_info {
            let now = Utc::now();
            if info.attention_time <= now {
                tracing::warn!("Attention time already passed, executing immediately");
                Box::pin(self.execute_attention(info.attention_id)).await;
            } else {
                let duration_to_wait = info.attention_time - now;
                tracing::info!(
                    "Transitioning to WaitingForAttention, next in {} seconds",
                    duration_to_wait.num_seconds()
                );
                self.state = AttentionTimerState::WaitingForAttention(info);
            }
        } else {
            tracing::info!("Transitioning to Idle (no enabled attentions)");
            self.state = AttentionTimerState::Idle;
        }
    }

    /// Handle Pause command
    fn handle_pause_command(&mut self, reason: PauseReason) {
        tracing::info!("Pausing AttentionTimer: {reason}");
        self.state = AttentionTimerState::Paused(reason);
    }

    /// Handle Resume command
    async fn handle_resume_command(&mut self) {
        tracing::info!("Resuming AttentionTimer");
        if matches!(self.state, AttentionTimerState::Paused(_)) {
            self.transition_to_calculating().await;
        }
    }

    /// Handle `UpdateConfig` command
    async fn handle_update_config_command(&mut self, new_config: crate::config::AppConfig) {
        tracing::debug!("Updating config in AttentionTimer");
        {
            let config = self.app_handle.state::<SharedConfig>();
            let mut config_guard = config.write().await;
            *config_guard = new_config;
        }

        // Only transition to calculating if not paused
        if matches!(self.state, AttentionTimerState::Paused(_)) {
            tracing::debug!("Config updated while paused, staying in paused state");
        } else {
            self.transition_to_calculating().await;
        }
    }

    /// Handle `TriggerEvent` command
    async fn handle_trigger_event_command(&mut self, attention_id: AttentionId) {
        tracing::info!("Manually triggering attention: {attention_id}");
        self.execute_attention(attention_id).await;
    }

    /// Handle `PromptFinished` command
    async fn handle_prompt_finished_command(&mut self) {
        tracing::debug!("Attention prompt finished, ending session");
        self.shared_state.write().end_attention_session();
        // Recalculate next attention after prompt finishes
        self.transition_to_calculating().await;
    }
}

/// Calculate the next attention time across all enabled attentions
pub(crate) fn calculate_next_attention(
    attentions: &[AttentionSettings],
) -> Option<(AttentionId, DateTime<Utc>)> {
    let now = Utc::now();
    let now_local = now.with_timezone(&Local);

    attentions
        .iter()
        .filter_map(|attention| {
            get_next_attention_time(attention, now_local).map(|time| (attention.id, time))
        })
        .min_by_key(|(_, time)| *time)
}

/// Get the next occurrence time for a specific attention
pub(crate) fn get_next_attention_time(
    attention: &AttentionSettings,
    now: DateTime<Local>,
) -> Option<DateTime<Utc>> {
    if !attention.enabled || attention.times.is_empty() || attention.days_of_week.is_empty() {
        tracing::debug!(
            "Attention '{}' is disabled or has no times/days configured.",
            attention.name
        );
        return None;
    }

    let now_date = now.date_naive();
    let now_time = now.time();

    let to_utc = |dt_local: DateTime<Local>| -> Option<DateTime<Utc>> {
        tracing::debug!(
            "Found potential attention '{}' time: {} (local)",
            attention.name,
            dt_local.to_rfc2822()
        );
        Some(dt_local.with_timezone(&Utc))
    };

    let build_datetime = |date: NaiveDate, time: NaiveTime| -> Option<DateTime<Local>> {
        match date.and_time(time).and_local_timezone(Local) {
            LocalResult::Single(dt) => Some(dt),
            LocalResult::Ambiguous(dt1, _) => {
                tracing::warn!(
                    "Ambiguous local time encountered for {time} on {date}. Using the first one."
                );
                Some(dt1)
            }
            LocalResult::None => {
                tracing::error!("No valid local time found for {time} on {date}.");
                None
            }
        }
    };

    // Check if there's a time today
    if attention.days_of_week.contains(&now.weekday())
        && let Some(next_time_today) = attention.times.earliest_after(&now_time)
        && let Some(dt_local) = build_datetime(now_date, next_time_today)
    {
        return to_utc(dt_local);
    }

    // Check next 7 days
    for i in 1..=7 {
        let next_date = now_date + chrono::Duration::days(i);
        if attention.days_of_week.contains(&next_date.weekday())
            && let Some(first_time) = attention.times.first()
            && let Some(dt_local) = build_datetime(next_date, first_time)
        {
            return to_utc(dt_local);
        }
    }
    tracing::error!(
        "No valid attention time found for '{}' within the next 7 days. This may indicate a bug or edge case in date calculations.",
        attention.name
    );
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::schedule::AttentionSettings;
    use crate::core::time::ShortTimes;
    use crate::scheduler::test_helpers::*;
    use chrono::{Timelike, Weekday};

    mod get_next_attention_time_tests {
        use super::*;

        #[test]
        fn returns_next_time_today() {
            let attention = AttentionSettings {
                enabled: true,
                times: ShortTimes::new(vec![naive_time(10, 0, 0), naive_time(14, 0, 0)]),
                days_of_week: all_weekdays(),
                ..Default::default()
            };

            let now = get_test_local_datetime(2025, 9, 3, 9, 0, 0);

            let result = get_next_attention_time(&attention, now);

            let next_time = result.unwrap();
            assert_eq!(next_time.with_timezone(&Local).hour(), 10);
            assert_eq!(next_time.with_timezone(&Local).day(), 3);
        }

        #[test]
        fn returns_next_time_tomorrow() {
            let attention = AttentionSettings {
                enabled: true,
                times: ShortTimes::new(vec![naive_time(10, 0, 0)]),
                days_of_week: all_weekdays(),
                ..Default::default()
            };

            let now = get_test_local_datetime(2025, 9, 3, 15, 0, 0);

            let result = get_next_attention_time(&attention, now);

            let next_time = result.unwrap();
            assert_eq!(next_time.with_timezone(&Local).hour(), 10);
            assert_eq!(next_time.with_timezone(&Local).day(), 4);
        }

        #[test]
        fn returns_none_when_disabled() {
            let attention = AttentionSettings {
                enabled: false,
                times: ShortTimes::new(vec![naive_time(10, 0, 0)]),
                days_of_week: all_weekdays(),
                ..Default::default()
            };

            let now = get_test_local_datetime(2025, 9, 3, 9, 0, 0);

            let result = get_next_attention_time(&attention, now);
            assert!(result.is_none());
        }

        #[test]
        fn returns_none_when_no_times() {
            let attention = AttentionSettings {
                enabled: true,
                times: ShortTimes::default(), // Empty
                days_of_week: all_weekdays(),
                ..Default::default()
            };

            let now = get_test_local_datetime(2025, 9, 3, 9, 0, 0);

            let result = get_next_attention_time(&attention, now);
            assert!(result.is_none());
        }

        #[test]
        fn skips_to_next_matching_day() {
            let attention = AttentionSettings {
                enabled: true,
                times: ShortTimes::new(vec![naive_time(10, 0, 0)]),
                days_of_week: vec![Weekday::Mon, Weekday::Wed, Weekday::Fri],
                ..Default::default()
            };

            let now = get_test_local_datetime(2025, 9, 2, 9, 0, 0);

            let result = get_next_attention_time(&attention, now);

            let next_time = result.unwrap();
            assert_eq!(next_time.with_timezone(&Local).weekday(), Weekday::Wed);
        }

        #[test]
        fn handles_multiple_times_in_day() {
            let attention = AttentionSettings {
                enabled: true,
                times: ShortTimes::new(vec![
                    naive_time(9, 0, 0),
                    naive_time(12, 0, 0),
                    naive_time(15, 0, 0),
                ]),
                days_of_week: all_weekdays(),
                ..Default::default()
            };

            let now = get_test_local_datetime(2025, 9, 3, 10, 0, 0);

            let result = get_next_attention_time(&attention, now);

            let next_time = result.unwrap();
            assert_eq!(next_time.with_timezone(&Local).hour(), 12);
            assert_eq!(next_time.with_timezone(&Local).day(), 3);
        }
    }

    mod calculate_next_attention_tests {
        use super::*;

        #[test]
        #[expect(clippy::similar_names)]
        fn returns_none_when_all_disabled() {
            let attention1 = AttentionSettings {
                enabled: false,
                times: ShortTimes::new(vec![naive_time(10, 0, 0)]),
                days_of_week: all_weekdays(),
                ..Default::default()
            };

            let attention2 = AttentionSettings {
                enabled: false,
                times: ShortTimes::new(vec![naive_time(14, 0, 0)]),
                days_of_week: all_weekdays(),
                ..Default::default()
            };

            let attentions = vec![attention1, attention2];

            let result = calculate_next_attention(&attentions);
            assert!(result.is_none());
        }

        #[test]
        fn returns_none_when_empty_list() {
            let attentions = vec![];
            let result = calculate_next_attention(&attentions);
            assert!(result.is_none());
        }

        #[test]
        fn ignores_invalid_attentions() {
            let valid_attention = AttentionSettings {
                enabled: true,
                times: ShortTimes::new(vec![naive_time(10, 0, 0)]),
                days_of_week: all_weekdays(),
                ..Default::default()
            };

            let no_times_attention = AttentionSettings {
                enabled: true,
                times: ShortTimes::default(),
                days_of_week: all_weekdays(),
                ..Default::default()
            };

            let no_days_attention = AttentionSettings {
                enabled: true,
                times: ShortTimes::new(vec![naive_time(10, 0, 0)]),
                days_of_week: vec![],
                ..Default::default()
            };

            let attentions = vec![
                valid_attention.clone(),
                no_times_attention,
                no_days_attention,
            ];

            let result = calculate_next_attention(&attentions);

            if let Some((id, _)) = result {
                assert_eq!(id, valid_attention.id);
            }
        }

        /// Test: Multiple attentions, returns one of them (earliest from now)
        #[test]
        fn returns_earliest_attention() {
            let attention_morning = AttentionSettings {
                enabled: true,
                times: ShortTimes::new(vec![naive_time(9, 0, 0)]),
                days_of_week: all_weekdays(),
                ..Default::default()
            };

            let attention_afternoon = AttentionSettings {
                enabled: true,
                times: ShortTimes::new(vec![naive_time(14, 0, 0)]),
                days_of_week: all_weekdays(),
                ..Default::default()
            };

            let attention_evening = AttentionSettings {
                enabled: true,
                times: ShortTimes::new(vec![naive_time(18, 0, 0)]),
                days_of_week: all_weekdays(),
                ..Default::default()
            };

            let attentions = vec![
                attention_afternoon.clone(),
                attention_evening.clone(),
                attention_morning.clone(),
            ];

            let result = calculate_next_attention(&attentions);

            assert!(result.is_some(), "Should return an attention");
            let (_id, time) = result.unwrap();
            let hour = time.with_timezone(&Local).hour();
            // Should be one of the configured times
            assert!(
                hour == 9 || hour == 14 || hour == 18,
                "Returned hour ({hour}) should be one of the configured times (9, 14, or 18)"
            );
        }

        /// Test: Mixed enabled/disabled attentions, returns only enabled
        #[test]
        fn filters_disabled_attentions() {
            let attention_disabled = AttentionSettings {
                enabled: false,
                times: ShortTimes::new(vec![naive_time(9, 0, 0)]),
                days_of_week: all_weekdays(),
                ..Default::default()
            };

            let attention_enabled = AttentionSettings {
                enabled: true,
                times: ShortTimes::new(vec![naive_time(14, 0, 0)]),
                days_of_week: all_weekdays(),
                ..Default::default()
            };

            let attentions = vec![attention_disabled, attention_enabled.clone()];

            let result = calculate_next_attention(&attentions);

            let (_id, time) = result.unwrap();
            // Should return the enabled attention's time (14:00)
            assert_eq!(time.with_timezone(&Local).hour(), 14);
        }
    }

    // Additional tests for get_next_attention_time edge cases
    mod get_next_attention_time_edge_cases {
        use super::*;

        /// Test: Exact time match (current time equals attention time)
        #[test]
        fn returns_next_time_when_exact_match() {
            let attention = AttentionSettings {
                enabled: true,
                times: ShortTimes::new(vec![naive_time(10, 0, 0), naive_time(14, 0, 0)]),
                days_of_week: all_weekdays(),
                ..Default::default()
            };

            // Current time is exactly 10:00
            let now = get_test_local_datetime(2025, 9, 3, 10, 0, 0);

            let result = get_next_attention_time(&attention, now);

            let next_time = result.unwrap();
            // Should return 14:00 (next time after current)
            assert_eq!(next_time.with_timezone(&Local).hour(), 14);
        }

        /// Test: Last time of day passed, should go to next day
        #[test]
        fn wraps_to_next_day_when_all_times_passed() {
            let attention = AttentionSettings {
                enabled: true,
                times: ShortTimes::new(vec![naive_time(9, 0, 0), naive_time(14, 0, 0)]),
                days_of_week: all_weekdays(),
                ..Default::default()
            };

            // Current time is 20:00 (all times passed)
            let now = get_test_local_datetime(2025, 9, 3, 20, 0, 0);

            let result = get_next_attention_time(&attention, now);

            let next_time = result.unwrap();
            // Should return next day at 9:00
            assert_eq!(next_time.with_timezone(&Local).day(), 4);
            assert_eq!(next_time.with_timezone(&Local).hour(), 9);
        }

        /// Test: Weekend-only attention on weekday should skip to weekend
        #[test]
        fn skips_to_weekend_when_weekday_only() {
            let attention = AttentionSettings {
                enabled: true,
                times: ShortTimes::new(vec![naive_time(10, 0, 0)]),
                days_of_week: weekend_days(),
                ..Default::default()
            };

            // Wednesday 2025-09-03
            let now = get_test_local_datetime(2025, 9, 3, 9, 0, 0);

            let result = get_next_attention_time(&attention, now);

            let next_time = result.unwrap();
            // Should skip to Saturday (2025-09-06)
            assert_eq!(next_time.with_timezone(&Local).weekday(), Weekday::Sat);
        }

        /// Test: Single day attention
        #[test]
        fn handles_single_day_attention() {
            let attention = AttentionSettings {
                enabled: true,
                times: ShortTimes::new(vec![naive_time(10, 0, 0)]),
                days_of_week: vec![Weekday::Fri],
                ..Default::default()
            };

            // Monday morning
            let now = get_test_local_datetime(2025, 9, 1, 9, 0, 0);

            let result = get_next_attention_time(&attention, now);

            let next_time = result.unwrap();
            // Should skip to Friday (2025-09-05)
            assert_eq!(next_time.with_timezone(&Local).weekday(), Weekday::Fri);
            assert_eq!(next_time.with_timezone(&Local).hour(), 10);
        }

        /// Test: Multiple times at different parts of day
        #[test]
        fn handles_early_morning_and_late_night_times() {
            let attention = AttentionSettings {
                enabled: true,
                times: ShortTimes::new(vec![
                    naive_time(0, 30, 0), // 00:30
                    naive_time(6, 0, 0),  // 06:00
                    naive_time(23, 0, 0), // 23:00
                ]),
                days_of_week: all_weekdays(),
                ..Default::default()
            };

            // Test at 23:30 (after last time)
            let now = get_test_local_datetime(2025, 9, 3, 23, 30, 0);

            let result = get_next_attention_time(&attention, now);

            let next_time = result.unwrap();
            // Should return next day at 00:30
            assert_eq!(next_time.with_timezone(&Local).day(), 4);
            assert_eq!(next_time.with_timezone(&Local).hour(), 0);
            assert_eq!(next_time.with_timezone(&Local).minute(), 30);
        }

        /// Test: Empty `days_of_week` should return None
        #[test]
        fn returns_none_when_no_days_configured() {
            let attention = AttentionSettings {
                enabled: true,
                times: ShortTimes::new(vec![naive_time(10, 0, 0)]),
                days_of_week: vec![],
                ..Default::default()
            };

            let now = get_test_local_datetime(2025, 9, 3, 9, 0, 0);

            let result = get_next_attention_time(&attention, now);
            assert!(result.is_none());
        }

        /// Test: Wrap around week when no days match in current week
        #[test]
        fn wraps_around_week_when_no_matching_days() {
            let attention = AttentionSettings {
                enabled: true,
                times: ShortTimes::new(vec![naive_time(10, 0, 0)]),
                days_of_week: vec![Weekday::Mon],
                ..Default::default()
            };

            let now = get_test_local_datetime(2025, 9, 3, 9, 0, 0); // Wednesday

            let result = get_next_attention_time(&attention, now);

            let next_time = result.unwrap();
            // Should wrap to next Monday (2025-09-08)
            assert_eq!(next_time.with_timezone(&Local).weekday(), Weekday::Mon);
            assert_eq!(next_time.with_timezone(&Local).day(), 8);
        }

        /// Test: Wrap around week when exactly one week ahead
        #[test]
        fn wraps_around_week_when_exactly_one_week_ahead() {
            let attention = AttentionSettings {
                enabled: true,
                times: ShortTimes::new(vec![naive_time(10, 0, 0)]),
                days_of_week: vec![Weekday::Mon],
                ..Default::default()
            };

            let now = get_test_local_datetime(2025, 9, 1, 10, 0, 0); // Monday

            let result = get_next_attention_time(&attention, now);

            let next_time = result.unwrap();
            // Should wrap to next Monday (2025-09-08)
            assert_eq!(next_time.with_timezone(&Local).weekday(), Weekday::Mon);
            assert_eq!(next_time.with_timezone(&Local).day(), 8);
        }
    }
}
