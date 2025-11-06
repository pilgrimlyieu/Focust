use tauri::AppHandle;
use tokio::sync::mpsc;

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

    // Find minimum interval for check duration
    let check_interval = monitors.iter().map(|m| m.interval()).min().unwrap_or(1);

    let mut interval_timer =
        tokio::time::interval(tokio::time::Duration::from_secs(check_interval));

    tracing::debug!("Monitor check interval: {check_interval}s");

    loop {
        interval_timer.tick().await;

        // Check each monitor if its interval has elapsed
        for monitor in &mut monitors {
            match monitor.check().await {
                Ok(action) => {
                    if action != MonitorAction::None {
                        tracing::debug!("Monitor '{}' triggered action: {action}", monitor.name());

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
}
