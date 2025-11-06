use std::ops::Deref;

use tauri::State;
use tokio::sync::{mpsc::Sender, watch};

use crate::scheduler::models::{Command, PauseReason, SchedulerEvent};

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
pub async fn request_scheduler_status(state: State<'_, SchedulerCmd>) -> Result<(), String> {
    state
        .send(Command::RequestStatus)
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
        .send(Command::Postpone)
        .await
        .map_err(|e| e.to_string())
}

/// Manually trigger a break for testing purposes
#[tauri::command]
pub async fn trigger_break(
    state: State<'_, SchedulerCmd>,
    break_kind: SchedulerEvent,
) -> Result<(), String> {
    state
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
pub async fn break_finished(
    state: State<'_, SchedulerCmd>,
    event: SchedulerEvent,
) -> Result<(), String> {
    state
        .send(Command::BreakFinished(event))
        .await
        .map_err(|e| e.to_string())
}
