//! macOS DND monitoring via polling
//!
//! This implementation uses the `defaults` command to poll the Focus Mode
//! status from system preferences. While not as efficient as event-driven
//! monitoring, it uses adaptive polling to minimize CPU usage.

use anyhow::{Context, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex as AsyncMutex, mpsc};

use super::{DndEvent, INTERVAL_SECS};

/// macOS DND monitor using polling
pub struct MacosDndMonitor {
    enabled: bool,
    is_monitoring: Arc<AsyncMutex<bool>>,
    last_state: Arc<AsyncMutex<bool>>,
}

impl MacosDndMonitor {
    /// Create a new macOS DND monitor
    pub fn new(enabled: bool) -> Result<Self> {
        Ok(Self {
            enabled,
            is_monitoring: Arc::new(AsyncMutex::new(false)),
            last_state: Arc::new(AsyncMutex::new(false)),
        })
    }

    /// Start monitoring Focus Mode status
    pub async fn start(&mut self, sender: mpsc::Sender<DndEvent>) -> Result<()> {
        if !self.enabled {
            tracing::info!("macOS DND monitoring is disabled");
            return Ok(());
        }

        let mut is_monitoring = self.is_monitoring.lock().await;
        if *is_monitoring {
            tracing::debug!("macOS DND monitoring is already running");
            return Ok(());
        }

        tracing::info!("Starting macOS DND monitoring with polling (interval: {INTERVAL_SECS}s)",);

        // Get initial state with error handling
        let initial_state = match self.is_enabled().await {
            Ok(state) => {
                tracing::debug!("Initial Focus Mode state: {state}",);
                state
            }
            Err(e) => {
                tracing::warn!("Failed to get initial Focus Mode state: {e}. Assuming disabled.");
                false
            }
        };
        *self.last_state.lock().await = initial_state;

        // Start polling loop
        let last_state = self.last_state.clone();

        tokio::spawn(async move {
            if let Err(e) = poll_focus_mode(sender, last_state).await {
                tracing::error!("macOS Focus Mode polling terminated with error: {e}");
            }
        });

        *is_monitoring = true;
        tracing::info!("macOS DND monitoring started successfully");
        Ok(())
    }

    /// Stop monitoring Focus Mode status
    pub async fn stop(&mut self) -> Result<()> {
        let mut is_monitoring = self.is_monitoring.lock().await;
        if !*is_monitoring {
            return Ok(());
        }

        tracing::info!("Stopping macOS DND monitoring");
        *is_monitoring = false;
        Ok(())
    }

    /// Get current Focus Mode status
    pub async fn is_enabled(&self) -> Result<bool> {
        check_focus_mode_status().await
    }
}

// ============================================================================
// Focus Mode Status Check
// ============================================================================

/// Check if Focus Mode is currently enabled
///
/// This reads the system preference using the `defaults` command.
async fn check_focus_mode_status() -> Result<bool> {
    let output = tokio::process::Command::new("defaults")
        .args(&[
            "read",
            "com.apple.controlcenter",
            "NSStatusItem Visible FocusModes",
        ])
        .output()
        .await
        .context("Failed to execute defaults command")?;

    if !output.status.success() {
        // If the key doesn't exist, Focus Mode is likely not active
        return Ok(false);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value = stdout.trim();

    // Value is "1" when Focus Mode menu item is visible (i.e., enabled)
    Ok(value == "1")
}

// ============================================================================
// Polling Loop
// ============================================================================

/// Poll Focus Mode status with optional adaptive interval
#[allow(clippy::unnecessary_wraps)]
async fn poll_focus_mode(
    sender: mpsc::Sender<DndEvent>,
    last_state: Arc<AsyncMutex<bool>>,
) -> Result<()> {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(INTERVAL_SECS));
    loop {
        interval.tick().await;

        match check_focus_mode_status().await {
            Ok(current_state) => {
                let mut last = last_state.lock().await;

                // Emit event if state changed
                if *last != current_state {
                    *last = current_state;

                    let event = if current_state {
                        DndEvent::Started
                    } else {
                        DndEvent::Finished
                    };

                    tracing::info!("macOS Focus Mode state changed: {}", event.description());

                    if let Err(e) = sender.send(event).await {
                        tracing::error!("Failed to send DND event: {e}");
                        break;
                    }
                }

                // Adaptive polling: slower when DND is active
                let interval =
                    tokio::time::interval(tokio::time::Duration::from_secs(if current_state {
                        INTERVAL_SECS * 3 // 3x slower when active
                    } else {
                        INTERVAL_SECS // Normal speed when inactive
                    }));
            }
            Err(e) => {
                tracing::debug!("Failed to check Focus Mode status: {e}");
                // Continue polling even if one check fails
            }
        }
    }

    Ok(())
}
