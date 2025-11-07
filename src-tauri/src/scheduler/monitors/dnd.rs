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
use std::time::Instant;

use tokio::sync::{Mutex as AsyncMutex, mpsc};

use super::{Monitor, MonitorAction, MonitorError, MonitorResult};
use crate::platform::dnd::{DndEvent, DndMonitor as PlatformDndMonitor, INTERVAL_SECS};
use crate::scheduler::models::PauseReason;
use crate::scheduler::shared_state::SharedState;

/// Debounce delay in seconds
///
/// DND state must remain stable for this duration before being reported.
/// This filters out rapid state changes (e.g., window opening/closing causing DND flicker).
const DEBOUNCE_SECS: u64 = 3;

/// Monitor that tracks system DND/Focus Assist state using event-driven approach
pub struct DndMonitor {
    /// Platform-specific DND monitor
    platform_monitor: Option<PlatformDndMonitor>,
    /// Channel receiver for DND events
    event_rx: Arc<AsyncMutex<Option<mpsc::Receiver<DndEvent>>>>,
    /// Whether the monitor is available on this platform
    available: bool,
    /// Current DND state (from system)
    current_dnd_state: bool,
    /// Last reported stable DND state
    reported_dnd_state: bool,
    /// When the DND state last changed (for debouncing)
    state_change_time: Option<Instant>,
    /// Shared scheduler state (for session checking)
    shared_state: SharedState,
}

impl DndMonitor {
    /// Create a new DND monitor
    #[must_use]
    pub fn new(shared_state: SharedState) -> Self {
        Self {
            platform_monitor: None,
            event_rx: Arc::new(AsyncMutex::new(None)),
            available: true, // Assume available, will check on start
            current_dnd_state: false,
            reported_dnd_state: false,
            state_change_time: None,
            shared_state,
        }
    }

    /// Initialize the platform DND monitor
    async fn initialize(&mut self) -> Result<(), String> {
        let mut monitor = PlatformDndMonitor::new().map_err(|e| {
            tracing::warn!("Failed to create DND monitor: {e}");
            self.available = false;
            format!("Platform DND monitor creation failed: {e}")
        })?;

        let (tx, rx) = mpsc::channel(16);

        self.current_dnd_state = false; // Assume disabled initially
        self.reported_dnd_state = false;

        monitor.start(tx).await.map_err(|e| {
            tracing::error!("Failed to start DND monitor: {e}");
            self.available = false;
            format!("Failed to start platform DND monitor: {e}")
        })?;

        self.platform_monitor = Some(monitor);
        *self.event_rx.lock().await = Some(rx);
        self.available = true;

        tracing::info!(
            "DND monitor initialized successfully (initial state: {})",
            if self.current_dnd_state {
                "active"
            } else {
                "inactive"
            }
        );

        Ok(())
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

            // Check if in any session (break or attention)
            // During sessions, we ignore DND changes to prevent self-triggering
            if self.shared_state.read().in_any_session() {
                // In session, ignore DND events but drain the channel
                let mut event_rx_guard = self.event_rx.lock().await;
                if let Some(rx) = event_rx_guard.as_mut() {
                    while rx.try_recv().is_ok() {
                        // Drain events without processing
                    }
                }
                return Ok(MonitorAction::None);
            }

            // Try to receive DND events (non-blocking)
            let mut event_rx_guard = self.event_rx.lock().await;
            if let Some(rx) = event_rx_guard.as_mut() {
                // Process all pending events
                while let Ok(event) = rx.try_recv() {
                    let new_state = match event {
                        DndEvent::Started => true,
                        DndEvent::Finished => false,
                    };

                    // Detect state change
                    if new_state != self.current_dnd_state {
                        tracing::debug!(
                            "DND state changed: {} -> {}",
                            if self.current_dnd_state {
                                "enabled"
                            } else {
                                "disabled"
                            },
                            if new_state { "enabled" } else { "disabled" }
                        );

                        self.current_dnd_state = new_state;
                        self.state_change_time = Some(Instant::now());
                    }
                }

                // Check for channel disconnection
                if rx.is_closed() {
                    tracing::error!("DND event channel disconnected");
                    self.available = false;
                    return Err(MonitorError::CheckFailed(
                        "Event channel disconnected".to_string(),
                    ));
                }
            }

            // Check if state has been stable long enough (debouncing)
            if let Some(change_time) = self.state_change_time
                && change_time.elapsed().as_secs() >= DEBOUNCE_SECS
            {
                // State has stabilized, check if we need to report it
                if self.current_dnd_state != self.reported_dnd_state {
                    self.reported_dnd_state = self.current_dnd_state;
                    self.state_change_time = None; // Reset

                    return if self.current_dnd_state {
                        tracing::info!("DND enabled (stable), pausing scheduler");
                        Ok(MonitorAction::Pause(PauseReason::Dnd))
                    } else {
                        tracing::info!("DND disabled (stable), resuming scheduler");
                        Ok(MonitorAction::Resume(PauseReason::Dnd))
                    };
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
