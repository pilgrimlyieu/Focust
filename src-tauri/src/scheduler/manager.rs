use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, watch};

use super::attention_timer::AttentionTimer;
use super::break_scheduler::BreakScheduler;
use super::models::Command;
use super::shared_state::{SharedState, create_shared_state};
use crate::scheduler::SchedulerEvent;

/// Top-level scheduler manager that coordinates break scheduling and attention timers
pub struct SchedulerManager;

impl SchedulerManager {
    /// Initialize and start the scheduler system
    ///
    /// Returns:
    /// - Command sender for external control
    /// - Shutdown sender for graceful shutdown
    /// - Shared scheduler state for monitors and status queries
    pub fn init(app_handle: &AppHandle) -> (mpsc::Sender<Command>, watch::Sender<()>, SharedState) {
        let (cmd_tx, cmd_rx) = mpsc::channel::<Command>(32);
        let (shutdown_tx, shutdown_rx) = watch::channel(());

        // Create shared state
        let shared_state = create_shared_state();

        // Create separate channels for each scheduler
        let (break_cmd_tx, break_cmd_rx) = mpsc::channel::<Command>(32);
        let (attention_cmd_tx, attention_cmd_rx) = mpsc::channel::<Command>(32);

        // Spawn break scheduler
        let break_scheduler_handle = app_handle.clone();
        let break_shutdown_rx = shutdown_rx.clone();
        let _break_shared_state = shared_state.clone(); // TODO: Pass to scheduler if needed
        tokio::spawn(async move {
            let mut scheduler = BreakScheduler::new(break_scheduler_handle, break_shutdown_rx);
            scheduler.run(break_cmd_rx).await;
        });

        // Spawn attention timer
        let attention_timer_handle = app_handle.clone();
        let attention_shutdown_rx = shutdown_rx.clone();
        let _attention_shared_state = shared_state.clone(); // TODO: Pass to scheduler if needed
        tokio::spawn(async move {
            let mut timer = AttentionTimer::new(attention_timer_handle, attention_shutdown_rx);
            timer.run(attention_cmd_rx).await;
        });

        // Spawn command broadcaster
        let router_shutdown_rx = shutdown_rx.clone();
        let router_shared_state = shared_state.clone();
        let router_app_handle = app_handle.clone();
        tokio::spawn(async move {
            Self::broadcast_commands(
                cmd_rx,
                break_cmd_tx,
                attention_cmd_tx,
                router_shutdown_rx,
                router_shared_state,
                router_app_handle,
            )
            .await;
        });

        tracing::info!("SchedulerManager initialized with shared state management");
        (cmd_tx, shutdown_tx, shared_state)
    }

    /// Broadcast incoming commands to appropriate schedulers
    ///
    /// This is the central command processing hub that:
    /// - Manages shared pause/resume state
    /// - Tracks session state (break/attention)
    /// - Routes commands to appropriate schedulers
    async fn broadcast_commands(
        mut cmd_rx: mpsc::Receiver<Command>,
        break_cmd_tx: mpsc::Sender<Command>,
        attention_cmd_tx: mpsc::Sender<Command>,
        mut shutdown_rx: watch::Receiver<()>,
        shared_state: SharedState,
        app_handle: AppHandle,
    ) {
        loop {
            tokio::select! {
                biased;
                _ = shutdown_rx.changed() => {
                    tracing::info!("Command broadcaster shutting down");
                    break;
                }
                Some(cmd) = cmd_rx.recv() => {
                    tracing::debug!("Processing command: {cmd}");

                    // Handle state management commands centrally
                    match cmd {
                        Command::Pause(reason) => {
                            let should_pause = shared_state.write()
                                .add_pause_reason(reason);

                            if should_pause {
                                // State transition: Running -> Paused
                                tracing::info!("Scheduler paused (reason: {reason})");
                                // Emit status change event
                                let _ = app_handle.emit("scheduler-paused", ());
                            }
                            // Don't forward to schedulers - they query shared state
                        }

                        Command::Resume(reason) => {
                            let should_resume = shared_state.write()
                                .remove_pause_reason(reason);

                            if should_resume {
                                // State transition: Paused -> Running
                                tracing::info!("Scheduler resumed (all pause reasons cleared)");
                                // Emit status change event
                                let _ = app_handle.emit("scheduler-resumed", ());
                                // Forward resume to schedulers
                                let _ = break_cmd_tx.send(Command::Resume(reason)).await;
                                let _ = attention_cmd_tx.send(Command::Resume(reason)).await;
                            }
                            // If still paused, don't forward
                        }

                        Command::TriggerEvent(event) => {
                            // Mark session start
                            if matches!(event, SchedulerEvent::MiniBreak(_) | SchedulerEvent::LongBreak(_)) {
                                {
                                    let mut state = shared_state.write();
                                    state.start_break_session();
                                }
                                let _ = break_cmd_tx.send(cmd).await;
                            } else if matches!(event, SchedulerEvent::Attention(_)) {
                                {
                                    let mut state = shared_state.write();
                                    state.start_attention_session();
                                }
                            }
                        }

                        Command::PromptFinished(event) => {
                            // Mark session end
                            if matches!(event, SchedulerEvent::MiniBreak(_) | SchedulerEvent::LongBreak(_)) {
                                {
                                    let mut state = shared_state.write();
                                    state.end_break_session();
                                }
                                let _ = break_cmd_tx.send(cmd).await;
                            } else if matches!(event, SchedulerEvent::Attention(_)) {
                                {
                                    let mut state = shared_state.write();
                                    state.end_attention_session();
                                }
                                let _ = attention_cmd_tx.send(cmd).await;
                            }
                        }

                        // Other commands forward as before
                        Command::UpdateConfig(_) => {
                            // These commands affect both schedulers
                            let _ = break_cmd_tx.send(cmd.clone()).await;
                            let _ = attention_cmd_tx.send(cmd).await;
                        }
                        Command::RequestBreakStatus | Command::PostponeBreak | Command::SkipBreak => {
                            // Break-specific commands
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
