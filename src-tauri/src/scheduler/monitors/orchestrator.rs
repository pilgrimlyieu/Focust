/// Monitor orchestrator that manages multiple monitors
///
/// This module provides functionality to run multiple monitors concurrently,
/// each checking system state at their own interval and sending commands
/// to the scheduler as needed.
use std::time::Duration;

use tauri::AppHandle;
use tokio::sync::mpsc;
use tokio::time::sleep;

use super::{Monitor, MonitorAction, action_to_command};
use crate::scheduler::models::Command;

/// Spawn monitor tasks for the scheduler
///
/// This creates a single background task that runs all provided monitors,
/// checking each at its configured interval and sending commands as needed.
pub fn spawn_monitor_tasks(
    monitors: Vec<Box<dyn Monitor>>,
    cmd_tx: mpsc::Sender<Command>,
    _app_handle: AppHandle,
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

    // Track last check time for each monitor
    let mut last_check_times: Vec<tokio::time::Instant> = monitors
        .iter()
        .map(|_| tokio::time::Instant::now())
        .collect();

    // Find minimum interval for sleep duration
    let check_interval = monitors
        .iter()
        .map(|m| m.interval())
        .min()
        .unwrap_or(Duration::from_secs(1));

    tracing::debug!("Monitor check interval: {check_interval:?}");

    loop {
        let now = tokio::time::Instant::now();

        // Check each monitor if its interval has elapsed
        for (i, monitor) in monitors.iter_mut().enumerate() {
            let elapsed = now.duration_since(last_check_times[i]);

            if elapsed >= monitor.interval() {
                last_check_times[i] = now;

                match monitor.check().await {
                    Ok(action) => {
                        if action != MonitorAction::None {
                            tracing::debug!(
                                "Monitor '{}' triggered action: {action}",
                                monitor.name()
                            );

                            if let Some(cmd) = action_to_command(action)
                                && let Err(e) = cmd_tx.send(cmd).await
                            {
                                tracing::error!(
                                    "Failed to send command from monitor '{}': {e}",
                                    monitor.name()
                                );
                                // Channel closed, exit
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::debug!("Monitor '{}' check error: {e}", monitor.name());
                    }
                }
            }
        }

        // Sleep for the check interval
        sleep(check_interval).await;
    }
}