//! Do Not Disturb (DND) monitoring system
//!
//! This module provides cross-platform DND state monitoring with event-driven
//! implementations for all supported platforms:
//!
//! - **Windows**: Event-driven via WNF (Windows Notification Facility)
//! - **Linux**: Event-driven via D-Bus (KDE, GNOME, XFCE, etc.)
//! - **macOS**: Polling-based (can be upgraded to event-driven in future)
//!
//! All platforms use a unified interface to emit `DndEvent` when DND status changes.

pub mod models;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

pub use models::*;

use anyhow::Result;
use tokio::sync::mpsc;

pub const INTERVAL_SECS: u64 = 10;

/// Platform-agnostic DND monitor
pub struct DndMonitor {
    #[cfg(target_os = "windows")]
    platform: windows::WindowsDndMonitor,

    #[cfg(target_os = "linux")]
    platform: linux::LinuxDndMonitor,

    #[cfg(target_os = "macos")]
    platform: macos::MacosDndMonitor,
}

impl DndMonitor {
    /// Create a new DND monitor with the given enabled state
    ///
    /// # Arguments
    /// * `enabled` - Whether DND monitoring should be enabled
    pub fn new() -> Result<Self> {
        #[cfg(target_os = "windows")]
        let platform = windows::WindowsDndMonitor::new()?;

        #[cfg(target_os = "linux")]
        let platform = linux::LinuxDndMonitor::new()?;

        #[cfg(target_os = "macos")]
        let platform = macos::MacosDndMonitor::new()?;

        Ok(Self { platform })
    }

    /// Start monitoring DND status changes
    ///
    /// This will spawn a background task that monitors the system DND state
    /// and sends events through the provided channel when changes occur.
    pub async fn start(&mut self, sender: mpsc::Sender<DndEvent>) -> Result<()> {
        self.platform.start(sender).await
    }

    /// Stop monitoring DND status
    pub async fn stop(&mut self) -> Result<()> {
        self.platform.stop().await
    }

    /// Get the current DND status
    ///
    /// This is useful for initialization or fallback scenarios.
    pub async fn is_enabled(&self) -> Result<bool> {
        self.platform.is_enabled().await
    }
}
