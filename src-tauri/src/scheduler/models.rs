use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;
use crate::core::schedule::{AttentionId, BreakId};

#[derive(Debug, Clone)]
pub struct ScheduledEvent {
    pub time: DateTime<Utc>,
    pub kind: EventKind,
}

/// Defines the different kinds of events the scheduler can handle.
/// The order of variants defines their priority (lower discriminant = higher priority).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
    UpdateConfig(AppConfig),
    Pause(PauseReason),
    Resume(PauseReason),
    Postpone,
    Shutdown, // Added for completeness, if needed
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::UpdateConfig(_) => write!(f, "UpdateConfig"),
            Command::Pause(reason) => write!(f, "Pause({reason:?})"),
            Command::Resume(reason) => write!(f, "Resume({reason:?})"),
            Command::Postpone => write!(f, "Postpone"),
            Command::Shutdown => write!(f, "Shutdown"),
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
