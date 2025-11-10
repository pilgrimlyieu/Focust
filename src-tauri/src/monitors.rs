//! Generic monitoring framework for scheduler state management
//!
//! This module provides a trait-based system for implementing various monitors
//! that can trigger scheduler pause/resume actions based on system state.
//!
//! # Architecture
//!
//! The monitoring system consists of:
//!
//! - **`Monitor` trait**: Interface that all monitors must implement
//! - **Concrete monitors**: `IdleMonitor`, `DndMonitor`, `AppWhitelistMonitor`
//! - **Orchestrator**: Runs all monitors in a single task, checking at configured intervals
//! - **Action conversion**: Converts `MonitorAction` to `Command` for the scheduler
//! - **Session protection**: Unified session checking to prevent self-interference
//!
//! # Monitor Lifecycle
//!
//! 1. **Creation**: Monitors are created with their specific configuration
//! 2. **Initialization**: `on_start()` is called when the monitor begins
//! 3. **Monitoring Loop**: `check()` is called periodically at the monitor's interval
//! 4. **Shutdown**: `on_stop()` is called when the monitor stops (currently unused)
//!
//! # Session Protection
//!
//! During active sessions (break or attention prompts), monitors are typically skipped
//! to avoid self-interference. For example:
//!
//! - A break window may trigger system DND mode
//! - Without session protection, `DndMonitor` would detect this and pause the scheduler
//! - This would interrupt the break, causing unexpected behavior
//!
//! **Implementation**: The orchestrator checks `SharedState::in_any_session()` before
//! calling each monitor's `check()` method. Monitors with `skip_during_session() == true`
//! (the default) are automatically skipped during sessions.
//!
//! # Design Patterns
//!
//! - **Trait-based polymorphism**: All monitors implement the same interface
//! - **Event-driven where possible**: `DndMonitor` uses OS events instead of polling
//! - **Graceful degradation**: Monitors that fail to initialize return `Unavailable`
//! - **Non-blocking**: All checks must be fast and non-blocking
//! - **Unified session protection**: Orchestrator handles session checking, not monitors
//!
//! # Example
//!
//! ```rust,ignore
//! use focust::monitors::{Monitor, MonitorAction, MonitorResult};
//!
//! struct CustomMonitor {
//!     interval_secs: u64,
//! }
//!
//! impl Monitor for CustomMonitor {
//!     fn name(&self) -> &'static str {
//!         "CustomMonitor"
//!     }
//!
//!     fn interval(&self) -> u64 {
//!         self.interval_secs
//!     }
//!
//!     fn check(&mut self) -> Pin<Box<dyn Future<Output = MonitorResult> + Send + '_>> {
//!         Box::pin(async move {
//!             // Check some system state
//!             if should_pause() {
//!                 Ok(MonitorAction::Pause(PauseReason::Custom))
//!             } else {
//!                 Ok(MonitorAction::None)
//!             }
//!         })
//!     }
//! }
//! ```

mod app_whitelist;
mod dnd;
mod idle;
mod orchestrator;

pub use app_whitelist::AppWhitelistMonitor;
pub use dnd::DndMonitor;
pub use idle::IdleMonitor;
pub use orchestrator::spawn_monitor_tasks;

use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;

use crate::scheduler::models::{Command, PauseReason};

/// Result type for monitor check operations
pub type MonitorResult = Result<MonitorAction, MonitorError>;

/// Actions that a monitor can trigger
///
/// Monitors return these actions to indicate what the scheduler should do.
/// The orchestrator converts these to `Command`s and sends them to the scheduler.
///
/// # Examples
///
/// ```rust,ignore
/// // Monitor detects user is idle
/// Ok(MonitorAction::Pause(PauseReason::UserIdle))
///
/// // Monitor detects user is active again
/// Ok(MonitorAction::Resume(PauseReason::UserIdle))
///
/// // Nothing changed, no action needed
/// Ok(MonitorAction::None)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MonitorAction {
    /// No action needed
    ///
    /// The monitor checked the state and found no change requiring action.
    #[default]
    None,
    /// Request to pause scheduler
    ///
    /// The monitor detected a condition that requires pausing the scheduler.
    /// The reason is used for tracking which monitor caused the pause.
    Pause(PauseReason),
    /// Request to resume scheduler
    ///
    /// The monitor detected that a previous pause condition has cleared.
    /// The reason must match the one used in the corresponding `Pause`.
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
///
/// Monitors return these errors when they cannot complete a check.
/// The orchestrator logs these errors but continues running other monitors.
#[derive(Debug, thiserror::Error)]
pub enum MonitorError {
    /// The monitor check operation failed
    ///
    /// This indicates a transient error during checking. The monitor
    /// may recover on the next check.
    #[error("Monitor check failed: {0}")]
    CheckFailed(String),

    /// The monitor is not available on this platform
    ///
    /// This is a permanent error. The monitor will not be retried.
    /// Example: DND monitoring on a platform without DND support.
    #[error("Monitor unavailable on this platform")]
    Unavailable,
}

/// Trait for implementing system state monitors
///
/// Monitors observe system state and trigger scheduler pause/resume actions.
/// Each monitor checks a specific condition (user idle, DND mode, app exclusion, etc.)
/// and reports changes via `MonitorAction`.
///
/// # Implementation Requirements
///
/// - **Thread Safety**: Must be `Send + Sync` for multi-threaded access
/// - **Non-blocking**: `check()` should be fast and never block
/// - **Idempotent**: Multiple checks should be safe even if state hasn't changed
/// - **Error Handling**: Return `MonitorError` for failures, not panic
///
/// # Performance Considerations
///
/// - `check()` is called frequently (every 1-60 seconds typically)
/// - Avoid expensive operations in `check()`
/// - Use caching where appropriate
/// - Prefer event-driven approaches over polling when available
///
/// # State Management
///
/// - Monitors should track their own state to detect changes
/// - Only return `Pause`/`Resume` when state actually changes
/// - Return `None` when nothing changed
///
/// # Example Implementation
///
/// ```rust,ignore
/// struct MyMonitor {
///     last_state: bool,
/// }
///
/// impl Monitor for MyMonitor {
///     fn name(&self) -> &'static str { "MyMonitor" }
///     fn interval(&self) -> u64 { 5 }  // Check every 5 seconds
///
///     fn check(&mut self) -> Pin<Box<dyn Future<Output = MonitorResult> + Send + '_>> {
///         Box::pin(async move {
///             let current_state = check_system_condition()?;
///             
///             if current_state != self.last_state {
///                 self.last_state = current_state;
///                 
///                 if current_state {
///                     Ok(MonitorAction::Pause(PauseReason::Custom))
///                 } else {
///                     Ok(MonitorAction::Resume(PauseReason::Custom))
///                 }
///             } else {
///                 Ok(MonitorAction::None)
///             }
///         })
///     }
/// }
/// ```
pub trait Monitor: Send + Sync {
    /// Unique name for this monitor (for logging)
    ///
    /// This name appears in logs and helps identify which monitor
    /// triggered an action or encountered an error.
    ///
    /// # Examples
    ///
    /// - `"IdleMonitor"`: User idle detection
    /// - `"DndMonitor"`: Do Not Disturb mode detection
    /// - `"AppWhitelistMonitor"`: Application exclusion detection
    fn name(&self) -> &'static str;

    /// Check interval for this monitor in seconds
    ///
    /// The orchestrator will call `check()` approximately every N seconds,
    /// where N is the value returned by this method.
    ///
    /// # Choosing an Interval
    ///
    /// - **Fast checks (1-5s)**: Event-driven monitors that just poll an event queue
    /// - **Medium checks (5-30s)**: Lightweight system state checks
    /// - **Slow checks (30-60s)**: Expensive operations or rarely-changing state
    ///
    /// # Note
    ///
    /// The actual interval may be slightly longer due to scheduling overhead
    /// and time spent running other monitors.
    fn interval(&self) -> u64;

    /// Perform a check and return the action to take
    ///
    /// This method is called periodically according to the monitor's interval.
    /// It should check the current system state and return an action if needed.
    ///
    /// # Returns
    ///
    /// - `Ok(MonitorAction::None)`: No change detected
    /// - `Ok(MonitorAction::Pause(reason))`: Condition detected, should pause
    /// - `Ok(MonitorAction::Resume(reason))`: Condition cleared, should resume
    /// - `Err(MonitorError::CheckFailed(msg))`: Transient error, will retry
    /// - `Err(MonitorError::Unavailable)`: Permanent error, will not retry
    ///
    /// # Performance
    ///
    /// This method is called frequently and should be fast. Avoid:
    /// - Blocking I/O operations
    /// - Expensive computations
    /// - Long-running system calls
    ///
    /// # Error Handling
    ///
    /// Return errors instead of panicking. The orchestrator will log errors
    /// and continue running other monitors.
    fn check(&mut self) -> Pin<Box<dyn Future<Output = MonitorResult> + Send + '_>>;

    /// Called when the monitor is first started
    ///
    /// Override this to perform any initialization needed, such as:
    /// - Opening system handles
    /// - Registering for events
    /// - Allocating resources
    ///
    /// # Default Implementation
    ///
    /// Does nothing. Most monitors don't need initialization.
    ///
    /// # Error Handling
    ///
    /// If initialization fails, the monitor should set an internal flag
    /// and return `Unavailable` from future `check()` calls.
    fn on_start(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {})
    }

    /// Called when the monitor is stopped
    ///
    /// Override this to perform any cleanup needed, such as:
    /// - Closing system handles
    /// - Unregistering from events
    /// - Freeing resources
    ///
    /// # Default Implementation
    ///
    /// Does nothing. Most monitors don't need cleanup.
    ///
    /// # Note
    ///
    /// Currently, monitors run until the application exits and `on_stop()`
    /// is never called. This method is provided for future use.
    fn on_stop(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {})
    }

    /// Whether to skip this monitor's checks during active sessions
    ///
    /// # Session Protection
    ///
    /// During an active session (break window or attention prompt), the scheduler
    /// is already interacting with the user. Environment changes detected by monitors
    /// are often **side effects** of the session itself:
    ///
    /// - **Break windows** may trigger system DND mode
    /// - **User leaving computer** during break is expected (goal of break)
    /// - **Excluded apps** may still be running during break
    ///
    /// If monitors send Pause commands during sessions, it can cause:
    /// - Self-interruption (break window triggers DND → pauses break)
    /// - Unexpected behavior (user expects break to continue)
    /// - State machine confusion (pausing an already-active session)
    ///
    /// # Default Behavior
    ///
    /// Returns `true` by default (skip checks during sessions).
    /// Most monitors should keep this default.
    ///
    /// # When to Override
    ///
    /// Only override to return `false` if the monitor needs to detect
    /// critical system conditions that should interrupt sessions
    /// (e.g., low battery, system shutdown).
    ///
    /// # Implementation Note
    ///
    /// Session checking is enforced at the orchestrator level, not in
    /// individual monitor implementations. This ensures consistent behavior
    /// and reduces code duplication.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Default: skip during sessions (most monitors)
    /// fn skip_during_session(&self) -> bool {
    ///     true
    /// }
    ///
    /// // Override: continue during sessions (rare cases)
    /// fn skip_during_session(&self) -> bool {
    ///     false
    /// }
    /// ```
    fn skip_during_session(&self) -> bool {
        true
    }
}

/// Convert a `MonitorAction` to a `Command`
///
/// This function is used by the orchestrator to convert monitor actions
/// into scheduler commands.
///
/// # Returns
///
/// - `Some(Command::Pause(reason))` for `MonitorAction::Pause`
/// - `Some(Command::Resume(reason))` for `MonitorAction::Resume`
/// - `None` for `MonitorAction::None`
///
/// # Example
///
/// ```rust,ignore
/// let action = MonitorAction::Pause(PauseReason::UserIdle);
/// let cmd = action_to_command(action);
/// ```
#[must_use]
pub fn action_to_command(action: MonitorAction) -> Option<Command> {
    match action {
        MonitorAction::None => None,
        MonitorAction::Pause(reason) => Some(Command::Pause(reason)),
        MonitorAction::Resume(reason) => Some(Command::Resume(reason)),
    }
}
