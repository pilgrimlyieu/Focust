//! macOS DND monitoring via polling
//!
//! This implementation uses the `defaults` command to poll the Focus Mode
//! status from system preferences. While not as efficient as event-driven
//! monitoring, it uses adaptive polling to minimize CPU usage.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context, Result};
use tokio::sync::mpsc;

use super::DndEvent;
use crate::platform::dnd::INTERVAL_SECS;

/// macOS DND monitor using polling
pub struct MacosDndMonitor {
    is_monitoring: Arc<AtomicBool>,
    last_state: Arc<AtomicBool>,
}

impl MacosDndMonitor {
    /// Creates a new macOS DND monitor.
    ///
    /// # Errors
    ///
    /// This function does not return errors in normal operation.
    pub fn new() -> Result<Self> {
        Ok(Self {
            is_monitoring: Arc::new(AtomicBool::new(false)),
            last_state: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Starts monitoring Focus Mode status.
    ///
    /// Uses polling to periodically check the Focus Mode state.
    ///
    /// # Errors
    ///
    /// Returns an error if getting the initial Focus Mode state fails.
    pub async fn start(&mut self, sender: mpsc::Sender<DndEvent>) -> Result<()> {
        if self.is_monitoring.load(Ordering::Acquire) {
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
        self.last_state.store(initial_state, Ordering::Relaxed);

        // Start polling loop
        let last_state = self.last_state.clone();

        tokio::spawn(async move {
            if let Err(e) = poll_focus_mode(sender, last_state).await {
                tracing::error!("macOS Focus Mode polling terminated with error: {e}");
            }
        });

        self.is_monitoring.store(true, Ordering::Release);
        tracing::info!("macOS DND monitoring started successfully");
        Ok(())
    }

    /// Stops monitoring Focus Mode status.
    ///
    /// # Errors
    ///
    /// This function does not return errors in normal operation.
    pub async fn stop(&mut self) -> Result<()> {
        if !self.is_monitoring.load(Ordering::Acquire) {
            return Ok(());
        }

        tracing::info!("Stopping macOS DND monitoring");
        self.is_monitoring.store(false, Ordering::Release);
        Ok(())
    }

    /// Gets the current Focus Mode status.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Executing the `defaults` command fails
    /// - Parsing the command output fails
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
#[expect(clippy::unnecessary_wraps)]
async fn poll_focus_mode(
    sender: mpsc::Sender<DndEvent>,
    last_state: Arc<AtomicBool>,
) -> Result<()> {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(INTERVAL_SECS));
    loop {
        interval.tick().await;

        match check_focus_mode_status().await {
            Ok(current_state) => {
                let last = last_state.load(Ordering::Relaxed);

                // Emit event if state changed
                if last != current_state {
                    last_state.store(current_state, Ordering::Relaxed);

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
                interval =
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
