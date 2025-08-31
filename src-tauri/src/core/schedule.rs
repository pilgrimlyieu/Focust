use std::{fmt::Display, sync::atomic::AtomicUsize};

use crate::core::{
    audio::AudioSettings,
    ideas::IdeasSettings,
    theme::ThemeSettings,
    time::{ShortTimes, TimeRange},
};
use chrono::Weekday;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

static NEXT_SCHEDULE_ID: AtomicUsize = AtomicUsize::new(0);
static NEXT_ATTENTION_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TS)]
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

impl Default for BreakId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(rename_all = "camelCase")]
pub struct BaseBreakSettings {
    /// Unique identifier for the break settings
    pub id: BreakId,
    /// If the break is enabled
    pub enabled: bool,
    /// Theme settings for the break
    pub theme: ThemeSettings,
    /// Audio settings for the break
    pub audio: AudioSettings,
    /// If the break should be shown in fullscreen
    pub fullscreen: bool,
    /// Ideas settings for the break
    pub ideas_source: IdeasSettings,
    /// Duration of the break in seconds
    pub duration_s: u64,
    /// Postponed time in seconds
    pub postponed_s: u64,
    /// If the break should be strictly followed
    pub strict_mode: bool,
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

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(rename_all = "camelCase")]
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

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(rename_all = "camelCase")]
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

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(rename_all = "camelCase")]
pub struct ScheduleSettings {
    /// Unique identifier for the break settings
    pub name: String,
    /// If the break is enabled
    pub enabled: bool,
    /// Theme settings for the break
    pub time_range: TimeRange,
    /// Audio settings for the break
    pub days_of_week: Vec<Weekday>,
    /// If the break should be shown in fullscreen
    pub notification_before_s: u64,

    /// Ideas settings for the break
    pub mini_breaks: MiniBreakSettings,
    /// Duration of the break in seconds
    pub long_breaks: LongBreakSettings,
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TS)]
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

impl Default for AttentionId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(rename_all = "camelCase")]
pub struct AttentionSettings {
    /// Unique identifier for the break settings
    pub id: AttentionId,
    /// If the break is enabled
    pub name: String,
    /// Theme settings for the break
    pub enabled: bool,
    /// Audio settings for the break
    pub theme: ThemeSettings,
    /// If the break should be shown in fullscreen
    pub times: ShortTimes,
    /// Ideas settings for the break
    pub days_of_week: Vec<Weekday>,
    /// Duration of the break in seconds
    pub title: String,
    /// Postponed time in seconds
    pub message: String,
    /// If the break should be strictly followed
    pub duration_s: u64,
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
