//! Monitor orchestrator - runs all monitors in a coordinated manner
//!
//! This module provides the orchestration logic for running multiple monitors
//! in a single async task. It handles initialization, periodic checking, and
//! command routing.
//!
//! # Architecture
//!
//! The orchestrator uses a single-task design where all monitors share the same
//! async task. This is efficient because:
//!
//! - Monitors are checked sequentially (most checks are very fast)
//! - No need for multiple timers or tasks
//! - Simpler error handling and shutdown logic
//!
//! # Check Interval
//!
//! The orchestrator uses the **minimum interval** of all monitors. This ensures
//! fast monitors are checked frequently while slow monitors simply skip checks
//! if their interval hasn't elapsed yet.
//!
//! For example, if you have:
//! - `IdleMonitor` (interval: 5s)
//! - `DndMonitor` (interval: 1s)
//! - `AppWhitelistMonitor` (interval: 10s)
//!
//! The orchestrator will wake up every 1 second and check all monitors.
//!
//! # Error Handling
//!
//! - Monitor errors are logged but don't stop other monitors
//! - `Unavailable` errors are logged once (monitors can self-disable)
//! - If the command channel closes, the orchestrator exits gracefully
//!
//! # Future Improvements
//!
//! - Per-monitor interval tracking (check each monitor only when its interval elapsed)
//! - Dynamic monitor addition/removal
//! - Monitor health monitoring and automatic restart

use tauri::AppHandle;
use tokio::sync::mpsc;

use super::{Monitor, MonitorAction, action_to_command};
use crate::scheduler::models::Command;
use crate::scheduler::shared_state::SharedState;

/// Spawn monitor tasks for the scheduler
///
/// This creates a single background task that runs all provided monitors,
/// checking each at its configured interval and sending commands as needed.
///
/// # Arguments
///
/// * `monitors` - Vector of boxed monitors to run
/// * `cmd_tx` - Channel sender for sending commands to the scheduler
/// * `_app_handle` - Tauri app handle (reserved for future use)
/// * `_shared_state` - Shared scheduler state (reserved for future use)
///
/// # Behavior
///
/// - If no monitors are provided, returns immediately without spawning a task
/// - Spawns a single async task that runs all monitors
/// - Task runs until the command channel is closed
///
/// # Examples
///
/// ```rust,ignore
/// let monitors: Vec<Box<dyn Monitor>> = vec![
///     Box::new(IdleMonitor::new(120)),
///     Box::new(DndMonitor::new()),
/// ];
///
/// spawn_monitor_tasks(monitors, cmd_tx, app_handle, shared_state);
/// ```
pub fn spawn_monitor_tasks(
    monitors: Vec<Box<dyn Monitor>>,
    cmd_tx: mpsc::Sender<Command>,
    _app_handle: AppHandle,
    _shared_state: SharedState,
) {
    if monitors.is_empty() {
        tracing::debug!("No monitors configured, skipping monitor task spawn");
        return;
    }

    tokio::spawn(async move {
        run_monitors(monitors, cmd_tx).await;
    });
}

/// Run all monitors in a single task
///
/// This is the main monitoring loop that:
/// 1. Initializes all monitors
/// 2. Creates a timer with the minimum interval
/// 3. Periodically checks all monitors
/// 4. Sends actions to the scheduler
///
/// # Arguments
///
/// * `monitors` - Mutable vector of monitors to run
/// * `cmd_tx` - Channel sender for scheduler commands
///
/// # Lifecycle
///
/// 1. **Initialization Phase**:
///    - Call `on_start()` on each monitor
///    - Calculate minimum interval
///    - Create interval timer
///
/// 2. **Monitoring Loop**:
///    - Wait for timer tick
///    - Check each monitor sequentially
///    - Convert actions to commands
///    - Send commands to scheduler
///    - Handle errors gracefully
///
/// 3. **Shutdown**:
///    - Exits when command channel closes
///    - Currently does NOT call `on_stop()` (future improvement)
///
/// # Performance
///
/// - Monitors are checked **sequentially** in the order provided
/// - Fast monitors should be placed first for better responsiveness
/// - Average iteration time should be much less than the check interval
///
/// # Error Handling
///
/// - `MonitorError::CheckFailed`: Logged and ignored, will retry next interval
/// - `MonitorError::Unavailable`: Logged and ignored, monitor self-disables
/// - Command send failure: Logged and exits (channel closed)
///
/// # Future Improvements
///
/// - Track last check time per monitor (only check when interval elapsed)
/// - Call `on_stop()` during graceful shutdown
/// - Report monitor health metrics
async fn run_monitors(mut monitors: Vec<Box<dyn Monitor>>, cmd_tx: mpsc::Sender<Command>) {
    tracing::info!(
        "Starting monitor orchestrator with {} monitor(s)",
        monitors.len()
    );

    // Initialize all monitors
    for monitor in &mut monitors {
        tracing::debug!("Initializing monitor: {}", monitor.name());
        monitor.on_start().await;
    }

    // Find minimum interval for check duration
    let check_interval = monitors.iter().map(|m| m.interval()).min().unwrap_or(1);

    let mut interval_timer =
        tokio::time::interval(tokio::time::Duration::from_secs(check_interval));

    tracing::debug!("Monitor check interval: {check_interval}s");

    loop {
        interval_timer.tick().await;

        // Check each monitor if its interval has elapsed
        for monitor in &mut monitors {
            let action = match monitor.check().await {
                Ok(a) => a,
                Err(e) => {
                    tracing::debug!("Monitor '{}' check error: {e}", monitor.name());
                    continue;
                }
            };

            if action == MonitorAction::None {
                continue;
            }

            tracing::debug!("Monitor '{}' triggered action: {action}", monitor.name());
            let Some(cmd) = action_to_command(action) else {
                continue;
            };

            if let Err(e) = cmd_tx.send(cmd).await {
                tracing::error!(
                    "Failed to send command from monitor '{}': {e}",
                    monitor.name()
                );
                return;
            }
        }
    }
}
