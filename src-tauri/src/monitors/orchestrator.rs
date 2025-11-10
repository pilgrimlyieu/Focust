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
/// * `shared_state` - Shared scheduler state for session checking
///
/// # Behavior
///
/// - If no monitors are provided, returns immediately without spawning a task
/// - Spawns a single async task that runs all monitors
/// - Task runs until the command channel is closed
/// - Automatically skips monitors during active sessions (if configured)
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
    shared_state: SharedState,
) {
    if monitors.is_empty() {
        tracing::debug!("No monitors configured, skipping monitor task spawn");
        return;
    }

    tokio::spawn(async move {
        run_monitors(monitors, cmd_tx, shared_state).await;
    });
}

/// Run all monitors in a single task
///
/// # Overview
///
/// This is the main monitoring loop that coordinates all environment monitors
/// (idle detection, DND status, app exclusions) and converts their actions
/// into scheduler commands.
///
/// # Integration with Scheduler System
///
/// ```text
/// Monitor.check() → MonitorAction → action_to_command() → Command
///                                                             ↓
///                                                    cmd_tx channel
///                                                             ↓
///                                              SchedulerManager.broadcast_commands()
///                                                             ↓
///                                                Update SharedState + Route to schedulers
/// ```
///
/// **Key Integration Points:**
///
/// 1. **`MonitorAction` → Command Conversion**:
///    - `MonitorAction::Pause(reason)` → `Command::Pause(reason)`
///    - `MonitorAction::Resume(reason)` → `Command::Resume(reason)`
///    - `MonitorAction::None` → No command sent
///
/// 2. **Unified Session Protection**:
///    - **Orchestrator checks** `shared_state.in_any_session()` before each monitor
///    - If in session AND monitor has `skip_during_session() == true`, skip check
///    - This prevents monitors from interfering with active break/attention sessions
///    - Example: Break window triggers DND → `DndMonitor` skipped → no self-pause
///
/// 3. **Pause Reason Tracking**:
///    - Each monitor has a unique `PauseReason` (`UserIdle`, Dnd, `AppExclusion`)
///    - Multiple reasons can coexist (managed by `SharedState`)
///    - Scheduler resumes only when all reasons cleared
///
/// # Arguments
///
/// * `monitors` - Mutable vector of monitors to run
/// * `cmd_tx` - Channel sender for scheduler commands
/// * `shared_state` - Shared scheduler state for session checking
///
/// # Lifecycle
///
/// 1. **Initialization Phase**:
///    - Call `on_start()` on each monitor (async initialization)
///    - Calculate minimum interval across all monitors
///    - Create interval timer with that period
///
/// 2. **Monitoring Loop**:
///    - Wait for timer tick
///    - Check each monitor sequentially
///    - Convert actions to commands using `action_to_command()`
///    - Send commands to scheduler via `cmd_tx`
///    - Handle errors gracefully (log but continue)
///
/// 3. **Shutdown**:
///    - Exits when command channel closes (scheduler shutdown)
///    - Currently does NOT call `on_stop()` (future improvement)
///
/// # Performance Considerations
///
/// - **Sequential Checking**: Monitors checked one after another (not parallel)
/// - **Minimum Interval**: Uses fastest monitor's interval for timer period
/// - **Fast Monitors First**: Order monitors by check speed for best responsiveness
/// - **Average Iteration**: Should be << check interval (typically ~1ms per monitor)
///
/// # Error Handling
///
/// - `MonitorError::CheckFailed`: Logged and ignored, will retry next interval
/// - `MonitorError::Unavailable`: Logged and ignored, monitor self-disables
/// - Command send failure: Logged and exits gracefully (channel closed)
///
/// # Monitor Responsibilities
///
/// Each monitor should:
/// - Return quickly from `check()` (avoid blocking)
/// - Handle its own errors (return `MonitorError` if needed)
/// - Self-disable after repeated failures (don't spam logs)
/// - Check [`SharedState`] if needed to avoid interfering with sessions
///
/// # Example Monitor Flow
///
/// ```text
/// IdleMonitor.check()
///   ↓ User idle for 120s (threshold)
/// MonitorAction::Pause(UserIdle)
///   ↓ action_to_command()
/// Command::Pause(UserIdle)
///   ↓ cmd_tx.send()
/// SchedulerManager receives command
///   ↓ Updates SharedState
///   ↓ Forwards to schedulers
/// BreakScheduler pauses (stops timers)
/// AttentionTimer pauses (stops timers)
/// ```
///
/// # See Also
///
/// - [`Monitor`] trait - Interface all monitors implement
/// - [`action_to_command()`] - Conversion function
///
/// # Future Improvements
///
/// - Per-monitor interval tracking (only check when interval elapsed)
/// - Call `on_stop()` during graceful shutdown
/// - Monitor health metrics and auto-restart
/// - Parallel monitor checking (if safe)
async fn run_monitors(
    mut monitors: Vec<Box<dyn Monitor>>,
    cmd_tx: mpsc::Sender<Command>,
    shared_state: SharedState,
) {
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
            // Skip monitors during session if they request it
            if monitor.skip_during_session() && shared_state.read().in_any_session() {
                continue;
            }

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
