use tauri::AppHandle;
use tokio::sync::{mpsc, watch};

use super::attention_timer::AttentionTimer;
use super::break_scheduler::BreakScheduler;
use super::models::Command;
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
                        Command::UpdateConfig(_) | Command::Pause(_) | Command::Resume(_) => {
                            // These commands affect both schedulers
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

/// Public API function to initialize the scheduler (for backward compatibility)
#[must_use]
pub fn init_scheduler(app_handle: &AppHandle) -> (mpsc::Sender<Command>, watch::Sender<()>) {
    SchedulerManager::init(app_handle)
}
