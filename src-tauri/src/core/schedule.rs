use std::{fmt::Display, sync::atomic::AtomicU32};

use crate::core::{
    audio::AudioSettings,
    suggestions::SuggestionsSettings,
    theme::ThemeSettings,
    time::{ShortTimes, TimeRange},
};
use chrono::Weekday;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

static NEXT_SCHEDULE_ID: AtomicU32 = AtomicU32::new(0);
static NEXT_ATTENTION_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TS)]
pub struct BreakId(u32);

impl Display for BreakId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "B{}", self.0)
    }
}

impl BreakId {
    pub fn new() -> Self {
        BreakId(NEXT_SCHEDULE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

impl From<BreakId> for u32 {
    fn from(id: BreakId) -> Self {
        id.0
    }
}

impl Default for BreakId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
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
    /// Suggestions display settings for the break
    pub suggestions: SuggestionsSettings,
    /// Duration of the break in seconds
    pub duration_s: u32,
    /// Postponed time in seconds
    pub postponed_s: u32,
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
            suggestions: SuggestionsSettings::default(),
            duration_s: 20,   // Last default to 20 seconds
            postponed_s: 300, // Postpone default to 5 minutes
            strict_mode: false,
        }
    }
}

/// Settings for mini breaks
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct MiniBreakSettings {
    /// Base break settings
    #[serde(flatten)]
    pub base: BaseBreakSettings,

    /// Interval between mini breaks in seconds
    pub interval_s: u32,
}

impl Default for MiniBreakSettings {
    fn default() -> Self {
        MiniBreakSettings {
            base: BaseBreakSettings::default(),
            interval_s: 1200, // Default to 20 minutes between mini breaks
        }
    }
}

/// Settings for long breaks
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct LongBreakSettings {
    /// Base break settings
    #[serde(flatten)]
    pub base: BaseBreakSettings,

    /// Number of mini breaks after which to trigger a long break
    pub after_mini_breaks: u8,
}

impl Default for LongBreakSettings {
    fn default() -> Self {
        LongBreakSettings {
            base: BaseBreakSettings {
                duration_s: 300, // Last default to 5 minutes
                ..BaseBreakSettings::default()
            },
            after_mini_breaks: 4, // Default to have a long break after 4 mini breaks
        }
    }
}

/// Settings for a break schedule
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ScheduleSettings {
    /// Unique identifier for the break settings
    pub name: String,
    /// If the break is enabled
    pub enabled: bool,
    /// Time range during which the schedule is active
    pub time_range: TimeRange,
    /// Days of the week when the schedule is active
    pub days_of_week: Vec<Weekday>,
    /// Notification time before breaks in seconds
    pub notification_before_s: u32,
    /// Mini break settings
    pub mini_breaks: MiniBreakSettings,
    /// Long break settings
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
    #[must_use]
    pub fn has_notification(&self) -> bool {
        self.notification_before_s > 0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TS)]
pub struct AttentionId(u32);

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

impl From<AttentionId> for u32 {
    fn from(id: AttentionId) -> Self {
        id.0
    }
}

impl Default for AttentionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Settings for attention reminders
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct AttentionSettings {
    /// Unique identifier for the attention settings
    pub id: AttentionId,
    /// Name of the attention reminder
    pub name: String,
    /// If the attention is enabled
    pub enabled: bool,
    /// Theme settings for the attention
    pub theme: ThemeSettings,
    /// Times when the attention should trigger
    pub times: ShortTimes,
    /// Days of the week when the attention should trigger
    pub days_of_week: Vec<Weekday>,
    /// Title of the attention reminder
    pub title: String,
    /// Message of the attention reminder
    pub message: String,
    /// Duration of the attention reminder in seconds
    pub duration_s: u32,
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
            duration_s: 5, // Default to 5 seconds
        }
    }
}
