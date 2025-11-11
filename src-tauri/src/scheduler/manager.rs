use tauri::{AppHandle, Emitter, Runtime};
use tokio::sync::{mpsc, watch};

use super::attention_timer::AttentionTimer;
use super::break_scheduler::BreakScheduler;
use super::event_emitter::TauriEventEmitter;
use super::models::{Command, PauseReason, SchedulerStatus};
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
        let break_event_emitter = TauriEventEmitter::new(app_handle.clone());
        let break_shutdown_rx = shutdown_rx.clone();
        let break_shared_state = shared_state.clone();
        tokio::spawn(async move {
            let mut scheduler = BreakScheduler::new(
                break_scheduler_handle,
                break_event_emitter,
                break_shutdown_rx,
                break_shared_state,
            );
            scheduler.run(break_cmd_rx).await;
        });

        // Spawn attention timer
        let attention_timer_handle = app_handle.clone();
        let attention_event_emitter = TauriEventEmitter::new(app_handle.clone());
        let attention_shutdown_rx = shutdown_rx.clone();
        let attention_shared_state = shared_state.clone();
        tokio::spawn(async move {
            let mut timer = AttentionTimer::new(
                attention_timer_handle,
                attention_event_emitter,
                attention_shutdown_rx,
                attention_shared_state,
            );
            timer.run(attention_cmd_rx).await;
        });

        // Spawn command broadcaster
        let router_shutdown_rx = shutdown_rx.clone();
        let router_shared_state = shared_state.clone();
        let router_app_handle = app_handle.clone();
        tokio::spawn(async move {
            broadcast_commands(
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
}

/// Broadcast incoming commands to appropriate schedulers
///
/// # Command Processing Architecture
///
/// This is the central command router that coordinates all scheduler communication.
/// Commands are categorized into three types:
///
/// ## 1. Global Commands (Processed + Forwarded)
///
/// These commands affect global state and all schedulers:
/// - **Pause(reason)**: Updates [`SharedState`], forwards to all schedulers
/// - **Resume(reason)**: Updates [`SharedState`], forwards only if all reasons cleared
///
/// Flow: Command → Update [`SharedState`] → Forward to schedulers → Emit events
///
/// ## 2. Broadcast Commands (Forwarded to All)
///
/// These commands are sent to all schedulers for processing:
/// - **`UpdateConfig`**: All schedulers recalculate next events
///
/// Flow: Command → Forward to all schedulers
///
/// ## 3. Targeted Commands (Routed to Specific Scheduler)
///
/// These commands are routed based on event type or functionality:
/// - **TriggerEvent(event)**: Routed by event type (Break → [`BreakScheduler`], Attention → [`AttentionTimer`])
/// - **PromptFinished(event)**: Routed by event type
/// - **PostponeBreak/SkipBreak/RequestBreakStatus**: Only to [`BreakScheduler`]
///
/// Flow: Command → Pattern match → Forward to appropriate scheduler
///
/// # State Management
///
/// - **[`SharedState`]**: Single source of truth for pause reasons and sessions
/// - **Schedulers**: Implement business logic and internal state machines
/// - **Manager**: Coordinates state updates and command routing
pub(crate) async fn broadcast_commands<R: Runtime>(
    mut cmd_rx: mpsc::Receiver<Command>,
    break_cmd_tx: mpsc::Sender<Command>,
    attention_cmd_tx: mpsc::Sender<Command>,
    mut shutdown_rx: watch::Receiver<()>,
    shared_state: SharedState,
    app_handle: AppHandle<R>,
) {
    loop {
        tokio::select! {
            biased;
            _ = shutdown_rx.changed() => {
                tracing::info!("Command broadcaster shutting down");
                break;
            }
            Some(cmd) = cmd_rx.recv() => {
                tracing::debug!("Routing command: {cmd}");

                match cmd {
                    // === GLOBAL COMMANDS: Process + Forward ===

                    Command::Pause(reason) => {
                        handle_pause_command(
                            reason,
                            &shared_state,
                            &break_cmd_tx,
                            &attention_cmd_tx,
                            &app_handle,
                        ).await;
                    }

                    Command::Resume(reason) => {
                        handle_resume_command(
                            reason,
                            &shared_state,
                            &break_cmd_tx,
                            &attention_cmd_tx,
                            &app_handle,
                        ).await;
                    }

                    // === BROADCAST COMMANDS: Forward to All ===

                    Command::UpdateConfig(_) => {
                        tracing::debug!("Broadcasting UpdateConfig to all schedulers");
                        let _ = break_cmd_tx.send(cmd.clone()).await;
                        let _ = attention_cmd_tx.send(cmd).await;
                    }

                    // === TARGETED COMMANDS: Route by Event Type ===

                    Command::TriggerEvent(event) | Command::PromptFinished(event)  => {
                        route_event_command(cmd, event, &break_cmd_tx, &attention_cmd_tx).await;
                    }

                    // === BREAK-SPECIFIC COMMANDS ===

                    Command::RequestBreakStatus | Command::PostponeBreak | Command::SkipBreak => {
                        tracing::debug!("Forwarding break-specific command to BreakScheduler");
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

/// Handle Pause command: Update `SharedState` and forward if needed
///
/// This implements the "add pause reason" logic:
/// - If first pause reason → forward to schedulers (trigger pause)
/// - If additional reason → only update `SharedState` (already paused)
async fn handle_pause_command<R: Runtime>(
    reason: PauseReason,
    shared_state: &SharedState,
    break_cmd_tx: &mpsc::Sender<Command>,
    attention_cmd_tx: &mpsc::Sender<Command>,
    app_handle: &AppHandle<R>,
) {
    let should_pause = shared_state.write().add_pause_reason(reason);

    if should_pause {
        // State transition: Running → Paused
        tracing::info!("Scheduler paused (first reason: {reason})");

        // Emit events for frontend
        let status = SchedulerStatus {
            paused: true,
            next_event: None,
            mini_break_counter: 0, // Counter is not relevant when paused
        };
        let _ = app_handle.emit("scheduler-status", &status);
        let _ = app_handle.emit("scheduler-paused", ());

        // Forward to all schedulers to update their internal state
        let _ = break_cmd_tx.send(Command::Pause(reason)).await;
        let _ = attention_cmd_tx.send(Command::Pause(reason)).await;
    } else {
        // Already paused, just added another reason
        tracing::debug!("Added pause reason {reason} (already paused)");
    }
}

/// Handle Resume command: Update `SharedState` and forward if all reasons cleared
///
/// This implements the "remove pause reason" logic:
/// - If last reason removed → forward to schedulers (trigger resume)
/// - If reasons remain → only update `SharedState` (stay paused)
async fn handle_resume_command<R: Runtime>(
    reason: PauseReason,
    shared_state: &SharedState,
    break_cmd_tx: &mpsc::Sender<Command>,
    attention_cmd_tx: &mpsc::Sender<Command>,
    app_handle: &AppHandle<R>,
) {
    let should_resume = shared_state.write().remove_pause_reason(reason);

    if should_resume {
        // State transition: Paused → Running
        tracing::info!("Scheduler resumed (all pause reasons cleared)");

        // Emit resume event (schedulers will emit detailed status)
        let _ = app_handle.emit("scheduler-resumed", ());

        // Forward to all schedulers to recalculate next events
        let _ = break_cmd_tx.send(Command::Resume(reason)).await;
        let _ = attention_cmd_tx.send(Command::Resume(reason)).await;
    } else {
        // Still paused (other reasons remain)
        tracing::debug!("Removed pause reason {reason} (still paused)");
    }
}

/// Route event-based commands to appropriate scheduler
///
/// - Break events (MiniBreak/LongBreak) → `BreakScheduler`
/// - Attention events → `AttentionTimer`
async fn route_event_command(
    cmd: Command,
    event: SchedulerEvent,
    break_cmd_tx: &mpsc::Sender<Command>,
    attention_cmd_tx: &mpsc::Sender<Command>,
) {
    if matches!(
        event,
        SchedulerEvent::MiniBreak(_) | SchedulerEvent::LongBreak(_)
    ) {
        tracing::debug!("Routing {event} command to BreakScheduler");
        let _ = break_cmd_tx.send(cmd).await;
    } else if matches!(event, SchedulerEvent::Attention(_)) {
        tracing::debug!("Routing {event} command to AttentionTimer");
        let _ = attention_cmd_tx.send(cmd).await;
    }
}
