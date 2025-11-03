use tauri::{AppHandle, Manager};
use tokio::sync::{mpsc, watch};
use tokio::time::sleep;
use user_idle::UserIdle;

use super::attention_timer::AttentionTimer;
use super::break_scheduler::BreakScheduler;
use super::models::{Command, PauseReason};
use crate::config::SharedConfig;
use crate::scheduler::SchedulerEvent;

/// Top-level scheduler manager that coordinates break scheduling and attention timers
pub struct SchedulerManager;

impl SchedulerManager {
    /// Initialize and start the scheduler system
    pub fn init(app_handle: &AppHandle) -> (mpsc::Sender<Command>, watch::Sender<()>) {
        let (cmd_tx, cmd_rx) = mpsc::channel::<Command>(32);
        let (shutdown_tx, shutdown_rx) = watch::channel(());

        // Create separate channels for each scheduler
        let (break_cmd_tx, break_cmd_rx) = mpsc::channel::<Command>(32);
        let (attention_cmd_tx, attention_cmd_rx) = mpsc::channel::<Command>(32);

        // Spawn break scheduler
        let break_scheduler_handle = app_handle.clone();
        let break_shutdown_rx = shutdown_rx.clone();
        tokio::spawn(async move {
            let mut scheduler = BreakScheduler::new(break_scheduler_handle, break_shutdown_rx);
            scheduler.run(break_cmd_rx).await;
        });

        // Spawn attention timer
        let attention_timer_handle = app_handle.clone();
        let attention_shutdown_rx = shutdown_rx.clone();
        tokio::spawn(async move {
            let mut timer = AttentionTimer::new(attention_timer_handle, attention_shutdown_rx);
            timer.run(attention_cmd_rx).await;
        });

        // Spawn idle monitor
        spawn_idle_monitor_task(cmd_tx.clone(), app_handle.clone());

        // Spawn command broadcaster
        let router_shutdown_rx = shutdown_rx.clone();
        tokio::spawn(async move {
            Self::broadcast_commands(cmd_rx, break_cmd_tx, attention_cmd_tx, router_shutdown_rx)
                .await;
        });

        tracing::info!("SchedulerManager initialized");
        (cmd_tx, shutdown_tx)
    }

    /// Broadcast incoming commands to appropriate schedulers
    /// Some commands go to both schedulers, some only to one
    async fn broadcast_commands(
        mut cmd_rx: mpsc::Receiver<Command>,
        break_cmd_tx: mpsc::Sender<Command>,
        attention_cmd_tx: mpsc::Sender<Command>,
        mut shutdown_rx: watch::Receiver<()>,
    ) {
        loop {
            tokio::select! {
                biased;
                _ = shutdown_rx.changed() => {
                    tracing::info!("Command broadcaster shutting down");
                    break;
                }
                Some(cmd) = cmd_rx.recv() => {
                    tracing::debug!("Broadcasting command: {cmd}");

                    // Determine which scheduler(s) should receive this command
                    match &cmd {
                        Command::UpdateConfig(_) => {
                            // Both schedulers need config updates
                            let _ = break_cmd_tx.send(cmd.clone()).await;
                            let _ = attention_cmd_tx.send(cmd).await;
                        }
                        Command::TriggerEvent(SchedulerEvent::Attention(_)) => {
                            // Attention-specific
                            let _ = attention_cmd_tx.send(cmd).await;
                        }
                        Command::TriggerEvent(_) => {
                            // Break-specific
                            let _ = break_cmd_tx.send(cmd).await;
                        }
                        // All other commands are break-specific
                        _ => {
                            let _ = break_cmd_tx.send(cmd).await;
                        }
                    }
                }
                else => {
                    tracing::info!("Command channel closed, broadcaster shutting down");
                    break;
                }
            }
        }
    }
}

/// Spawn a task to monitor user idle state and pause/resume the scheduler
fn spawn_idle_monitor_task(cmd_tx: mpsc::Sender<Command>, app_handle: AppHandle) {
    tracing::debug!("Spawning user idle monitor task...");

    tokio::spawn(async move {
        const CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(10);
        let mut was_idle = false;

        loop {
            let inactive_s = {
                let config = app_handle.state::<SharedConfig>();
                let config_guard = config.read().await;
                config_guard.inactive_s
            };

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

/// Public API function to initialize the scheduler (for backward compatibility)
#[must_use]
pub fn init_scheduler(app_handle: &AppHandle) -> (mpsc::Sender<Command>, watch::Sender<()>) {
    SchedulerManager::init(app_handle)
}
