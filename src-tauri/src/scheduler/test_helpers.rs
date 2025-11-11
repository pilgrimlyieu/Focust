//! Test helpers and utilities for scheduler testing
//!
//! This module provides reusable test utilities to simplify writing tests
//! for the scheduler system. It includes configuration builders, time helpers,
//! and assertion utilities.
//!
//! # Design Goals
//!
//! - **Readability**: Tests should read like specifications
//! - **Reusability**: Avoid code duplication across tests
//! - **Flexibility**: Support diverse test scenarios
//! - **Stability**: Depend on public APIs, not internal implementation

use chrono::{DateTime, Datelike, Duration, Local, NaiveTime, TimeZone, Timelike, Utc, Weekday};
use tauri::Manager;

use crate::config::AppConfig;
use crate::core::schedule::{
    BaseBreakSettings, LongBreakSettings, MiniBreakSettings, ScheduleSettings,
};
use crate::core::time::TimeRange;

// ============================================================================
// Configuration Builders
// ============================================================================

/// Builder for creating test configurations with reasonable defaults
///
/// # Example
///
/// ```ignore
/// let config = TestConfigBuilder::new()
///     .mini_break_interval_s(60)
///     .mini_break_duration_s(20)
///     .time_range(time_range(9, 0, 17, 0))
///     .build();
/// ```
pub struct TestConfigBuilder {
    config: AppConfig,
}

#[allow(dead_code)]
impl TestConfigBuilder {
    /// Create a new builder with default test configuration
    ///
    /// Default values:
    /// - Mini break: 20min interval, 20s duration, enabled
    /// - Long break: after 4 mini breaks, 5min duration, enabled
    /// - Time range: 00:00-00:00 (all day)
    /// - Days: Monday-Sunday (all week)
    /// - Notification: 0s before (disabled)
    pub fn new() -> Self {
        let default_schedule = ScheduleSettings {
            name: "Test Schedule".to_string(),
            enabled: true,
            time_range: TimeRange {
                start: NaiveTime::MIN,
                end: NaiveTime::MIN,
            },
            days_of_week: all_weekdays(),
            notification_before_s: 0,
            mini_breaks: MiniBreakSettings {
                base: BaseBreakSettings {
                    enabled: true,
                    duration_s: 20,
                    postponed_s: 300,
                    max_postpone_count: 2,
                    ..BaseBreakSettings::default()
                },
                interval_s: 1200,
            },
            long_breaks: LongBreakSettings {
                base: BaseBreakSettings {
                    enabled: true,
                    duration_s: 300,
                    postponed_s: 300,
                    max_postpone_count: 2,
                    ..BaseBreakSettings::default()
                },
                after_mini_breaks: 4,
            },
        };

        Self {
            config: AppConfig {
                schedules: vec![default_schedule],
                attentions: vec![],
                ..Default::default()
            },
        }
    }

    /// Set mini break interval (seconds)
    pub fn mini_break_interval_s(mut self, seconds: u32) -> Self {
        self.config.schedules[0].mini_breaks.interval_s = seconds;
        self
    }

    /// Set mini break duration (seconds)
    pub fn mini_break_duration_s(mut self, seconds: u32) -> Self {
        self.config.schedules[0].mini_breaks.base.duration_s = seconds;
        self
    }

    /// Set long break duration (seconds)
    pub fn long_break_duration_s(mut self, seconds: u32) -> Self {
        self.config.schedules[0].long_breaks.base.duration_s = seconds;
        self
    }

    /// Set long break trigger condition (after N mini breaks)
    pub fn long_break_after_mini_breaks(mut self, count: u8) -> Self {
        self.config.schedules[0].long_breaks.after_mini_breaks = count;
        self
    }

    /// Set notification time before break (seconds)
    pub fn notification_before_s(mut self, seconds: u32) -> Self {
        self.config.schedules[0].notification_before_s = seconds;
        self
    }

    /// Set time range for the schedule
    pub fn time_range(mut self, range: TimeRange) -> Self {
        self.config.schedules[0].time_range = range;
        self
    }

    /// Set days of week for the schedule
    pub fn days_of_week(mut self, days: Vec<Weekday>) -> Self {
        self.config.schedules[0].days_of_week = days;
        self
    }

    /// Enable/disable mini breaks
    pub fn mini_breaks_enabled(mut self, enabled: bool) -> Self {
        self.config.schedules[0].mini_breaks.base.enabled = enabled;
        self
    }

    /// Enable/disable long breaks
    pub fn long_breaks_enabled(mut self, enabled: bool) -> Self {
        self.config.schedules[0].long_breaks.base.enabled = enabled;
        self
    }

    /// Enable/disable the entire schedule
    pub fn schedule_enabled(mut self, enabled: bool) -> Self {
        self.config.schedules[0].enabled = enabled;
        self
    }

    /// Add a second schedule (for multi-schedule tests)
    pub fn add_schedule(mut self, schedule: ScheduleSettings) -> Self {
        self.config.schedules.push(schedule);
        self
    }

    /// Set postpone settings
    pub fn postpone_settings(mut self, max_count: u8, postponed_s: u32) -> Self {
        self.config.schedules[0].mini_breaks.base.max_postpone_count = max_count;
        self.config.schedules[0].mini_breaks.base.postponed_s = postponed_s;
        self.config.schedules[0].long_breaks.base.max_postpone_count = max_count;
        self.config.schedules[0].long_breaks.base.postponed_s = postponed_s;
        self
    }

    /// Build the final configuration
    pub fn build(self) -> AppConfig {
        self.config
    }
}

impl Default for TestConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Time Helpers
// ============================================================================

/// Get all weekdays (Monday through Sunday)
pub fn all_weekdays() -> Vec<Weekday> {
    vec![
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ]
}

/// Get typical workdays (Monday through Friday)
pub fn workdays() -> Vec<Weekday> {
    vec![
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
    ]
}

/// Get weekend days (Saturday and Sunday)
pub fn weekend_days() -> Vec<Weekday> {
    vec![Weekday::Sat, Weekday::Sun]
}

/// Create a UTC datetime for testing
///
/// # Example
///
/// ```ignore
/// let dt = test_datetime(2025, 9, 3, 10, 30, 0);
/// ```
pub fn test_datetime(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, hour, min, sec)
        .single()
        .expect("Invalid test datetime")
}

/// Create a UTC datetime from local datetime for testing
pub fn test_datetime_with_local(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
) -> DateTime<Utc> {
    Local
        .with_ymd_and_hms(year, month, day, hour, min, sec)
        .single()
        .expect("Invalid test datetime")
        .with_timezone(&Utc)
}

/// Create a Local datetime for testing
pub fn test_local_datetime(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
) -> DateTime<Local> {
    Local
        .with_ymd_and_hms(year, month, day, hour, min, sec)
        .single()
        .expect("Invalid test datetime")
}

/// Create a duration from milliseconds
pub fn duration_ms(milliseconds: i64) -> Duration {
    Duration::milliseconds(milliseconds)
}

/// Create a duration from seconds
pub fn duration_s(seconds: i64) -> Duration {
    Duration::seconds(seconds)
}

/// Create a duration from minutes
pub fn duration_m(minutes: i64) -> Duration {
    Duration::minutes(minutes)
}

/// Create a duration from hours
pub fn duration_h(hours: i64) -> Duration {
    Duration::hours(hours)
}

/// Create a time helper
pub fn naive_time(hour: u32, min: u32, sec: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(hour, min, sec).unwrap()
}

/// Create a time range helper
pub fn time_range(start_hour: u32, start_min: u32, end_hour: u32, end_min: u32) -> TimeRange {
    TimeRange {
        start: naive_time(start_hour, start_min, 0),
        end: naive_time(end_hour, end_min, 0),
    }
}

/// Create a full-day time range
pub fn full_time_range() -> TimeRange {
    TimeRange {
        start: NaiveTime::MIN,
        end: NaiveTime::MIN,
    }
}

// ============================================================================
// Assertion Helpers
// ============================================================================

/// Assert that two datetimes are close (within tolerance)
///
/// Useful for comparing calculated times that may have small variations
/// due to execution time.
///
/// # Example
///
/// ```ignore
/// assert_time_near(actual, expected, Duration::seconds(1));
/// ```
pub fn assert_time_near(actual: DateTime<Utc>, expected: DateTime<Utc>, tolerance: Duration) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= tolerance,
        "Time difference too large: expected {expected}, actual {actual}, diff: {} seconds (tolerance: {} seconds)",
        diff.num_seconds(),
        tolerance.num_seconds()
    );
}

/// Assert that an option contains a value within a time tolerance
#[allow(unused)]
pub fn assert_time_option_near(
    actual: Option<DateTime<Utc>>,
    expected: DateTime<Utc>,
    tolerance: Duration,
) {
    match actual {
        Some(actual_time) => assert_time_near(actual_time, expected, tolerance),
        None => panic!("Expected Some({expected}), but got None"),
    }
}

/// Assert that a duration in seconds is close to expected
pub fn assert_duration_near(actual_seconds: i64, expected_seconds: i64, tolerance_seconds: i64) {
    let diff = (actual_seconds - expected_seconds).abs();
    assert!(
        diff <= tolerance_seconds,
        "Duration difference too large: expected {expected_seconds}s, actual {actual_seconds}s, diff: {diff}s (tolerance: {tolerance_seconds}s)"
    );
}

// ============================================================================
// Test Data Factories
// ============================================================================

/// Create a minimal schedule for testing specific scenarios
#[allow(unused)]
pub fn minimal_schedule(
    time_range: TimeRange,
    days: Vec<Weekday>,
    mini_interval_s: u32,
) -> ScheduleSettings {
    ScheduleSettings {
        name: "Minimal Schedule".to_string(),
        enabled: true,
        time_range,
        days_of_week: days,
        notification_before_s: 0,
        mini_breaks: MiniBreakSettings {
            base: BaseBreakSettings {
                enabled: true,
                duration_s: 20,
                max_postpone_count: 2,
                postponed_s: 300,
                ..BaseBreakSettings::default()
            },
            interval_s: mini_interval_s,
        },
        long_breaks: LongBreakSettings {
            base: BaseBreakSettings {
                enabled: false,
                ..BaseBreakSettings::default()
            },
            after_mini_breaks: 0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder_defaults() {
        let config = TestConfigBuilder::new().build();

        assert_eq!(config.schedules.len(), 1);
        assert!(config.schedules[0].enabled);
        assert_eq!(config.schedules[0].mini_breaks.interval_s, 1200);
        assert_eq!(config.schedules[0].mini_breaks.base.duration_s, 20);
    }

    #[test]
    fn test_config_builder_customization() {
        let config = TestConfigBuilder::new()
            .mini_break_interval_s(120)
            .mini_break_duration_s(30)
            .time_range(time_range(9, 0, 17, 0))
            .days_of_week(workdays())
            .build();

        assert_eq!(config.schedules[0].mini_breaks.interval_s, 120);
        assert_eq!(config.schedules[0].mini_breaks.base.duration_s, 30);
        assert_eq!(config.schedules[0].days_of_week.len(), 5);
    }

    #[test]
    fn test_time_helpers() {
        let dt = test_datetime(2025, 9, 3, 10, 30, 0);
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 9);
        assert_eq!(dt.day(), 3);
        assert_eq!(dt.hour(), 10);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_duration_helpers() {
        assert_eq!(duration_s(60).num_seconds(), 60);
        assert_eq!(duration_m(5).num_seconds(), 300);
        assert_eq!(duration_h(2).num_seconds(), 7200);
    }

    #[test]
    fn test_assert_time_near_passes() {
        let t1 = test_datetime(2025, 1, 15, 10, 0, 0);
        let t2 = test_datetime(2025, 1, 15, 10, 0, 1);
        assert_time_near(t1, t2, duration_s(2));
    }

    #[test]
    #[should_panic(expected = "Time difference too large")]
    fn test_assert_time_near_fails() {
        let t1 = test_datetime(2025, 9, 3, 10, 0, 0);
        let t2 = test_datetime(2025, 9, 3, 10, 0, 5);
        assert_time_near(t1, t2, duration_s(2));
    }

    #[test]
    fn test_weekday_helpers() {
        assert_eq!(all_weekdays().len(), 7);
        assert_eq!(workdays().len(), 5);
        assert_eq!(weekend_days().len(), 2);
    }
}

// ============================================================================
// State Machine Testing Helpers
// ============================================================================

#[cfg(test)]
pub mod state_machine {
    use super::*;
    use crate::config::SharedConfig;
    use crate::core::payload::PromptPayloadStore;
    use crate::core::suggestions::{SharedSuggestions, SuggestionsConfig};
    use crate::scheduler::break_scheduler::BreakScheduler;
    use crate::scheduler::event_emitter::TestEventEmitter;
    use crate::scheduler::models::{SchedulerEvent, SchedulerStatus};
    use crate::scheduler::shared_state::create_shared_state;

    use tauri::AppHandle;
    use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};
    use tokio::sync::watch;

    /// Create a test break scheduler with mock dependencies
    ///
    /// Returns (`scheduler`, `event_emitter`, `shutdown_tx`, `app_handle`)
    pub fn create_test_break_scheduler(
        config: AppConfig,
    ) -> (
        BreakScheduler<TestEventEmitter, MockRuntime>,
        TestEventEmitter,
        watch::Sender<()>,
        AppHandle<MockRuntime>,
    ) {
        let app = mock_builder()
            .plugin(tauri_plugin_notification::init())
            .build(mock_context(noop_assets()))
            .expect("Failed to create mock app");
        let app_handle = app.handle().clone();

        let event_emitter = TestEventEmitter::new();
        let (shutdown_tx, shutdown_rx) = watch::channel(());
        let shared_state = create_shared_state();

        // Install config in app state
        let shared_config = SharedConfig::from(config);
        app_handle.manage(shared_config);

        // Install suggestions config (required by window creation)
        let suggestions_config = SuggestionsConfig::default();
        let shared_suggestions = SharedSuggestions::new(suggestions_config);
        app_handle.manage(shared_suggestions);

        // Install prompt payload store (required by window creation)
        let prompt_payload_store = PromptPayloadStore::new();
        app_handle.manage(prompt_payload_store);

        let scheduler = BreakScheduler::new(
            app_handle.clone(),
            event_emitter.clone(),
            shutdown_rx,
            shared_state,
        );

        (scheduler, event_emitter, shutdown_tx, app_handle)
    }

    /// Advance time and give scheduler a chance to process
    ///
    /// This function:
    /// 1. Advances tokio's mock time by the given duration
    /// 2. Yields multiple times to let tasks process
    /// 3. Sleeps briefly to ensure all async operations complete
    pub async fn advance_time_and_yield(duration: chrono::Duration) {
        let std_duration = duration.to_std().unwrap_or(std::time::Duration::ZERO);
        tokio::time::advance(std_duration).await;
        // Yield multiple times to ensure scheduler processes
        for _ in 0..5 {
            tokio::task::yield_now().await;
        }
        // Extra small sleep to ensure all async tasks complete
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    /// Assert that an event was emitted with the given name
    ///
    /// Returns the event payload for further assertions
    ///
    /// # Panics
    /// Panics if the event was not found
    #[allow(dead_code)]
    pub fn assert_event_emitted(emitter: &TestEventEmitter, event_name: &str) -> serde_json::Value {
        emitter
            .get_events_by_name(event_name)
            .into_iter()
            .last()
            .unwrap_or_else(|| {
                panic!(
                    "Event '{event_name}' not found. Available events: {:?}",
                    emitter
                        .get_events()
                        .iter()
                        .map(|(n, _)| n)
                        .collect::<Vec<_>>()
                )
            })
    }

    /// Assert that an event was NOT emitted
    ///
    /// # Panics
    /// Panics if the event was found
    #[allow(dead_code)]
    pub fn assert_event_not_emitted(emitter: &TestEventEmitter, event_name: &str) {
        assert!(
            !emitter.has_event(event_name),
            "Event '{event_name}' should not have been emitted"
        );
    }

    /// Clear all recorded events
    #[allow(dead_code)]
    pub fn clear_events(emitter: &TestEventEmitter) {
        emitter.clear();
    }

    /// Extract `SchedulerStatus` from event payload
    ///
    /// # Panics
    /// Panics if the payload cannot be parsed as `SchedulerStatus`
    pub fn extract_scheduler_status(payload: &serde_json::Value) -> SchedulerStatus {
        serde_json::from_value(payload.clone())
            .unwrap_or_else(|e| panic!("Failed to parse SchedulerStatus: {e}"))
    }

    /// Wait for a specific event to be emitted
    ///
    /// Polls the event emitter until the event appears with a timeout.
    ///
    /// # Returns
    /// The event payload if found, None if timeout
    #[allow(dead_code)]
    pub async fn wait_for_event(
        emitter: &TestEventEmitter,
        event_name: &str,
        timeout_ms: u64,
    ) -> Option<serde_json::Value> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        while start.elapsed() < timeout {
            if emitter.has_event(event_name) {
                return Some(assert_event_emitted(emitter, event_name));
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        None
    }

    /// Assert that the next event in scheduler status matches expectations
    ///
    /// # Arguments
    /// * `status` - The `SchedulerStatus` to check
    /// * `expected_kind` - Expected event type (or None if should be idle)
    /// * `expected_seconds_tolerance` - Tolerance for time comparison (seconds)
    #[allow(dead_code)]
    pub fn assert_next_event_matches(
        status: &SchedulerStatus,
        expected_kind: Option<SchedulerEvent>,
        expected_seconds_tolerance: i32,
    ) {
        match (status.next_event.as_ref(), expected_kind) {
            (None, None) => {
                // Both None - OK
            }
            (Some(event_info), Some(expected)) => {
                assert_eq!(
                    event_info.kind, expected,
                    "Event kind mismatch: expected {expected}, got {}",
                    event_info.kind
                );
                // Optionally check timing if tolerance is positive
                if expected_seconds_tolerance > 0 {
                    assert!(
                        event_info.seconds_until.abs() <= expected_seconds_tolerance,
                        "Event timing out of tolerance: {} seconds (tolerance: {expected_seconds_tolerance})",
                        event_info.seconds_until,
                    );
                }
            }
            (None, Some(expected)) => {
                panic!("Expected event {expected}, but status has no next_event");
            }
            (Some(event_info), None) => {
                panic!("Expected no next_event, but status has {}", event_info.kind);
            }
        }
    }

    /// Get the most recent scheduler-status event
    ///
    /// # Returns
    /// The parsed `SchedulerStatus`, or panics if not found
    pub fn get_latest_status(emitter: &TestEventEmitter) -> SchedulerStatus {
        let events = emitter.get_events_by_name("scheduler-status");
        assert!(!events.is_empty(), "No scheduler-status events found");
        extract_scheduler_status(events.last().unwrap())
    }

    /// Assert that scheduler is in a specific state (via state string)
    #[allow(dead_code)]
    pub fn assert_scheduler_state_contains(state_str: &str, expected_substr: &str) {
        assert!(
            state_str.contains(expected_substr),
            "Expected state to contain '{expected_substr}', but got: {state_str}"
        );
    }
}

// ============================================================================
// Manager Testing Helpers
// ============================================================================

#[cfg(test)]
pub mod manager {
    use super::*;
    use crate::config::SharedConfig;
    use crate::core::payload::PromptPayloadStore;
    use crate::core::suggestions::{SharedSuggestions, SuggestionsConfig};
    use crate::scheduler::event_emitter::TestEventEmitter;
    use crate::scheduler::models::Command;
    use crate::scheduler::shared_state::SharedState;

    use tauri::AppHandle;
    use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};
    use tokio::sync::mpsc;
    use tokio::sync::watch;

    /// Test environment for `SchedulerManager` integration tests
    ///
    /// Contains all necessary components to test the full scheduler system
    /// including monitors, event routing, and state management.
    #[allow(dead_code)] // Some fields used conditionally in tests
    pub struct ManagerTestEnv {
        pub app_handle: AppHandle<MockRuntime>,
        pub event_emitter: TestEventEmitter,
        pub cmd_tx: mpsc::Sender<Command>,
        pub shutdown_tx: watch::Sender<()>,
        pub shared_state: SharedState,
    }

    /// Create a complete test environment for `SchedulerManager`
    ///
    /// This sets up:
    /// - Mock Tauri app with all required plugins and state
    /// - `TestEventEmitter` for capturing events
    /// - Command channel for sending commands
    /// - Shutdown channel for graceful shutdown
    /// - `SharedState` for pause reason tracking
    ///
    /// Returns the environment struct with all components
    pub fn create_manager_test_env(config: AppConfig) -> ManagerTestEnv {
        let app = mock_builder()
            .plugin(tauri_plugin_notification::init())
            .build(mock_context(noop_assets()))
            .expect("Failed to create mock app");
        let app_handle = app.handle().clone();

        // Install config
        let shared_config = SharedConfig::from(config);
        app_handle.manage(shared_config);

        // Install suggestions
        let suggestions_config = SuggestionsConfig::default();
        let shared_suggestions = SharedSuggestions::new(suggestions_config);
        app_handle.manage(shared_suggestions);

        // Install prompt payload store
        let prompt_payload_store = PromptPayloadStore::new();
        app_handle.manage(prompt_payload_store);

        // Create channels
        let (cmd_tx, _cmd_rx) = mpsc::channel(32);
        let (shutdown_tx, _shutdown_rx) = watch::channel(());
        let shared_state = crate::scheduler::shared_state::create_shared_state();

        // Note: We don't spawn SchedulerManager here - each test will do that
        // This gives tests full control over the lifecycle

        let event_emitter = TestEventEmitter::new();

        ManagerTestEnv {
            app_handle,
            event_emitter,
            cmd_tx,
            shutdown_tx,
            shared_state,
        }
    }

    /// Spawn a minimal scheduler manager for testing
    ///
    /// This spawns break scheduler and attention timer with command broadcasting,
    /// but does NOT spawn monitors (tests will add monitors manually if needed).
    pub async fn spawn_test_manager(env: &ManagerTestEnv, cmd_rx: mpsc::Receiver<Command>) {
        use crate::scheduler::attention_timer::AttentionTimer;
        use crate::scheduler::break_scheduler::BreakScheduler;
        use crate::scheduler::manager::broadcast_commands;

        let (break_cmd_tx, break_cmd_rx) = mpsc::channel::<Command>(32);
        let (attention_cmd_tx, attention_cmd_rx) = mpsc::channel::<Command>(32);

        // Spawn break scheduler (using TestEventEmitter for tests)
        let break_scheduler_handle = env.app_handle.clone();
        let break_event_emitter = env.event_emitter.clone();
        let break_shutdown_rx = env.shutdown_tx.subscribe();
        let break_shared_state = env.shared_state.clone();
        tokio::spawn(async move {
            let mut scheduler = BreakScheduler::new(
                break_scheduler_handle,
                break_event_emitter,
                break_shutdown_rx,
                break_shared_state,
            );
            scheduler.run(break_cmd_rx).await;
        });

        // Spawn attention timer (using TestEventEmitter for tests)
        let attention_timer_handle = env.app_handle.clone();
        let attention_event_emitter = env.event_emitter.clone();
        let attention_shutdown_rx = env.shutdown_tx.subscribe();
        let attention_shared_state = env.shared_state.clone();
        tokio::spawn(async move {
            let mut timer = AttentionTimer::new(
                attention_timer_handle,
                attention_event_emitter,
                attention_shutdown_rx,
                attention_shared_state,
            );
            timer.run(attention_cmd_rx).await;
        });

        // Spawn command broadcaster (simplified version without monitors)
        let router_shutdown_rx = env.shutdown_tx.subscribe();
        let router_shared_state = env.shared_state.clone();
        let router_app_handle = env.app_handle.clone();
        tokio::spawn(async move {
            broadcast_commands(
                cmd_rx,
                break_cmd_tx,
                attention_cmd_tx,
                router_shutdown_rx,
                router_shared_state,
                router_app_handle,
            )
            .await;
        });
    }
}
