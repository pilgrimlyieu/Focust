use chrono::Utc;
use chrono::{DateTime, Local, NaiveTime, Weekday};

use crate::{config::AppConfig, core::schedule::ScheduleSettings};

// This module is now primarily used internally by BreakScheduler
// The EventSource pattern is deprecated in favor of direct calculation

pub struct SchedulingContext<'a> {
    pub now_utc: DateTime<Utc>,
    pub now_local: DateTime<Local>,
    pub config: &'a AppConfig,
    pub mini_break_counter: u8,
    pub last_break_time: Option<DateTime<Utc>>,
}

/// Get the active schedule for a given time and day
#[must_use]
pub fn get_active_schedule(
    config: &AppConfig,
    now_time: NaiveTime,
    now_day: Weekday,
) -> Option<&ScheduleSettings> {
    config.schedules.iter().find(|s| {
        s.enabled && s.days_of_week.contains_day(now_day) && s.time_range.contains(&now_time)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::core::schedule::ScheduleSettings;
    use crate::scheduler::test_helpers::*;

    use chrono::Weekday;

    fn create_test_config() -> AppConfig {
        use crate::core::schedule::DaysOfWeek;

        AppConfig {
            schedules: vec![
                ScheduleSettings {
                    name: "Weekday Schedule".to_owned(),
                    enabled: true,
                    time_range: time_range(9, 0, 17, 0),
                    days_of_week: DaysOfWeek::workdays(),
                    ..Default::default()
                },
                ScheduleSettings {
                    name: "Weekend Schedule".to_owned(),
                    enabled: true,
                    time_range: time_range(10, 0, 14, 0),
                    days_of_week: DaysOfWeek::weekend(),
                    ..Default::default()
                },
                ScheduleSettings {
                    name: "Disabled Schedule".to_owned(),
                    enabled: false, // DISABLED
                    time_range: full_time_range(),
                    days_of_week: DaysOfWeek::MONDAY,
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn get_active_schedule_during_weekday() {
        let config = create_test_config();
        let now_time = naive_time(10, 30, 0);
        let now_day = Weekday::Mon;

        let active_schedule = get_active_schedule(&config, now_time, now_day);

        assert_eq!(active_schedule.unwrap().name, "Weekday Schedule");
    }

    #[test]
    fn get_active_schedule_outside_time_range() {
        let config = create_test_config();
        // Too late
        let now_time = naive_time(8, 0, 0);
        let now_day = Weekday::Tue;

        let active_schedule = get_active_schedule(&config, now_time, now_day);
        assert!(active_schedule.is_none());
    }

    #[test]
    fn get_active_schedule_on_weekend() {
        let config = create_test_config();
        // Weekend
        let now_time = naive_time(11, 0, 0);
        let now_day = Weekday::Sat;

        let active_schedule = get_active_schedule(&config, now_time, now_day);
        assert_eq!(active_schedule.unwrap().name, "Weekend Schedule");
    }

    #[test]
    fn get_active_schedule_ignores_disabled() {
        let config = create_test_config();
        // Match Weekday and Disabled schedule, but Disabled should be ignored
        let now_time = naive_time(10, 0, 0);
        let now_day = Weekday::Mon;

        let active_schedule = get_active_schedule(&config, now_time, now_day);
        assert_eq!(active_schedule.unwrap().name, "Weekday Schedule");
    }

    #[test]
    fn get_active_schedule_no_match_day() {
        let config = create_test_config();
        // Disabled
        let now_time = naive_time(21, 0, 0);
        let now_day = Weekday::Mon;

        let active_schedule = get_active_schedule(&config, now_time, now_day);
        assert!(active_schedule.is_none());
    }

    #[test]
    fn get_active_schedule_no_schedules_defined() {
        let config = AppConfig {
            schedules: vec![],
            ..Default::default()
        };
        let now_time = naive_time(10, 0, 0);
        let now_day = Weekday::Mon;

        let active_schedule = get_active_schedule(&config, now_time, now_day);
        assert!(active_schedule.is_none());
    }
}
