/// Monitor for user idle time
///
/// Checks periodically if the user has been idle for longer than configured threshold,
/// and triggers pause/resume actions accordingly.
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use super::{Monitor, MonitorAction, MonitorError, MonitorResult};
use crate::scheduler::models::PauseReason;

const CHECK_INTERVAL: Duration = Duration::from_secs(10);
const MAX_CONSECUTIVE_FAILURES: u32 = 3;

/// Monitor that tracks user idle time
pub struct IdleMonitor {
    /// Idle threshold in seconds
    inactive_threshold_s: u32,
    /// Whether user was idle in last check
    was_idle: bool,
    /// Number of consecutive check failures
    consecutive_failures: u32,
    /// Whether the monitor has been disabled due to repeated failures
    disabled: bool,
}

impl IdleMonitor {
    /// Create a new idle monitor with the given threshold
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
    pub fn update_threshold(&mut self, inactive_threshold_s: u32) {
        self.inactive_threshold_s = inactive_threshold_s;
    }
}

impl Monitor for IdleMonitor {
    fn name(&self) -> &'static str {
        "IdleMonitor"
    }

    fn interval(&self) -> Duration {
        CHECK_INTERVAL
    }

    fn check(&mut self) -> Pin<Box<dyn Future<Output = MonitorResult> + Send + '_>> {
        Box::pin(async move {
            // Skip if disabled due to repeated failures
            if self.disabled {
                return Ok(MonitorAction::None);
            }

            match user_idle::UserIdle::get_time() {
                Ok(idle_duration) => {
                    // Reset failure counter on success
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
                }
                Err(e) => {
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

                    Err(MonitorError::CheckFailed(e.to_string()))
                }
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
