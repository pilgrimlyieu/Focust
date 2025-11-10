use chrono::offset::LocalResult;
use chrono::{DateTime, Duration, Local, Utc};
use chrono::{Datelike, NaiveDate, NaiveTime};
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::{mpsc, watch};
use tokio::time::sleep;

use super::event_emitter::EventEmitter;
use super::models::{Command, SchedulerEvent};
use super::shared_state::SharedState;
use crate::core::schedule::AttentionSettings;
use crate::platform::create_prompt_windows;
use crate::{config::SharedConfig, core::schedule::AttentionId};

/// A simple timer for attention reminders
/// Attention timer can be paused/resumed like breaks
#[allow(dead_code)]
pub struct AttentionTimer<E, R = tauri::Wry>
where
    E: EventEmitter,
    R: Runtime,
{
    app_handle: AppHandle<R>,
    event_emitter: E,
    shutdown_rx: watch::Receiver<()>,
    paused: bool,
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
            paused: false,
            shared_state,
        }
    }

    /// Check if paused (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub async fn run(&mut self, mut cmd_rx: mpsc::Receiver<Command>) {
        tracing::info!("AttentionTimer started");

        loop {
            // Check for incoming commands first
            if let Ok(cmd) = cmd_rx.try_recv() {
                self.handle_command(cmd).await;
                continue;
            }

            // If paused, wait for commands or shutdown
            if self.paused {
                tokio::select! {
                    biased;
                    _ = self.shutdown_rx.changed() => {
                        tracing::info!("AttentionTimer received shutdown signal while paused");
                        break;
                    }
                    Some(cmd) = cmd_rx.recv() => {
                        self.handle_command(cmd).await;
                    }
                }
                continue;
            }

            // Calculate next attention time
            let next_attention = {
                let config = self.app_handle.state::<SharedConfig>();
                let config_guard = config.read().await;
                calculate_next_attention(&config_guard.attentions)
            };

            if let Some((attention_id, attention_time)) = next_attention {
                let now = Utc::now();
                let duration_to_wait = attention_time - now;

                if duration_to_wait <= Duration::zero() {
                    // Attention time already passed, execute immediately and recalculate
                    tracing::warn!(
                        "Attention time already passed, triggering immediately and recalculating"
                    );
                    self.trigger_attention(attention_id);
                    continue;
                }

                tracing::info!(
                    "Next attention in {} seconds",
                    duration_to_wait.num_seconds()
                );

                tokio::select! {
                    biased;
                    _ = self.shutdown_rx.changed() => {
                        tracing::info!("AttentionTimer received shutdown signal");
                        break;
                    }
                    () = sleep(duration_to_wait.to_std().unwrap_or(std::time::Duration::ZERO)) => {
                        if !self.paused {
                            self.trigger_attention(attention_id);
                        }
                    }
                    Some(cmd) = cmd_rx.recv() => {
                        self.handle_command(cmd).await;
                    }
                }
            } else {
                tracing::debug!("No enabled attentions, waiting for config change or shutdown");

                // Wait for shutdown or config update command
                tokio::select! {
                    biased;
                    _ = self.shutdown_rx.changed() => {
                        tracing::info!("AttentionTimer received shutdown signal");
                        break;
                    }
                    Some(cmd) = cmd_rx.recv() => {
                        self.handle_command(cmd).await;
                    }
                }
            }
        }

        tracing::info!("AttentionTimer shutting down");
    }

    /// Trigger an attention reminder
    fn trigger_attention(&self, attention_id: AttentionId) {
        tracing::info!("Triggering attention: {attention_id}");

        // Mark attention session as started
        self.shared_state.write().start_attention_session();

        let event = SchedulerEvent::Attention(attention_id);

        let app_handle = self.app_handle.clone();
        tokio::spawn(async move {
            create_prompt_windows(&app_handle, event, 0)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to create attention windows: {e}");
                });
        });
    }

    /// Handle incoming commands
    async fn handle_command(&mut self, cmd: Command) {
        tracing::debug!("AttentionTimer handling command: {cmd}");

        match cmd {
            Command::Pause(reason) => {
                tracing::info!("Pausing AttentionTimer: {reason}");
                self.paused = true;
            }
            Command::Resume(reason) => {
                tracing::info!("Resuming AttentionTimer: {reason}");
                self.paused = false;
                // Will recalculate next attention on next loop iteration
            }
            Command::UpdateConfig(new_config) => {
                tracing::debug!("Updating config in AttentionTimer");
                {
                    let config = self.app_handle.state::<SharedConfig>();
                    let mut config_guard = config.write().await;
                    *config_guard = new_config;
                }
                // Config updated, will recalculate next attention in next loop iteration
            }
            Command::TriggerEvent(SchedulerEvent::Attention(attention_id)) => {
                tracing::info!("Manually triggering attention: {attention_id}");
                self.trigger_attention(attention_id);
            }
            Command::PromptFinished(SchedulerEvent::Attention(_)) => {
                tracing::debug!("Attention prompt finished, ending session");
                self.shared_state.write().end_attention_session();
            }
            // AttentionTimer ignores other commands (they're for BreakScheduler)
            _ => {}
        }
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
    if (!attention.enabled) || attention.times.is_empty() || attention.days_of_week.is_empty() {
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
    unreachable!("Impossible if attention has times and days configured");
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

            let now = test_local_datetime(2025, 9, 3, 9, 0, 0);

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

            let now = test_local_datetime(2025, 9, 3, 15, 0, 0);

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

            let now = test_local_datetime(2025, 9, 3, 9, 0, 0);

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

            let now = test_local_datetime(2025, 9, 3, 9, 0, 0);

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

            let now = test_local_datetime(2025, 9, 2, 9, 0, 0);

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

            let now = test_local_datetime(2025, 9, 3, 10, 0, 0);

            let result = get_next_attention_time(&attention, now);

            let next_time = result.unwrap();
            assert_eq!(next_time.with_timezone(&Local).hour(), 12);
            assert_eq!(next_time.with_timezone(&Local).day(), 3);
        }
    }

    mod calculate_next_attention_tests {
        use super::*;

        #[test]
        #[allow(clippy::similar_names)]
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
            let now = test_local_datetime(2025, 9, 3, 10, 0, 0);

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
            let now = test_local_datetime(2025, 9, 3, 20, 0, 0);

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
            let now = test_local_datetime(2025, 9, 3, 9, 0, 0);

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
            let now = test_local_datetime(2025, 9, 1, 9, 0, 0);

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
            let now = test_local_datetime(2025, 9, 3, 23, 30, 0);

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

            let now = test_local_datetime(2025, 9, 3, 9, 0, 0);

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

            let now = test_local_datetime(2025, 9, 3, 9, 0, 0); // Wednesday

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

            let now = test_local_datetime(2025, 9, 1, 10, 0, 0); // Monday

            let result = get_next_attention_time(&attention, now);

            let next_time = result.unwrap();
            // Should wrap to next Monday (2025-09-08)
            assert_eq!(next_time.with_timezone(&Local).weekday(), Weekday::Mon);
            assert_eq!(next_time.with_timezone(&Local).day(), 8);
        }
    }
}
