use std::ops::Deref;

use tauri::State;
use tokio::sync::{mpsc::Sender, watch};

use crate::scheduler::models::{Command, PauseReason, SchedulerEvent};
use crate::scheduler::shared_state::SharedState;

pub struct SchedulerCmd(pub Sender<Command>);

impl Deref for SchedulerCmd {
    type Target = Sender<Command>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SchedulerCmd {
    /// Create a new [`SchedulerCmd`]
    #[must_use]
    pub fn new(sender: Sender<Command>) -> Self {
        SchedulerCmd(sender)
    }

    /// Try to send a command to the scheduler
    pub fn try_send_command(&self, command: &Command) {
        self.0.try_send(command.clone()).unwrap_or_else(|e| {
            tracing::error!("Failed to send {command} to scheduler: {e}");
        });
    }

    /// Send a command to the scheduler asynchronously
    pub async fn send_command(&self, command: &Command) {
        self.0.send(command.clone()).await.unwrap_or_else(|e| {
            tracing::error!("Failed to send {command} to scheduler: {e}");
        });
    }
}

/// Shutdown sender to keep the scheduler alive
pub struct ShutdownTx(pub watch::Sender<()>);

/// Request the scheduler to emit its current status
#[tauri::command]
pub async fn request_break_status(state: State<'_, SchedulerCmd>) -> Result<(), String> {
    state
        .send(Command::RequestBreakStatus)
        .await
        .map_err(|e| e.to_string())
}

/// Pause the scheduler manually
///
/// Pauses the scheduler with [`PauseReason::Manual`].
#[tauri::command]
pub async fn pause_scheduler(state: State<'_, SchedulerCmd>) -> Result<(), String> {
    state
        .send(Command::Pause(PauseReason::Manual))
        .await
        .map_err(|e| e.to_string())
}

/// Resume the scheduler
#[tauri::command]
pub async fn resume_scheduler(state: State<'_, SchedulerCmd>) -> Result<(), String> {
    state
        .send(Command::Resume(PauseReason::Manual))
        .await
        .map_err(|e| e.to_string())
}

/// Postpone the current or next break
#[tauri::command]
pub async fn postpone_break(state: State<'_, SchedulerCmd>) -> Result<(), String> {
    state
        .send(Command::PostponeBreak)
        .await
        .map_err(|e| e.to_string())
}

/// Manually trigger a break for testing purposes
///
/// Returns an error if the scheduler is currently paused.
#[tauri::command]
pub async fn trigger_event(
    scheduler_cmd: State<'_, SchedulerCmd>,
    shared_state: State<'_, SharedState>,
    break_kind: SchedulerEvent,
) -> Result<(), String> {
    // Validate pause state before sending command
    if shared_state.read().is_paused() {
        let reasons = shared_state
            .read()
            .pause_reasons()
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "Cannot trigger event while scheduler is paused (reasons: {reasons})"
        ));
    }

    scheduler_cmd
        .send(Command::TriggerEvent(break_kind))
        .await
        .map_err(|e| e.to_string())
}

/// Skip the current break immediately
#[tauri::command]
pub async fn skip_break(state: State<'_, SchedulerCmd>) -> Result<(), String> {
    state
        .send(Command::SkipBreak)
        .await
        .map_err(|e| e.to_string())
}

/// Notify that a break has finished normally
#[tauri::command]
pub async fn prompt_finished(
    state: State<'_, SchedulerCmd>,
    event: SchedulerEvent,
) -> Result<(), String> {
    state
        .send(Command::PromptFinished(event))
        .await
        .map_err(|e| e.to_string())
}
