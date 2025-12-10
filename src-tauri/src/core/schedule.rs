use std::fmt::{self, Display};
use std::sync::atomic::{AtomicU32, Ordering};

use bitflags::bitflags;
use chrono::Weekday;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core::{
    audio::AudioSettings,
    suggestions::SuggestionsSettings,
    theme::ThemeSettings,
    time::{ShortTimes, TimeRange},
};

static NEXT_SCHEDULE_ID: AtomicU32 = AtomicU32::new(0);
static NEXT_ATTENTION_ID: AtomicU32 = AtomicU32::new(0);

bitflags! {
    /// Flags representing days of the week
    ///
    /// This is an internal representation using bitflags for efficient storage and operations.
    /// For communication with frontend and serialization, it converts to/from `Vec<Weekday>`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DaysOfWeek: u8 {
        const SUNDAY    = 1 << 0; // 0b0000000
        const MONDAY    = 1 << 1; // 0b0000010
        const TUESDAY   = 1 << 2; // 0b0000100
        const WEDNESDAY = 1 << 3; // 0b0001000
        const THURSDAY  = 1 << 4; // 0b0010000
        const FRIDAY    = 1 << 5; // 0b0100000
        const SATURDAY  = 1 << 6; // 0b1000000
    }
}

impl From<Weekday> for DaysOfWeek {
    fn from(day: Weekday) -> Self {
        match day {
            Weekday::Sun => DaysOfWeek::SUNDAY,
            Weekday::Mon => DaysOfWeek::MONDAY,
            Weekday::Tue => DaysOfWeek::TUESDAY,
            Weekday::Wed => DaysOfWeek::WEDNESDAY,
            Weekday::Thu => DaysOfWeek::THURSDAY,
            Weekday::Fri => DaysOfWeek::FRIDAY,
            Weekday::Sat => DaysOfWeek::SATURDAY,
        }
    }
}

impl From<DaysOfWeek> for Weekday {
    fn from(days: DaysOfWeek) -> Self {
        match days {
            DaysOfWeek::SUNDAY => Weekday::Sun,
            DaysOfWeek::MONDAY => Weekday::Mon,
            DaysOfWeek::TUESDAY => Weekday::Tue,
            DaysOfWeek::WEDNESDAY => Weekday::Wed,
            DaysOfWeek::THURSDAY => Weekday::Thu,
            DaysOfWeek::FRIDAY => Weekday::Fri,
            DaysOfWeek::SATURDAY => Weekday::Sat,
            _ => unreachable!("Attempted to convert multiple DaysOfWeek to a single Weekday"),
        }
    }
}

impl DaysOfWeek {
    /// Check if the `DaysOfWeek` contains a specific day
    #[must_use]
    pub fn contains_day(self, day: Weekday) -> bool {
        self.contains(DaysOfWeek::from(day))
    }

    /// Convert to a Vec of Weekdays (for serialization and frontend communication)
    #[must_use]
    pub fn to_vec(self) -> Vec<Weekday> {
        self.iter().map(Weekday::from).collect()
    }

    /// Create from a Vec of Weekdays (from deserialization and frontend)
    #[must_use]
    pub fn from_vec(days: &[Weekday]) -> Self {
        let mut result = DaysOfWeek::empty();
        for &day in days {
            result.insert(DaysOfWeek::from(day));
        }
        result
    }

    /// Workdays (Monday to Friday)
    #[must_use]
    pub const fn workdays() -> Self {
        DaysOfWeek::MONDAY
            .union(DaysOfWeek::TUESDAY)
            .union(DaysOfWeek::WEDNESDAY)
            .union(DaysOfWeek::THURSDAY)
            .union(DaysOfWeek::FRIDAY)
    }

    /// Weekend (Saturday and Sunday)
    #[must_use]
    pub const fn weekend() -> Self {
        DaysOfWeek::SATURDAY.union(DaysOfWeek::SUNDAY)
    }

    /// Get the number of active days
    #[must_use]
    #[expect(clippy::len_without_is_empty)]
    pub fn len(self) -> usize {
        self.bits().count_ones() as usize
    }
}

impl Default for DaysOfWeek {
    fn default() -> Self {
        Self::all()
    }
}

// Custom serialization: serialize as Vec<Weekday> for compatibility
impl Serialize for DaysOfWeek {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_vec().serialize(serializer)
    }
}

// Custom deserialization: deserialize from Vec<Weekday>
impl<'de> Deserialize<'de> for DaysOfWeek {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let days = Vec::<Weekday>::deserialize(deserializer)?;
        Ok(Self::from_vec(&days))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TS)]
pub struct BreakId(u32);

impl Display for BreakId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "B{}", self.0)
    }
}

impl BreakId {
    pub fn new() -> Self {
        BreakId(NEXT_SCHEDULE_ID.fetch_add(1, Ordering::Relaxed))
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
#[serde(default, rename_all = "camelCase")]
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
    /// Maximum number of times a break can be postponed
    pub max_postpone_count: u8,
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
            duration_s: 20,        // Last default to 20 seconds
            postponed_s: 300,      // Postpone default to 5 minutes
            max_postpone_count: 2, // Default: allow 2 postpones
            strict_mode: false,
        }
    }
}

/// Settings for mini breaks
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(default, rename_all = "camelCase")]
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
#[serde(default, rename_all = "camelCase")]
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
#[serde(default, rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ScheduleSettings {
    /// Unique identifier for the break settings
    pub name: String,
    /// If the break is enabled
    pub enabled: bool,
    /// Time range during which the schedule is active
    pub time_range: TimeRange,
    /// Days of the week when the schedule is active
    /// Internally stored as bitflags, but serialized as `Vec<Weekday>` for compatibility
    #[ts(as = "Vec<Weekday>")]
    pub days_of_week: DaysOfWeek,
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
            name: "Default Schedule".to_owned(),
            enabled: true,
            time_range: TimeRange::default(),
            days_of_week: DaysOfWeek::all(), // Active every day
            notification_before_s: 10,       // Default to 10 seconds before breaks
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A{}", self.0)
    }
}

impl AttentionId {
    pub fn new() -> Self {
        AttentionId(NEXT_ATTENTION_ID.fetch_add(1, Ordering::Relaxed))
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
    /// Internally stored as bitflags, but serialized as `Vec<Weekday>` for compatibility
    #[ts(as = "Vec<Weekday>")]
    pub days_of_week: DaysOfWeek,
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
            name: "Default Attention".to_owned(),
            enabled: true,
            theme: ThemeSettings::default(),
            times: ShortTimes::default(),
            days_of_week: DaysOfWeek::all(), // Active every day
            title: "Attention Reminder".to_owned(),
            message: "This is an attention reminder.".to_owned(),
            duration_s: 5, // Default to 5 seconds
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // DaysOfWeek tests
    #[test]
    fn days_of_week_from_single_weekday() {
        assert_eq!(DaysOfWeek::from(Weekday::Mon), DaysOfWeek::MONDAY);
        assert_eq!(DaysOfWeek::from(Weekday::Sun), DaysOfWeek::SUNDAY);
    }

    #[test]
    fn days_of_week_contains_day() {
        let days = DaysOfWeek::MONDAY | DaysOfWeek::FRIDAY;
        assert!(days.contains_day(Weekday::Mon));
        assert!(days.contains_day(Weekday::Fri));
        assert!(!days.contains_day(Weekday::Wed));
    }

    #[test]
    fn days_of_week_to_vec() {
        let days = DaysOfWeek::MONDAY | DaysOfWeek::WEDNESDAY | DaysOfWeek::FRIDAY;
        let vec = days.to_vec();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec, vec![Weekday::Mon, Weekday::Wed, Weekday::Fri]);
    }

    #[test]
    fn days_of_week_to_vec_maintains_order() {
        // Even if created out of order, to_vec should return in week order
        let days = DaysOfWeek::FRIDAY | DaysOfWeek::MONDAY | DaysOfWeek::WEDNESDAY;
        let vec = days.to_vec();
        assert_eq!(vec, vec![Weekday::Mon, Weekday::Wed, Weekday::Fri]);
    }

    #[test]
    fn full_days_of_week_to_vec() {
        let days = DaysOfWeek::all();
        let vec = days.to_vec();
        assert_eq!(vec.len(), 7);
    }

    #[test]
    fn days_of_week_from_vec() {
        let vec = vec![Weekday::Tue, Weekday::Thu, Weekday::Sat];
        let days = DaysOfWeek::from_vec(&vec);
        assert!(days.contains_day(Weekday::Tue));
        assert!(days.contains_day(Weekday::Thu));
        assert!(days.contains_day(Weekday::Sat));
        assert!(!days.contains_day(Weekday::Mon));
    }

    #[test]
    fn days_of_week_from_vec_with_duplicates() {
        let vec = vec![Weekday::Mon, Weekday::Mon, Weekday::Tue];
        let days = DaysOfWeek::from_vec(&vec);
        assert_eq!(days.to_vec(), vec![Weekday::Mon, Weekday::Tue]);
    }

    #[test]
    fn days_of_week_all() {
        let days = DaysOfWeek::all();
        assert_eq!(days.to_vec().len(), 7);
        for day in &[
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ] {
            assert!(days.contains_day(*day));
        }
    }

    #[test]
    fn days_of_week_workdays() {
        let days = DaysOfWeek::workdays();
        assert_eq!(days.to_vec().len(), 5);
        assert!(days.contains_day(Weekday::Mon));
        assert!(days.contains_day(Weekday::Fri));
        assert!(!days.contains_day(Weekday::Sat));
        assert!(!days.contains_day(Weekday::Sun));
    }

    #[test]
    fn days_of_week_weekend() {
        let days = DaysOfWeek::weekend();
        assert_eq!(days.to_vec().len(), 2);
        assert!(days.contains_day(Weekday::Sat));
        assert!(days.contains_day(Weekday::Sun));
        assert!(!days.contains_day(Weekday::Fri));
    }

    #[test]
    fn days_of_week_serialization() {
        let days = DaysOfWeek::MONDAY | DaysOfWeek::WEDNESDAY | DaysOfWeek::FRIDAY;
        let json = serde_json::to_string(&days).unwrap();
        // Should serialize as array of weekday strings
        assert!(json.contains("Mon"));
        assert!(json.contains("Wed"));
        assert!(json.contains("Fri"));
    }

    #[test]
    fn days_of_week_deserialization() {
        let json = r#"["Mon","Wed","Fri"]"#;
        let days: DaysOfWeek = serde_json::from_str(json).unwrap();
        assert!(days.contains_day(Weekday::Mon));
        assert!(days.contains_day(Weekday::Wed));
        assert!(days.contains_day(Weekday::Fri));
        assert!(!days.contains_day(Weekday::Tue));
    }

    #[test]
    fn days_of_week_roundtrip() {
        let original = DaysOfWeek::TUESDAY | DaysOfWeek::THURSDAY | DaysOfWeek::SATURDAY;
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: DaysOfWeek = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn schedule_settings_days_of_week_serialization() {
        let settings = ScheduleSettings {
            days_of_week: DaysOfWeek::workdays(),
            ..Default::default()
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: ScheduleSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings.days_of_week, deserialized.days_of_week);
        assert_eq!(deserialized.days_of_week.to_vec().len(), 5);
    }

    #[test]
    fn attention_settings_days_of_week_serialization() {
        let settings = AttentionSettings {
            days_of_week: DaysOfWeek::weekend(),
            ..Default::default()
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AttentionSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings.days_of_week, deserialized.days_of_week);
        assert_eq!(deserialized.days_of_week.to_vec().len(), 2);
    }
}
