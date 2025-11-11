use bitflags::bitflags;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum_macros::{Display as StrumDisplay, EnumIter, EnumString};
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
    /// Number of times this break has been postponed (for future limit enforcement)
    pub postpone_count: u8,
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
    PostponeBreak,
    /// Skip the current break immediately
    SkipBreak,
    /// Manually trigger a break for testing/debugging
    TriggerEvent(SchedulerEvent),
    /// Request the break scheduler to emit its current status
    RequestBreakStatus,
    /// Notify that a break or an attention (i.e., a prompt) has finished normally
    PromptFinished(SchedulerEvent),
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::UpdateConfig(_) => write!(f, "UpdateConfig"),
            Command::Pause(reason) => write!(f, "Pause({reason})"),
            Command::Resume(reason) => write!(f, "Resume({reason})"),
            Command::PostponeBreak => write!(f, "PostponeBreak"),
            Command::TriggerEvent(event) => write!(f, "TriggerBreak({event})"),
            Command::SkipBreak => write!(f, "SkipBreak"),
            Command::RequestBreakStatus => write!(f, "RequestBreakStatus"),
            Command::PromptFinished(event) => write!(f, "PromptFinished({event})"),
        }
    }
}

// ============================================================================
// State Types
// ============================================================================

/// Reason for pausing the scheduler
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, StrumDisplay, EnumString, EnumIter)]
#[strum(serialize_all = "PascalCase")]
pub enum PauseReason {
    UserIdle,
    Dnd,
    Manual,
    AppExclusion,
}

bitflags! {
    /// Flags representing the current pause reasons
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PauseReasons: u8 {
        const USER_IDLE     = 1 << 0; // 0b0001
        const DND           = 1 << 1; // 0b0010
        const MANUAL        = 1 << 2; // 0b0100
        const APP_EXCLUSION = 1 << 3; // 0b1000
    }
}

impl From<PauseReason> for PauseReasons {
    fn from(reason: PauseReason) -> Self {
        match reason {
            PauseReason::UserIdle => PauseReasons::USER_IDLE,
            PauseReason::Dnd => PauseReasons::DND,
            PauseReason::Manual => PauseReasons::MANUAL,
            PauseReason::AppExclusion => PauseReasons::APP_EXCLUSION,
        }
    }
}

impl PauseReasons {
    /// Returns the number of active pause reasons
    #[must_use]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(self) -> usize {
        self.bits().count_ones() as usize
    }

    /// Returns an iterator over all active pause reasons
    pub fn active_reasons(self) -> impl Iterator<Item = PauseReason> {
        self.iter().map(|flag| match flag {
            PauseReasons::USER_IDLE => PauseReason::UserIdle,
            PauseReasons::DND => PauseReason::Dnd,
            PauseReasons::MANUAL => PauseReason::Manual,
            PauseReasons::APP_EXCLUSION => PauseReason::AppExclusion,
            _ => unreachable!(),
        })
    }

    /// Returns a vector of all active pause reasons
    #[must_use]
    pub fn to_vec(self) -> Vec<PauseReason> {
        self.active_reasons().collect()
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
    /// The current mini break counter (for tracking long break triggers)
    pub mini_break_counter: u8,
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
        assert_eq!(Command::PostponeBreak.to_string(), "PostponeBreak");
        assert_eq!(Command::SkipBreak.to_string(), "SkipBreak");
        assert_eq!(
            Command::RequestBreakStatus.to_string(),
            "RequestBreakStatus"
        );
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

    // PauseReasons tests
    #[test]
    fn test_pause_reasons_len() {
        assert_eq!(PauseReasons::empty().len(), 0);
        assert_eq!(PauseReasons::USER_IDLE.len(), 1);
        assert_eq!((PauseReasons::USER_IDLE | PauseReasons::DND).len(), 2);
        assert_eq!(PauseReasons::all().len(), 4);
    }

    #[test]
    fn test_pause_reasons_active_reasons() {
        let empty: Vec<_> = PauseReasons::empty().active_reasons().collect();
        assert_eq!(empty, vec![]);

        let single: Vec<_> = PauseReasons::USER_IDLE.active_reasons().collect();
        assert_eq!(single, vec![PauseReason::UserIdle]);

        let multiple: Vec<_> = (PauseReasons::USER_IDLE | PauseReasons::DND)
            .active_reasons()
            .collect();
        assert_eq!(multiple.len(), 2);
        assert!(multiple.contains(&PauseReason::UserIdle));
        assert!(multiple.contains(&PauseReason::Dnd));
    }

    #[test]
    fn test_pause_reasons_to_vec() {
        assert_eq!(PauseReasons::empty().to_vec(), vec![]);
        assert_eq!(
            PauseReasons::USER_IDLE.to_vec(),
            vec![PauseReason::UserIdle]
        );
    }
}
