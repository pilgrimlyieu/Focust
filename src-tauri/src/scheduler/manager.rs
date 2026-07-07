use chrono::{DateTime, Duration, Utc};
use std::{future, pin::Pin, time::Duration as StdDuration};
use tauri::{AppHandle, Emitter, Runtime};
use tokio::sync::{mpsc, watch};
use tokio::time::{Instant, sleep_until};

use super::attention_timer::AttentionTimer;
use super::break_scheduler::BreakScheduler;
use super::event_emitter::TauriEventEmitter;
use super::models::{Command, PauseReason, SchedulerStatus};
use super::shared_state::{SharedState, create_shared_state};
use crate::scheduler::SchedulerEvent;

type Timer = Pin<Box<dyn future::Future<Output = ()> + Send>>;

/// Interval between re-checks of the timed pause deadline.
///
/// The monotonic clock backing tokio timers may stop during a system sleep,
/// while the wall-clock deadline shown to the user keeps advancing. Re-checking
/// both clocks at this interval bounds how long a timed pause can overshoot its
/// wall-clock expiration after the system wakes up.
const TIMED_PAUSE_RECHECK_INTERVAL: StdDuration = StdDuration::from_mins(1);

/// Timer driving the expiration of a timed manual pause.
///
/// The pause expires when *either* clock reaches its deadline:
/// - the monotonic deadline honors the requested runtime duration;
/// - the wall-clock deadline (`timed_pause_until` in the shared state, checked
///   on every tick) catches up after a system sleep, during which the
///   monotonic clock may not advance.
struct TimedPauseTimer {
    /// Future awaited by the command broadcaster; pending forever when no
    /// timed pause is armed.
    sleep: Timer,
    /// Monotonic deadline of the armed timed pause.
    deadline: Option<Instant>,
}

impl TimedPauseTimer {
    /// Create a timer with no armed timed pause.
    fn inactive() -> Self {
        Self {
            sleep: Box::pin(future::pending()),
            deadline: None,
        }
    }

    /// Arm the timer to expire once `minutes` of runtime have elapsed.
    fn arm(&mut self, minutes: u32) {
        let deadline = Instant::now() + StdDuration::from_secs(u64::from(minutes) * 60);
        self.deadline = Some(deadline);
        self.sleep = Self::sleep_until_next_check(deadline);
    }

    /// Disarm the timer, e.g. when the timed pause is cleared by the user.
    fn disarm(&mut self) {
        *self = Self::inactive();
    }

    /// Handle a fired tick, returning `true` if the timed pause expired.
    ///
    /// If neither the monotonic nor the wall-clock deadline has been reached
    /// yet, the timer re-arms itself for the next re-check.
    fn on_tick(&mut self, wall_deadline: Option<DateTime<Utc>>) -> bool {
        let Some(deadline) = self.deadline else {
            // Tick fired without an armed deadline; treat as spurious.
            self.disarm();
            return false;
        };

        let monotonic_expired = Instant::now() >= deadline;
        let wall_expired = wall_deadline.is_some_and(|until| Utc::now() >= until);
        if monotonic_expired || wall_expired {
            self.disarm();
            return true;
        }

        self.sleep = Self::sleep_until_next_check(deadline);
        false
    }

    /// Sleep until the deadline or the next re-check, whichever comes first.
    fn sleep_until_next_check(deadline: Instant) -> Timer {
        let next_check = deadline.min(Instant::now() + TIMED_PAUSE_RECHECK_INTERVAL);
        Box::pin(sleep_until(next_check))
    }
}

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
/// - **`ResumeUserPauses`**: Clears all user-started pause reasons in one step
/// - **`PauseForMinutes(minutes)`**: Sets a timed manual pause and schedules expiration
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
/// - **TriggerBreakNow(kind)**: Routed to [`BreakScheduler`] for schedule selection and execution
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
    let mut timed_pause_timer = TimedPauseTimer::inactive();

    loop {
        tokio::select! {
            biased;
            _ = shutdown_rx.changed() => {
                tracing::info!("Command broadcaster shutting down");
                break;
            }
            () = &mut timed_pause_timer.sleep => {
                let wall_deadline = shared_state.read().timed_pause_until();
                if timed_pause_timer.on_tick(wall_deadline) {
                    handle_resume_command(
                        PauseReason::TimedManual,
                        &shared_state,
                        &break_cmd_tx,
                        &attention_cmd_tx,
                        &app_handle,
                    ).await;
                }
            }
            cmd = cmd_rx.recv() => {
                let Some(cmd) = cmd else {
                    tracing::info!("Command channel closed, broadcaster shutting down");
                    break;
                };

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
                        if reason == PauseReason::TimedManual {
                            timed_pause_timer.disarm();
                        }
                        handle_resume_command(
                            reason,
                            &shared_state,
                            &break_cmd_tx,
                            &attention_cmd_tx,
                            &app_handle,
                        ).await;
                    }

                    Command::ResumeUserPauses => {
                        timed_pause_timer.disarm();
                        handle_resume_user_pauses(
                            &shared_state,
                            &break_cmd_tx,
                            &attention_cmd_tx,
                            &app_handle,
                        ).await;
                    }

                    Command::PauseForMinutes(minutes) => {
                        if minutes == 0 {
                            tracing::warn!("Ignoring timed pause with zero duration");
                            continue;
                        }

                        let until = Utc::now() + Duration::minutes(i64::from(minutes));
                        shared_state.write().set_timed_pause_until(until);
                        timed_pause_timer.arm(minutes);

                        handle_pause_command(
                            PauseReason::TimedManual,
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

                    Command::RequestBreakStatus
                    | Command::PostponeBreak
                    | Command::SkipBreak
                    | Command::TriggerBreakNow(_) => {
                        tracing::debug!("Forwarding break-specific command to BreakScheduler");
                        let _ = break_cmd_tx.send(cmd).await;
                    }
                }
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
        emit_paused_status(app_handle, shared_state);
        let _ = app_handle.emit("scheduler-paused", ());

        // Forward to all schedulers to update their internal state
        let _ = break_cmd_tx.send(Command::Pause(reason)).await;
        let _ = attention_cmd_tx.send(Command::Pause(reason)).await;
    } else {
        // Already paused, just added another reason
        tracing::debug!("Added pause reason {reason} (already paused)");
        emit_paused_status(app_handle, shared_state);
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
        resume_schedulers(reason, break_cmd_tx, attention_cmd_tx, app_handle).await;
    } else {
        // Still paused (other reasons remain)
        tracing::debug!("Removed pause reason {reason} (still paused)");
        emit_paused_status(app_handle, shared_state);
    }
}

/// Handle `ResumeUserPauses` command: clear all user-started pause reasons
///
/// Removes every reason in [`PauseReason::USER_CLEARABLE`] in one step, so a
/// single user-facing "Resume" click cannot leave a stale manual or timed
/// manual pause behind. Environment-driven reasons (DND, idle, app exclusion)
/// are untouched and keep the scheduler paused if active.
async fn handle_resume_user_pauses<R: Runtime>(
    shared_state: &SharedState,
    break_cmd_tx: &mpsc::Sender<Command>,
    attention_cmd_tx: &mpsc::Sender<Command>,
    app_handle: &AppHandle<R>,
) {
    let should_resume = {
        let mut state = shared_state.write();
        let was_paused = state.is_paused();
        for reason in PauseReason::USER_CLEARABLE {
            state.remove_pause_reason(reason);
        }
        was_paused && !state.is_paused()
    };

    if should_resume {
        // Schedulers resume unconditionally, the forwarded reason is informational
        resume_schedulers(
            PauseReason::Manual,
            break_cmd_tx,
            attention_cmd_tx,
            app_handle,
        )
        .await;
    } else {
        // Still paused (environment-driven reasons remain)
        tracing::debug!("Cleared user pauses (still paused)");
        emit_paused_status(app_handle, shared_state);
    }
}

/// Complete the Paused → Running transition: emit events and forward Resume
async fn resume_schedulers<R: Runtime>(
    reason: PauseReason,
    break_cmd_tx: &mpsc::Sender<Command>,
    attention_cmd_tx: &mpsc::Sender<Command>,
    app_handle: &AppHandle<R>,
) {
    tracing::info!("Scheduler resumed (all pause reasons cleared)");

    // Emit resume event (schedulers will emit detailed status)
    let _ = app_handle.emit("scheduler-resumed", ());

    // Forward to all schedulers to recalculate next events
    let _ = break_cmd_tx.send(Command::Resume(reason)).await;
    let _ = attention_cmd_tx.send(Command::Resume(reason)).await;
}

/// Emit a `scheduler-status` snapshot of the current pause state
///
/// Used whenever the pause reason set changes without a Running/Paused
/// transition, and for the paused half of the transition itself. The
/// mini-break counter is not tracked here; `BreakScheduler` emits statuses
/// with the real counter while running.
fn emit_paused_status<R: Runtime>(app_handle: &AppHandle<R>, shared_state: &SharedState) {
    let status = {
        let state = shared_state.read();
        SchedulerStatus {
            paused: state.is_paused(),
            pause_reasons: state.pause_reasons(),
            timed_pause_until: state.timed_pause_until_rfc3339(),
            next_event: None,
            mini_break_counter: 0,
        }
    };
    let _ = app_handle.emit("scheduler-status", &status);
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
    match event {
        SchedulerEvent::MiniBreak(_) | SchedulerEvent::LongBreak(_) => {
            tracing::debug!("Routing {event} command to BreakScheduler");
            let _ = break_cmd_tx.send(cmd).await;
        }
        SchedulerEvent::Attention(_) => {
            tracing::debug!("Routing {event} command to AttentionTimer");
            let _ = attention_cmd_tx.send(cmd).await;
        }
    }
}
