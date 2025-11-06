/// Generic monitoring framework for scheduler state management
///
/// This module provides a trait-based system for implementing various monitors
/// that can trigger scheduler pause/resume actions based on system state.
mod app_whitelist;
mod dnd;
mod idle;
mod orchestrator;

pub use app_whitelist::AppWhitelistMonitor;
pub use dnd::DndMonitor;
pub use idle::IdleMonitor;
pub use orchestrator::spawn_monitor_tasks;

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use std::fmt::Display;

use super::models::{Command, PauseReason};

/// Result type for monitor check operations
pub type MonitorResult = Result<MonitorAction, MonitorError>;

/// Actions that a monitor can trigger
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorAction {
    /// No action needed
    None,
    /// Request to pause scheduler
    Pause(PauseReason),
    /// Request to resume scheduler
    Resume(PauseReason),
}

impl Display for MonitorAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonitorAction::None => write!(f, "None"),
            MonitorAction::Pause(reason) => write!(f, "Pause({reason})"),
            MonitorAction::Resume(reason) => write!(f, "Resume({reason})"),
        }
    }
}

/// Errors that can occur during monitoring
#[derive(Debug, thiserror::Error)]
pub enum MonitorError {
    #[error("Monitor check failed: {0}")]
    CheckFailed(String),
    #[error("Monitor unavailable on this platform")]
    Unavailable,
}

/// Trait for implementing system state monitors
///
/// Monitors periodically check system state and can trigger scheduler
/// pause/resume actions based on their findings.
pub trait Monitor: Send + Sync {
    /// Unique name for this monitor (for logging)
    fn name(&self) -> &'static str;

    /// Check interval for this monitor
    fn interval(&self) -> Duration;

    /// Perform a check and return the action to take
    ///
    /// This method is called periodically according to the monitor's interval.
    /// It should be fast and non-blocking.
    fn check(&mut self) -> Pin<Box<dyn Future<Output = MonitorResult> + Send + '_>>;

    /// Called when the monitor is first started
    ///
    /// Override this to perform any initialization needed
    fn on_start(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {})
    }

    /// Called when the monitor is stopped
    ///
    /// Override this to perform any cleanup needed
    fn on_stop(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {})
    }
}

/// Convert a `MonitorAction` to a `Command`
#[must_use]
pub fn action_to_command(action: MonitorAction) -> Option<Command> {
    match action {
        MonitorAction::None => None,
        MonitorAction::Pause(reason) => Some(Command::Pause(reason)),
        MonitorAction::Resume(reason) => Some(Command::Resume(reason)),
    }
}