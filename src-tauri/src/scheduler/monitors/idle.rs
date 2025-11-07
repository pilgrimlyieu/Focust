//! User idle time monitoring
//!
//! This module provides the `IdleMonitor` which detects when the user has been
//! inactive for a configured period and triggers scheduler pause/resume actions.
//!
//! # Platform Support
//!
//! Idle detection uses the `user-idle2` crate which supports:
//! - **Windows**: `GetLastInputInfo` API
//! - **macOS**: Core Graphics `CGEventSourceSecondsSinceLastEventType`
//! - **Linux**: X11 `XScreenSaverQueryInfo` or Wayland equivalent
//!
//! # Behavior
//!
//! - **Pauses** scheduler when user has been idle for >= threshold
//! - **Resumes** scheduler when user becomes active again
//! - **Self-disables** after 3 consecutive detection failures
//!
//! # Configuration
//!
//! - `inactive_threshold_s`: Seconds of inactivity before triggering pause
//! - `INTERVAL_SECS`: How often to check (10 seconds)
//!
//! # Error Handling
//!
//! If idle detection fails repeatedly (e.g., platform not supported, permissions issue),
//! the monitor disables itself after `MAX_CONSECUTIVE_FAILURES` to avoid spam.

use std::future::Future;
use std::pin::Pin;

use user_idle2::UserIdle;

use super::{Monitor, MonitorAction, MonitorError, MonitorResult};
use crate::scheduler::models::PauseReason;

/// How often to check for idle state (seconds)
const INTERVAL_SECS: u64 = 10;

/// Maximum consecutive failures before self-disabling
const MAX_CONSECUTIVE_FAILURES: u32 = 3;

/// Monitor that tracks user idle time
///
/// This monitor checks system idle time periodically and triggers
/// pause/resume actions based on a configurable threshold.
///
/// # State Machine
///
/// ```text
/// Active (was_idle = false)
///     ↓ idle_time >= threshold
/// Idle (was_idle = true) → Pause(UserIdle)
///     ↓ idle_time < threshold
/// Active (was_idle = false) → Resume(UserIdle)
/// ```
///
/// # Example
///
/// ```rust,ignore
/// // Pause after 2 minutes of inactivity
/// let monitor = IdleMonitor::new(120);
/// ```
pub struct IdleMonitor {
    /// Idle threshold in seconds
    ///
    /// User must be idle for at least this long before triggering a pause.
    inactive_threshold_s: u32,

    /// Whether user was idle in last check
    ///
    /// Used to detect state transitions (active ↔ idle).
    was_idle: bool,

    /// Number of consecutive check failures
    ///
    /// Incremented on each failed idle detection attempt.
    /// Reset to 0 on successful check.
    consecutive_failures: u32,

    /// Whether the monitor has been disabled due to repeated failures
    ///
    /// Set to true after [`MAX_CONSECUTIVE_FAILURES`] consecutive failures.
    /// Once disabled, the monitor returns `None` for all checks.
    disabled: bool,
}

impl IdleMonitor {
    /// Create a new idle monitor with the given threshold
    ///
    /// # Arguments
    ///
    /// * `inactive_threshold_s` - Seconds of inactivity before pausing (e.g., 120 for 2 minutes)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Pause after 5 minutes of inactivity
    /// let monitor = IdleMonitor::new(300);
    /// ```
    #[must_use]
    pub fn new(inactive_threshold_s: u32) -> Self {
        Self {
            inactive_threshold_s,
            was_idle: false,
            consecutive_failures: 0,
            disabled: false,
        }
    }

    /// Update the idle threshold
    ///
    /// This allows changing the threshold without recreating the monitor.
    /// The change takes effect on the next check.
    ///
    /// # Arguments
    ///
    /// * `inactive_threshold_s` - New threshold in seconds
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// monitor.update_threshold(180);  // Change to 3 minutes
    /// ```
    pub fn update_threshold(&mut self, inactive_threshold_s: u32) {
        self.inactive_threshold_s = inactive_threshold_s;
    }
}

impl Monitor for IdleMonitor {
    fn name(&self) -> &'static str {
        "IdleMonitor"
    }

    fn interval(&self) -> u64 {
        INTERVAL_SECS
    }

    fn check(&mut self) -> Pin<Box<dyn Future<Output = MonitorResult> + Send + '_>> {
        Box::pin(async move {
            // Skip if disabled due to repeated failures
            if self.disabled {
                return Ok(MonitorAction::None);
            }

            let idle_duration = UserIdle::get_time().map_err(|e| {
                self.consecutive_failures += 1;

                if self.consecutive_failures == 1 {
                    // Log error only on first failure
                    tracing::warn!(
                        "Failed to get user idle time: {e}. \
                        This is normal if running in a headless environment or without X11 Screen Saver extension."
                    );
                }

                if self.consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    tracing::warn!(
                        "User idle detection failed {} times. Disabling idle monitoring. \
                        The application will continue to work without idle detection.",
                        self.consecutive_failures
                    );
                    self.disabled = true;
                }

                MonitorError::CheckFailed(e.to_string())
            })?;

            self.consecutive_failures = 0;

            let idle_seconds = idle_duration.as_seconds();
            let is_idle = idle_seconds >= u64::from(self.inactive_threshold_s);

            if is_idle && !self.was_idle {
                // User became idle
                tracing::info!(
                    "User became idle (idle for {idle_seconds}s, threshold: {}s)",
                    self.inactive_threshold_s
                );
                self.was_idle = true;
                Ok(MonitorAction::Pause(PauseReason::UserIdle))
            } else if !is_idle && self.was_idle {
                // User became active
                tracing::info!("User became active");
                self.was_idle = false;
                Ok(MonitorAction::Resume(PauseReason::UserIdle))
            } else {
                Ok(MonitorAction::None)
            }
        })
    }

    fn on_start(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {
            tracing::debug!(
                "IdleMonitor started with threshold: {}s",
                self.inactive_threshold_s
            );
        })
    }
}
