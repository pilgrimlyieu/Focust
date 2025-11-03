use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::config::AppConfig;
use crate::core::schedule::{AttentionId, BreakId};

// ============================================================================
// Event Types - Sent to Frontend
// ============================================================================

/// Events sent to the frontend for display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub enum SchedulerEvent {
    MiniBreak(BreakId),
    LongBreak(BreakId),
    Attention(AttentionId),
}

impl Display for SchedulerEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchedulerEvent::MiniBreak(id) => write!(f, "MiniBreak({id})"),
            SchedulerEvent::LongBreak(id) => write!(f, "LongBreak({id})"),
            SchedulerEvent::Attention(id) => write!(f, "Attention({id})"),
        }
    }
}

impl SchedulerEvent {
    /// Check if the event is a break (mini or long)
    #[must_use]
    pub fn is_break(&self) -> bool {
        matches!(
            self,
            SchedulerEvent::MiniBreak(_) | SchedulerEvent::LongBreak(_)
        )
    }

    /// Check if the event is an attention reminder
    #[must_use]
    pub fn is_attention(&self) -> bool {
        matches!(self, SchedulerEvent::Attention(_))
    }

    /// Check if the event is a mini break
    #[must_use]
    pub fn is_mini(&self) -> bool {
        matches!(self, SchedulerEvent::MiniBreak(_))
    }

    /// Check if the event is a long break
    #[must_use]
    pub fn is_long(&self) -> bool {
        matches!(self, SchedulerEvent::LongBreak(_))
    }
}

// ============================================================================
// Internal Types
// ============================================================================

/// Internal representation of a scheduled break (used by `BreakScheduler`)
#[derive(Debug, Clone)]
pub(crate) struct BreakInfo {
    pub break_time: DateTime<Utc>,
    pub notification_time: Option<DateTime<Utc>>,
    pub event: SchedulerEvent,
}

// ============================================================================
// Command Types
// ============================================================================

/// Commands sent to the scheduler from external sources
#[derive(Debug, Clone)]
pub enum Command {
    /// Update the scheduler configuration
    UpdateConfig(AppConfig),
    /// Pause the break scheduler
    Pause(PauseReason),
    /// Resume the break scheduler
    Resume(PauseReason),
    /// Postpone the current break
    Postpone,
    /// Skip the current break immediately
    SkipBreak,
    /// Manually trigger a break for testing/debugging
    TriggerEvent(SchedulerEvent),
    /// Request the scheduler to emit its current status
    RequestStatus,
    /// Notify that a break has finished normally (from frontend)
    BreakFinished(SchedulerEvent),
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::UpdateConfig(_) => write!(f, "UpdateConfig"),
            Command::Pause(reason) => write!(f, "Pause({reason})"),
            Command::Resume(reason) => write!(f, "Resume({reason})"),
            Command::Postpone => write!(f, "Postpone"),
            Command::TriggerEvent(event) => write!(f, "TriggerBreak({event})"),
            Command::SkipBreak => write!(f, "SkipBreak"),
            Command::RequestStatus => write!(f, "RequestStatus"),
            Command::BreakFinished(event) => write!(f, "BreakFinished({event})"),
        }
    }
}

// ============================================================================
// State Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseReason {
    UserIdle,
    Dnd,
    Manual,
    AppExclusion,
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

// ============================================================================
// Status Types - For UI Display
// ============================================================================

/// Scheduler status information for UI display
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct SchedulerStatus {
    /// Whether the break scheduler is currently paused
    pub paused: bool,
    /// The next scheduled break event (if any)
    pub next_event: Option<SchedulerEventInfo>,
}

/// Information about a scheduled event
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct SchedulerEventInfo {
    /// The type of event
    pub kind: SchedulerEvent,
    /// When the event will occur (ISO 8601 timestamp)
    pub time: String,
    /// Seconds until the event
    pub seconds_until: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    // SchedulerEvent tests
    #[test]
    fn test_scheduler_event_display() {
        use crate::core::schedule::{AttentionId, BreakId};
        let mini_break = SchedulerEvent::MiniBreak(BreakId::new());
        let long_break = SchedulerEvent::LongBreak(BreakId::new());
        let attention = SchedulerEvent::Attention(AttentionId::new());

        assert!(mini_break.to_string().starts_with("MiniBreak("));
        assert!(long_break.to_string().starts_with("LongBreak("));
        assert!(attention.to_string().starts_with("Attention("));
    }

    #[test]
    fn test_scheduler_event_equality() {
        use crate::core::schedule::BreakId;
        let id = BreakId::new();
        let event1 = SchedulerEvent::MiniBreak(id);
        let event2 = SchedulerEvent::MiniBreak(id);
        let event3 = SchedulerEvent::MiniBreak(BreakId::new());

        assert_eq!(event1, event2);
        assert_ne!(event1, event3);
    }

    // Command tests
    #[test]
    fn test_command_display() {
        assert_eq!(Command::Postpone.to_string(), "Postpone");
        assert_eq!(Command::SkipBreak.to_string(), "SkipBreak");
        assert_eq!(Command::RequestStatus.to_string(), "RequestStatus");
    }

    #[test]
    fn test_command_pause_display() {
        let cmd = Command::Pause(PauseReason::UserIdle);
        assert_eq!(cmd.to_string(), "Pause(UserIdle)");
    }

    // PauseReason tests
    #[test]
    fn test_pause_reason_equality() {
        assert_eq!(PauseReason::UserIdle, PauseReason::UserIdle);
        assert_ne!(PauseReason::UserIdle, PauseReason::Manual);
    }

    #[test]
    fn test_pause_reason_display() {
        assert_eq!(PauseReason::UserIdle.to_string(), "UserIdle");
        assert_eq!(PauseReason::Dnd.to_string(), "Dnd");
        assert_eq!(PauseReason::Manual.to_string(), "Manual");
        assert_eq!(PauseReason::AppExclusion.to_string(), "AppExclusion");
    }
}
