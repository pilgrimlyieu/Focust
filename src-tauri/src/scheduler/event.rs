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
            let break_time = context.last_break_time.unwrap_or_else(|| context.now_utc) + interval;

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

fn get_active_schedule<'a>(
    config: &'a AppConfig,
    now_time: NaiveTime,
    now_day: Weekday,
) -> Option<&'a ScheduleSettings> {
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
                        "Ambiguous local time encountered for {time} on {date}. Using the first one."
                    );
                    Some(dt1)
                }
                LocalResult::None => {
                    log::error!("No valid local time found for {time} on {date}.");
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
        log::warn!(
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
