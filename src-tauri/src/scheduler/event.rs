use chrono::offset::LocalResult;
use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveTime, Weekday};
use chrono::{Duration, Utc};

use super::models::*;
use crate::core::schedule::AttentionSettings;
use crate::{config::AppConfig, core::schedule::ScheduleSettings};

pub struct SchedulingContext<'a> {
    pub now_utc: DateTime<Utc>,
    pub now_local: DateTime<Local>,
    pub config: &'a AppConfig,
    pub mini_break_counter: u8,
    pub last_break_time: Option<DateTime<Utc>>,
}

pub trait EventSource: Send + Sync {
    /// Based on the current context, calculate and return a list of potential future events.
    fn upcoming_events(&self, context: &SchedulingContext) -> Vec<ScheduledEvent>;
}

pub struct BreakEventSource;

impl EventSource for BreakEventSource {
    fn upcoming_events(&self, context: &SchedulingContext) -> Vec<ScheduledEvent> {
        let mut potential_events = Vec::new();

        let active_schedule = match get_active_schedule(
            context.config,
            context.now_local.time(),
            context.now_local.weekday(),
        ) {
            Some(s) => s,
            None => return potential_events, // No active schedule, no break events
        };

        let is_long_break_due = active_schedule.long_breaks.base.enabled
            && context.mini_break_counter >= active_schedule.long_breaks.after_mini_breaks;

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

        if break_settings.enabled {
            let interval = Duration::seconds(active_schedule.mini_breaks.interval_s as i64);
            let break_time = context.last_break_time.unwrap_or(context.now_utc) + interval;

            potential_events.push(ScheduledEvent {
                time: break_time,
                kind: break_kind,
            });

            if active_schedule.has_notification() {
                let notification_time =
                    break_time - Duration::seconds(active_schedule.notification_before_s as i64);
                let notification_kind = match break_kind {
                    EventKind::LongBreak(id) => NotificationKind::LongBreak(id),
                    EventKind::MiniBreak(id) => NotificationKind::MiniBreak(id),
                    _ => unreachable!("Notifition kind can only for break types."),
                };
                potential_events.push(ScheduledEvent {
                    time: notification_time,
                    kind: EventKind::Notification(notification_kind),
                });
            }
        }

        potential_events
    }
}

fn get_active_schedule(
    config: &AppConfig,
    now_time: NaiveTime,
    now_day: Weekday,
) -> Option<&ScheduleSettings> {
    config.schedules.iter().find(|s| {
        s.enabled && s.days_of_week.contains(&now_day) && s.time_range.contains(&now_time)
    })
}

pub struct AttentionEventSource;

impl AttentionEventSource {
    fn get_next_attention_time(
        &self,
        attention: &AttentionSettings,
        now: DateTime<Local>,
    ) -> Option<DateTime<Utc>> {
        let now_date = now.date_naive();
        let now_time = now.time();
        let to_utc = |dt_local: DateTime<Local>| -> Option<DateTime<Utc>> {
            tracing::debug!(
                "Found potential attention time: {} (local)",
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

        if attention.days_of_week.contains(&now.weekday())
            && let Some(next_time_today) = attention.times.earliest_after(&now_time)
            && let Some(dt_local) = build_datetime(now_date, next_time_today)
        {
            return to_utc(dt_local);
        }

        for i in 1..=7 {
            let next_date = now_date + chrono::Duration::days(i);
            if attention.days_of_week.contains(&next_date.weekday())
                && let Some(first_time) = attention.times.first()
                && let Some(dt_local) = build_datetime(next_date, first_time)
            {
                return to_utc(dt_local);
            }
        }
        tracing::warn!(
            "No scheduled time found for attention '{}' in the next 7 days.",
            attention.name
        );
        None
    }
}

impl EventSource for AttentionEventSource {
    fn upcoming_events(&self, context: &SchedulingContext) -> Vec<ScheduledEvent> {
        let mut potential_events = Vec::new();

        for attention in &context.config.attentions {
            if attention.enabled
                && let Some(next_attention_time) =
                    self.get_next_attention_time(attention, context.now_local)
            {
                potential_events.push(ScheduledEvent {
                    time: next_attention_time,
                    kind: EventKind::Attention(attention.id),
                });
            }
        }

        potential_events
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveTime, TimeZone, Weekday};

    use super::*;
    use crate::core::schedule::ScheduleSettings;
    use crate::core::time::ShortTimes;
    use crate::{config::AppConfig, core::time::TimeRange};

    fn create_test_config() -> AppConfig {
        AppConfig {
            schedules: vec![
                ScheduleSettings {
                    name: "Weekday Schedule".to_string(),
                    enabled: true,
                    time_range: TimeRange {
                        // 09:00 - 17:00
                        start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                        end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                    },
                    days_of_week: vec![
                        Weekday::Mon,
                        Weekday::Tue,
                        Weekday::Wed,
                        Weekday::Thu,
                        Weekday::Fri,
                    ],
                    ..Default::default()
                },
                ScheduleSettings {
                    name: "Weekend Schedule".to_string(),
                    enabled: true,
                    time_range: TimeRange {
                        // 10:00 - 14:00
                        start: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                        end: NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
                    },
                    days_of_week: vec![Weekday::Sat, Weekday::Sun],
                    ..Default::default()
                },
                ScheduleSettings {
                    name: "Disabled Schedule".to_string(),
                    enabled: false, // DISABLED
                    time_range: TimeRange {
                        // 00:00 - 23:59
                        start: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
                        end: NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
                    },
                    days_of_week: vec![Weekday::Mon],
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn test_get_active_schedule_during_weekday() {
        let config = create_test_config();
        let now_time = NaiveTime::from_hms_opt(10, 30, 0).unwrap();
        let now_day = Weekday::Mon;

        let active_schedule = get_active_schedule(&config, now_time, now_day);

        assert!(active_schedule.is_some());
        assert_eq!(active_schedule.unwrap().name, "Weekday Schedule");
    }

    #[test]
    fn test_get_active_schedule_outside_time_range() {
        let config = create_test_config();
        // Too late
        let now_time = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
        let now_day = Weekday::Tue;

        let active_schedule = get_active_schedule(&config, now_time, now_day);
        assert!(active_schedule.is_none());
    }

    #[test]
    fn test_get_active_schedule_on_weekend() {
        let config = create_test_config();
        // Weekend
        let now_time = NaiveTime::from_hms_opt(11, 0, 0).unwrap();
        let now_day = Weekday::Sat;

        let active_schedule = get_active_schedule(&config, now_time, now_day);
        assert!(active_schedule.is_some());
        assert_eq!(active_schedule.unwrap().name, "Weekend Schedule");
    }

    #[test]
    fn test_get_active_schedule_ignores_disabled() {
        let config = create_test_config();
        // Match Weekday and Disabled schedule, but Disabled should be ignored
        let now_time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let now_day = Weekday::Mon;

        let active_schedule = get_active_schedule(&config, now_time, now_day);
        assert!(active_schedule.is_some());
        assert_eq!(active_schedule.unwrap().name, "Weekday Schedule");
    }

    #[test]
    fn test_get_active_schedule_no_match_day() {
        let config = create_test_config();
        // Disabled
        let now_time = NaiveTime::from_hms_opt(21, 0, 0).unwrap();
        let now_day = Weekday::Mon;

        let active_schedule = get_active_schedule(&config, now_time, now_day);
        assert!(active_schedule.is_none());
    }

    #[test]
    fn test_get_active_schedule_no_schedules_defined() {
        let config = AppConfig {
            schedules: vec![],
            ..Default::default()
        };
        let now_time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let now_day = Weekday::Mon;

        let active_schedule = get_active_schedule(&config, now_time, now_day);
        assert!(active_schedule.is_none());
    }

    #[test]
    fn test_get_next_attention_time_today() {
        let source = AttentionEventSource;
        let attention = AttentionSettings {
            enabled: true,
            days_of_week: vec![Weekday::Mon], // Mon
            times: ShortTimes::new(vec![
                NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            ]),
            ..Default::default()
        };

        // 2025-09-01 Mon
        let now = Local.with_ymd_and_hms(2025, 9, 1, 10, 0, 0).unwrap();
        let next_time = source.get_next_attention_time(&attention, now);

        assert!(next_time.is_some());
        let expected_time = Local
            .with_ymd_and_hms(2025, 9, 1, 14, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(next_time.unwrap(), expected_time);
    }

    #[test]
    fn test_get_next_attention_time_tomorrow() {
        let source = AttentionEventSource;
        let attention = AttentionSettings {
            enabled: true,
            days_of_week: vec![Weekday::Tue], // Tue
            times: ShortTimes::new(vec![NaiveTime::from_hms_opt(9, 30, 0).unwrap()]),
            ..Default::default()
        };

        // 2025-09-01 Mon
        let now = Local.with_ymd_and_hms(2025, 9, 1, 11, 0, 0).unwrap();
        let next_time = source.get_next_attention_time(&attention, now);

        assert!(next_time.is_some());
        let expected_time = Local
            .with_ymd_and_hms(2025, 9, 2, 9, 30, 0)
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(next_time.unwrap(), expected_time);
    }

    #[test]
    fn test_get_next_attention_time_wraps_to_next_week() {
        let source = AttentionEventSource;
        let attention = AttentionSettings {
            enabled: true,
            days_of_week: vec![Weekday::Mon], // Mon
            times: ShortTimes::new(vec![NaiveTime::from_hms_opt(10, 0, 0).unwrap()]),
            ..Default::default()
        };

        // 2025-09-05 Fri
        let now = Local.with_ymd_and_hms(2025, 9, 5, 12, 0, 0).unwrap();
        let next_time = source.get_next_attention_time(&attention, now);

        assert!(next_time.is_some());
        let expected_time = Local
            .with_ymd_and_hms(2025, 9, 8, 10, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(next_time.unwrap(), expected_time);
    }

    #[test]
    fn test_get_next_attention_time_no_upcoming_event() {
        let source = AttentionEventSource;
        let attention = AttentionSettings {
            enabled: true,
            days_of_week: vec![], // Empty days
            times: ShortTimes::new(vec![NaiveTime::from_hms_opt(10, 0, 0).unwrap()]),
            ..Default::default()
        };

        let now = Local.with_ymd_and_hms(2025, 9, 1, 12, 0, 0).unwrap();
        let next_time = source.get_next_attention_time(&attention, now);

        assert!(next_time.is_none());
    }
}
