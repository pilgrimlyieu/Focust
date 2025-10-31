use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::config::AppConfig;
use crate::core::schedule::{AttentionId, BreakId};

#[derive(Debug, Clone)]
pub struct ScheduledEvent {
    pub time: DateTime<Utc>,
    pub kind: EventKind,
}

/// Defines the different kinds of events the scheduler can handle.
/// The order of variants defines their priority (lower discriminant = higher priority).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub enum EventKind {
    Attention(AttentionId),
    LongBreak(BreakId),
    MiniBreak(BreakId),
    Notification(NotificationKind),
}

impl Display for EventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventKind::Attention(id) => write!(f, "Attention({id})"),
            EventKind::LongBreak(id) => write!(f, "LongBreak({id})"),
            EventKind::MiniBreak(id) => write!(f, "MiniBreak({id})"),
            EventKind::Notification(kind) => write!(f, "Notification({kind:?})"),
        }
    }
}

/// Specifies the type of break for a notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum NotificationKind {
    LongBreak(BreakId),
    MiniBreak(BreakId),
}

impl Display for NotificationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationKind::LongBreak(id) => write!(f, "LongBreak({id})"),
            NotificationKind::MiniBreak(id) => write!(f, "MiniBreak({id})"),
        }
    }
}

/// Commands that can be sent to the scheduler to control its behavior.
#[derive(Debug, Clone)]
pub enum Command {
    /// Update the scheduler configuration
    UpdateConfig(AppConfig),
    /// Pause the scheduler for a given reason
    Pause(PauseReason),
    /// Resume the scheduler from a paused state for a given reason
    Resume(PauseReason),
    /// Postpone the current break
    Postpone,
    /// Manually trigger a break for testing/debugging purposes
    TriggerBreak(EventKind),
    /// Skip the current break immediately
    SkipBreak,
    /// Request the scheduler to emit its current status
    RequestStatus,
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::UpdateConfig(_) => write!(f, "UpdateConfig"),
            Command::Pause(reason) => write!(f, "Pause({reason:?})"),
            Command::Resume(reason) => write!(f, "Resume({reason:?})"),
            Command::Postpone => write!(f, "Postpone"),
            Command::TriggerBreak(kind) => write!(f, "TriggerBreak({kind})"),
            Command::SkipBreak => write!(f, "SkipBreak"),
            Command::RequestStatus => write!(f, "RequestStatus"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PauseReason {
    UserIdle,
    Dnd, // TODO
    Manual,
    AppExclusion, // TODO
}

impl Display for PauseReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PauseReason::UserIdle => write!(f, "UserIdle"),
            PauseReason::Dnd => write!(f, "Dnd"),
            PauseReason::Manual => write!(f, "Manual"),
            PauseReason::AppExclusion => write!(f, "AppExclusion"),
        }
    }
}

/// Scheduler status information for UI display
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct SchedulerStatus {
    /// Whether the scheduler is currently paused
    pub paused: bool,
    /// The next scheduled event (if any)
    pub next_event: Option<SchedulerEventInfo>,
}

/// Information about a scheduled event
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct SchedulerEventInfo {
    /// The type of event
    pub kind: EventKind,
    /// When the event will occur (ISO 8601 timestamp)
    pub time: String,
    /// Seconds until the event
    pub seconds_until: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // EventKind priority test
    #[test]
    fn test_event_kind_priority_order() {
        use crate::core::schedule::{AttentionId, BreakId};
        // EventKind priority order should be: Attention > LongBreak > MiniBreak > Notification
        let attention = EventKind::Attention(AttentionId::new());
        let long_break = EventKind::LongBreak(BreakId::new());
        let mini_break = EventKind::MiniBreak(BreakId::new());
        let notification = EventKind::Notification(NotificationKind::MiniBreak(BreakId::new()));

        // Attention has highest priority
        assert!(attention < long_break);
        assert!(attention < mini_break);
        assert!(attention < notification);

        // LongBreak has second highest priority
        assert!(long_break < mini_break);
        assert!(long_break < notification);

        // MiniBreak has third priority
        assert!(mini_break < notification);
    }

    #[test]
    fn test_event_kind_equality() {
        use crate::core::schedule::BreakId;
        let id = BreakId::new();
        let event1 = EventKind::MiniBreak(id);
        let event2 = EventKind::MiniBreak(id);
        let event3 = EventKind::MiniBreak(BreakId::new());

        assert_eq!(event1, event2);
        assert_ne!(event1, event3);
    }

    #[test]
    fn test_event_kind_display() {
        use crate::core::schedule::{AttentionId, BreakId};
        let attention_id = AttentionId::new();
        let break_id = BreakId::new();
        let long_break_id = BreakId::new();

        assert!(
            EventKind::Attention(attention_id)
                .to_string()
                .starts_with("Attention(")
        );
        assert!(
            EventKind::LongBreak(long_break_id)
                .to_string()
                .starts_with("LongBreak(")
        );
        assert!(
            EventKind::MiniBreak(break_id)
                .to_string()
                .starts_with("MiniBreak(")
        );
    }

    // NotificationKind tests
    #[test]
    fn test_notification_kind_ordering() {
        use crate::core::schedule::BreakId;
        let long_notif = NotificationKind::LongBreak(BreakId::new());
        let mini_notif = NotificationKind::MiniBreak(BreakId::new());

        assert!(long_notif < mini_notif);
    }

    #[test]
    fn test_notification_kind_display() {
        use crate::core::schedule::BreakId;
        let break_id = BreakId::new();
        let notif = NotificationKind::MiniBreak(break_id);
        assert!(notif.to_string().starts_with("MiniBreak("));
    }

    // ScheduledEvent tests
    #[test]
    fn test_scheduled_event_creation() {
        use crate::core::schedule::BreakId;
        let time = Utc.with_ymd_and_hms(2025, 9, 1, 12, 0, 0).unwrap();
        let event = ScheduledEvent {
            time,
            kind: EventKind::MiniBreak(BreakId::new()),
        };

        assert_eq!(event.time, time);
    }

    #[test]
    fn test_scheduled_event_clone() {
        use crate::core::schedule::BreakId;
        let time = Utc.with_ymd_and_hms(2025, 9, 1, 12, 0, 0).unwrap();
        let break_id = BreakId::new();
        let event = ScheduledEvent {
            time,
            kind: EventKind::LongBreak(break_id),
        };

        let cloned = event.clone();
        assert_eq!(event.time, cloned.time);
        assert_eq!(event.kind, cloned.kind);
    }

    // Command tests
    #[test]
    fn test_command_display() {
        assert_eq!(Command::Postpone.to_string(), "Postpone");
        assert_eq!(
            Command::Pause(PauseReason::UserIdle).to_string(),
            "Pause(UserIdle)"
        );
        assert_eq!(
            Command::Resume(PauseReason::Manual).to_string(),
            "Resume(Manual)"
        );
    }

    #[test]
    fn test_command_update_config_display() {
        let config = AppConfig::default();
        let cmd = Command::UpdateConfig(config);
        assert_eq!(cmd.to_string(), "UpdateConfig");
    }

    #[test]
    fn test_command_clone() {
        let cmd = Command::Postpone;
        let cloned = cmd.clone();
        assert_eq!(cmd.to_string(), cloned.to_string());
    }

    // PauseReason tests
    #[test]
    fn test_pause_reason_equality() {
        assert_eq!(PauseReason::UserIdle, PauseReason::UserIdle);
        assert_ne!(PauseReason::UserIdle, PauseReason::Manual);
    }

    #[test]
    fn test_pause_reason_clone() {
        let reason = PauseReason::Dnd;
        let cloned = reason.clone();
        assert_eq!(reason, cloned);
    }

    #[test]
    fn test_pause_reason_display() {
        assert_eq!(PauseReason::UserIdle.to_string(), "UserIdle");
        assert_eq!(PauseReason::Dnd.to_string(), "Dnd");
        assert_eq!(PauseReason::Manual.to_string(), "Manual");
        assert_eq!(PauseReason::AppExclusion.to_string(), "AppExclusion");
    }

    // Serialization tests (EventKind and NotificationKind)
    #[test]
    fn test_event_kind_serialization() {
        use crate::core::schedule::BreakId;
        let event = EventKind::MiniBreak(BreakId::new());
        let json = serde_json::to_string(&event).expect("Failed to serialize");
        let _deserialized: EventKind = serde_json::from_str(&json).expect("Failed to deserialize");
        // Should be able to deserialize after serialization
        assert!(json.contains("miniBreak"));
    }

    #[test]
    fn test_notification_kind_serialization() {
        use crate::core::schedule::BreakId;
        let notif = NotificationKind::LongBreak(BreakId::new());
        let json = serde_json::to_string(&notif).expect("Failed to serialize");
        let _deserialized: NotificationKind =
            serde_json::from_str(&json).expect("Failed to deserialize");
        // Should be able to deserialize after serialization
        assert!(json.contains("longBreak"));
    }

    // Edge case tests
    #[test]
    fn test_event_kind_with_new_id() {
        use crate::core::schedule::BreakId;
        let event = EventKind::MiniBreak(BreakId::new());
        assert!(event.to_string().contains("MiniBreak"));
    }

    #[test]
    fn test_event_kind_with_attention_id() {
        use crate::core::schedule::AttentionId;
        let event = EventKind::Attention(AttentionId::new());
        assert!(event.to_string().contains("Attention"));
    }
}
