/// Monitor for Do Not Disturb (DND) / Focus Assist mode
///
/// Uses event-driven monitoring where available for zero-polling performance:
/// - **Windows**: Event-driven via WNF (Windows Notification Facility)
/// - **Linux**: Event-driven via D-Bus (KDE, GNOME, XFCE, etc.)
/// - **macOS**: Adaptive polling (can be upgraded to event-driven in future)
///
/// This monitor wraps the platform-specific `DndMonitor` from the `platform::dnd` module
/// and integrates it with the scheduler's monitoring framework.
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::{Mutex as AsyncMutex, mpsc};

use super::{Monitor, MonitorAction, MonitorError, MonitorResult};
use crate::platform::dnd::{DndEvent, DndMonitor as PlatformDndMonitor, INTERVAL_SECS};
use crate::scheduler::models::PauseReason;

/// Monitor that tracks system DND/Focus Assist state using event-driven approach
pub struct DndMonitor {
    /// Platform-specific DND monitor
    platform_monitor: Option<PlatformDndMonitor>,
    /// Channel receiver for DND events
    event_rx: Arc<AsyncMutex<Option<mpsc::Receiver<DndEvent>>>>,
    /// Whether the monitor is available on this platform
    available: bool,
    /// Whether DND is currently active
    is_active: bool,
}

impl Default for DndMonitor {
    fn default() -> Self {
        Self {
            platform_monitor: None,
            event_rx: Arc::new(AsyncMutex::new(None)),
            available: true, // Assume available, will check on start
            is_active: false,
        }
    }
}

impl DndMonitor {
    /// Create a new DND monitor
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize the platform DND monitor
    async fn initialize(&mut self) -> Result<(), String> {
        match PlatformDndMonitor::new() {
            Ok(mut monitor) => {
                // Create event channel
                let (tx, rx) = mpsc::channel(16);

                // Skip initial state query - will get state from first event
                self.is_active = false; // Assume disabled initially

                // Start monitoring with error handling
                match monitor.start(tx).await {
                    Ok(()) => {
                        self.platform_monitor = Some(monitor);
                        *self.event_rx.lock().await = Some(rx);
                        self.available = true;

                        tracing::info!(
                            "DND monitor initialized successfully (initial state: {})",
                            if self.is_active { "active" } else { "inactive" }
                        );
                        Ok(())
                    }
                    Err(e) => {
                        tracing::error!("Failed to start DND monitor: {e}");
                        self.available = false;
                        Err(format!("Failed to start platform DND monitor: {e}"))
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to create DND monitor: {e}");
                self.available = false;
                Err(format!("Platform DND monitor creation failed: {e}"))
            }
        }
    }
}

impl Monitor for DndMonitor {
    fn name(&self) -> &'static str {
        "DndMonitor"
    }

    fn interval(&self) -> u64 {
        INTERVAL_SECS
    }

    fn check(&mut self) -> Pin<Box<dyn Future<Output = MonitorResult> + Send + '_>> {
        Box::pin(async move {
            // Skip if not available
            if !self.available {
                return Err(MonitorError::Unavailable);
            }

            // Try to receive DND events (non-blocking)
            let mut event_rx_guard = self.event_rx.lock().await;
            if let Some(rx) = event_rx_guard.as_mut() {
                match rx.try_recv() {
                    Ok(event) => match event {
                        DndEvent::Started => {
                            tracing::info!("DND enabled, pausing scheduler");
                            self.is_active = true;
                            return Ok(MonitorAction::Pause(PauseReason::Dnd));
                        }
                        DndEvent::Finished => {
                            tracing::info!("DND disabled, resuming scheduler");
                            self.is_active = false;
                            return Ok(MonitorAction::Resume(PauseReason::Dnd));
                        }
                    },
                    Err(mpsc::error::TryRecvError::Empty) => {
                        // No new events, continue
                    }
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        tracing::error!("DND event channel disconnected");
                        self.available = false;
                        return Err(MonitorError::CheckFailed(
                            "Event channel disconnected".to_string(),
                        ));
                    }
                }
            }

            Ok(MonitorAction::None)
        })
    }

    fn on_start(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {
            match self.initialize().await {
                Ok(()) => {
                    tracing::info!("DndMonitor started successfully");
                }
                Err(e) => {
                    tracing::warn!(
                        "DndMonitor failed to start: {e}. DND monitoring will be disabled."
                    );
                    tracing::warn!("This is not a critical error and won't affect other features.");
                    self.available = false;
                }
            }
        })
    }
}
