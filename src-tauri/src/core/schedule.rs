use std::{fmt::Display, sync::atomic::AtomicUsize};

use crate::core::{
    audio::AudioSettings,
    ideas::IdeasSettings,
    theme::ThemeSettings,
    time::{ShortTimes, TimeRange},
};
use chrono::Weekday;
use serde::{Deserialize, Serialize};

static NEXT_SCHEDULE_ID: AtomicUsize = AtomicUsize::new(0);
static NEXT_ATTENTION_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BreakId(usize);

impl Display for BreakId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "S{}", self.0)
    }
}

impl BreakId {
    pub fn new() -> Self {
        BreakId(NEXT_SCHEDULE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseBreakSettings {
    pub id: BreakId,                 // Unique identifier for the break settings
    pub enabled: bool,               // If the break is enabled
    pub theme: ThemeSettings,        // Theme settings for the break
    pub audio: AudioSettings,        // Audio settings for the break
    pub fullscreen: bool,            // If the break should be shown in fullscreen
    pub ideas_source: IdeasSettings, // Ideas settings for the break
    pub duration_s: u64,             // Duration of the break in seconds
    pub postponed_s: u64,            // Postponed time in seconds
    pub strict_mode: bool,           // If the break should be strictly followed
}

impl Default for BaseBreakSettings {
    fn default() -> Self {
        BaseBreakSettings {
            id: BreakId::new(),
            enabled: true,
            theme: ThemeSettings::default(),
            audio: AudioSettings::default(),
            fullscreen: false,
            ideas_source: IdeasSettings::default(),
            duration_s: 20,   // Default to 20 seconds
            postponed_s: 300, // Default to 5 minutes
            strict_mode: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MiniBreakSettings {
    #[serde(flatten)]
    pub base: BaseBreakSettings,
    pub interval_s: u64, // Interval between mini breaks in seconds
}

impl Default for MiniBreakSettings {
    fn default() -> Self {
        MiniBreakSettings {
            base: BaseBreakSettings::default(),
            interval_s: 1200, // Default to 20 minutes
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LongBreakSettings {
    #[serde(flatten)]
    pub base: BaseBreakSettings,
    pub after_mini_breaks: u8, // Number of mini breaks after which to trigger a long break
}

impl Default for LongBreakSettings {
    fn default() -> Self {
        LongBreakSettings {
            base: BaseBreakSettings {
                duration_s: 300, // Default to 5 minutes
                ..BaseBreakSettings::default()
            },
            after_mini_breaks: 4, // Default to after 4 mini breaks
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScheduleSettings {
    pub name: String,               // Name of the schedule
    pub enabled: bool,              // If the schedule is enabled
    pub time_range: TimeRange,      // Time range for the schedule
    pub days_of_week: Vec<Weekday>, // Days of the week when the schedule is active
    pub notification_before_s: u64, // Notification time in seconds before breaks

    pub mini_breaks: MiniBreakSettings, // Settings for mini breaks
    pub long_breaks: LongBreakSettings, // Settings for long breaks
}

impl Default for ScheduleSettings {
    fn default() -> Self {
        ScheduleSettings {
            name: "Default Schedule".to_string(),
            enabled: true,
            time_range: TimeRange::default(),
            days_of_week: vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
                Weekday::Sat,
                Weekday::Sun,
            ], // Active every day
            notification_before_s: 10, // Default to 10 seconds before breaks
            mini_breaks: MiniBreakSettings::default(),
            long_breaks: LongBreakSettings::default(),
        }
    }
}

impl ScheduleSettings {
    pub fn has_notification(&self) -> bool {
        self.notification_before_s > 0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AttentionId(usize);

impl Display for AttentionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "A{}", self.0)
    }
}

impl AttentionId {
    pub fn new() -> Self {
        AttentionId(NEXT_ATTENTION_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttentionSettings {
    pub id: AttentionId,            // Unique identifier for the attention reminder
    pub name: String,               // Name of the attention reminder
    pub enabled: bool,              // If the attention reminder is enabled
    pub theme: ThemeSettings,       // Theme settings for the attention reminder
    pub times: ShortTimes,          // Times when the attention reminder should trigger
    pub days_of_week: Vec<Weekday>, // Days of the week when the attention reminder is active
    pub title: String,              // Title of the attention reminder
    pub message: String,            // Message of the attention reminder
    pub duration_s: u64,            // Duration of the attention reminder in seconds
}

impl Default for AttentionSettings {
    fn default() -> Self {
        AttentionSettings {
            id: AttentionId::new(),
            name: "Default Attention".to_string(),
            enabled: true,
            theme: ThemeSettings::default(),
            times: ShortTimes::default(),
            days_of_week: vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
                Weekday::Sat,
                Weekday::Sun,
            ],
            title: "Attention Reminder".to_string(),
            message: "This is an attention reminder.".to_string(),
            duration_s: 10, // Default to 10 seconds
        }
    }
}
