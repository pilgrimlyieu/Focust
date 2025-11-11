//! DND event definitions

use strum_macros::{Display, EnumIter, EnumString};

/// DND state change event
///
/// Emitted when the system DND/Focus Assist state changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, EnumIter)]
#[strum(serialize_all = "PascalCase")]
pub enum DndEvent {
    /// DND/Focus Assist was enabled
    Started,

    /// DND/Focus Assist was disabled
    Finished,
}

impl DndEvent {
    /// Check if this event indicates DND is now active
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Started)
    }

    /// Get a human-readable description of the event
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::Started => "DND enabled",
            Self::Finished => "DND disabled",
        }
    }
}
