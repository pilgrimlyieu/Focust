/// Monitor for Do Not Disturb (DND) / Focus Assist mode
///
/// Checks if the system is in DND/Focus Assist mode and triggers pause/resume actions.
/// Platform-specific implementations:
/// - Windows: Focus Assist via registry
/// - macOS: Do Not Disturb via notification center
/// - Linux: Varies by desktop environment (not fully supported yet)
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use super::{Monitor, MonitorAction, MonitorError, MonitorResult};
use crate::scheduler::models::PauseReason;

const CHECK_INTERVAL: Duration = Duration::from_secs(10);

/// Monitor that tracks system DND/Focus Assist state
pub struct DndMonitor {
    /// Whether DND was active in last check
    was_dnd_active: bool,
    /// Whether the monitor is available on this platform
    available: bool,
}

impl Default for DndMonitor {
    fn default() -> Self {
        let available = Self::is_available();

        if !available {
            tracing::warn!(
                "DND monitoring is not supported on this platform or desktop environment"
            );
        }

        Self {
            was_dnd_active: false,
            available,
        }
    }
}

impl DndMonitor {
    /// Create a new DND monitor
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if DND monitoring is available on this platform
    fn is_available() -> bool {
    }

    /// Check if DND/Focus Assist is currently active
    #[allow(clippy::unnecessary_wraps)]
    fn is_dnd_active() -> Result<bool, String> {
    }

    #[cfg(target_os = "windows")]
    #[allow(clippy::unnecessary_wraps)]
    fn check_windows_focus_assist() -> Result<bool, String> {
    }

    #[cfg(target_os = "macos")]
    fn check_macos_dnd() -> Result<bool, String> {
    }

    #[cfg(target_os = "linux")]
    fn check_linux_dnd() -> Result<bool, String> {
    }
}

impl Monitor for DndMonitor {
    fn name(&self) -> &'static str {
        "DndMonitor"
    }

    fn interval(&self) -> Duration {
        CHECK_INTERVAL
    }

    fn check(&mut self) -> Pin<Box<dyn Future<Output = MonitorResult> + Send + '_>> {
        Box::pin(async move {
            // Skip if not available on this platform
            if !self.available {
                return Err(MonitorError::Unavailable);
            }

            match Self::is_dnd_active() {
                Ok(is_dnd) => {
                    if is_dnd && !self.was_dnd_active {
                        // DND was enabled
                        tracing::info!("DND/Focus Assist enabled, pausing scheduler");
                        self.was_dnd_active = true;
                        Ok(MonitorAction::Pause(PauseReason::Dnd))
                    } else if !is_dnd && self.was_dnd_active {
                        // DND was disabled
                        tracing::info!("DND/Focus Assist disabled, resuming scheduler");
                        self.was_dnd_active = false;
                        Ok(MonitorAction::Resume(PauseReason::Dnd))
                    } else {
                        Ok(MonitorAction::None)
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to check DND status: {e}");
                    Err(MonitorError::CheckFailed(e))
                }
            }
        })
    }

    fn on_start(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {
            if self.available {
                tracing::debug!("DndMonitor started");
            } else {
                tracing::debug!("DndMonitor started but unavailable on this platform");
            }
        })
    }
}
