use tauri::{AppHandle, Manager};
use tokio::sync::mpsc;
use tokio::time::sleep;
use user_idle::UserIdle;

use super::models::{Command, PauseReason};
use crate::config::SharedConfig;

/// Spawn a task to monitor user idle state and pause/resume the scheduler
pub fn spawn_idle_monitor_task(cmd_tx: mpsc::Sender<Command>, app_handle: AppHandle) {
    tracing::debug!("Spawning user idle monitor task...");

    tokio::spawn(async move {
        const CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(10);
        let mut was_idle = false;
        let inactive_s = {
            let config = app_handle.state::<SharedConfig>();
            let config_guard = config.read().await;
            config_guard.inactive_s
        };

        loop {
            if let Ok(idle_duration) = UserIdle::get_time() {
                let idle_seconds = idle_duration.as_seconds();
                let is_idle = idle_seconds >= u64::from(inactive_s);

                if is_idle && !was_idle {
                    // User became idle
                    tracing::info!(
                        "User became idle (idle for {idle_seconds}s). Pausing scheduler."
                    );
                    if let Err(e) = cmd_tx.send(Command::Pause(PauseReason::UserIdle)).await {
                        tracing::error!("Failed to send Pause command: {e}");
                        break;
                    }
                    was_idle = true;
                } else if !is_idle && was_idle {
                    // User became active
                    tracing::info!("User became active. Resuming scheduler.");
                    if let Err(e) = cmd_tx.send(Command::Resume(PauseReason::UserIdle)).await {
                        tracing::error!("Failed to send Resume command: {e}");
                        break;
                    }
                    was_idle = false;
                }
            } else {
                tracing::error!("Failed to get user idle time");
            }

            sleep(CHECK_INTERVAL).await;
        }

        tracing::info!("Idle monitor task shutting down");
    });
}
