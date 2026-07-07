//! Tauri commands for scheduler state management and control.
//!
//! This module provides commands to interact with the break scheduler,
//! including pausing/resuming, postponing breaks, and triggering events.

use std::ops::Deref;

use tauri::State;
use tokio::sync::{mpsc::Sender, watch};

use crate::scheduler::models::{Command, PauseReason, SchedulerEvent};
use crate::scheduler::shared_state::SharedState;

/// Wrapper around the scheduler command sender for Tauri state management.
pub struct SchedulerCmd(pub Sender<Command>);

impl Deref for SchedulerCmd {
    type Target = Sender<Command>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SchedulerCmd {
    /// Creates a new `SchedulerCmd` wrapper.
    #[must_use]
    pub fn new(sender: Sender<Command>) -> Self {
        SchedulerCmd(sender)
    }

    /// Tries to send a command to the scheduler without blocking.
    ///
    /// Logs an error if the command fails to send.
    pub fn try_send_command(&self, command: &Command) {
        self.0.try_send(command.clone()).unwrap_or_else(|e| {
            tracing::error!("Failed to send {command} to scheduler: {e}");
        });
    }

    /// Sends a command to the scheduler asynchronously.
    ///
    /// Logs an error if the command fails to send.
    pub async fn send_command(&self, command: &Command) {
        self.0.send(command.clone()).await.unwrap_or_else(|e| {
            tracing::error!("Failed to send {command} to scheduler: {e}");
        });
    }
}

/// Shutdown sender wrapper to keep the scheduler alive.
///
/// Dropping this sender will signal the scheduler to shut down.
pub struct ShutdownTx(pub watch::Sender<()>);

/// Requests the scheduler to emit its current status.
///
/// # Errors
///
/// Returns an error if sending the command to the scheduler fails.
#[tauri::command]
pub async fn request_break_status(state: State<'_, SchedulerCmd>) -> Result<(), String> {
    state
        .send(Command::RequestBreakStatus)
        .await
        .map_err(|e| e.to_string())
}

/// Pauses the scheduler manually.
///
/// Pauses the scheduler with `PauseReason::Manual`.
///
/// # Errors
///
/// Returns an error if sending the command to the scheduler fails.
#[tauri::command]
pub async fn pause_scheduler(state: State<'_, SchedulerCmd>) -> Result<(), String> {
    state
        .send(Command::Pause(PauseReason::Manual))
        .await
        .map_err(|e| e.to_string())
}

/// Resumes the scheduler from user-started pauses.
///
/// Clears both ordinary manual pause and timed manual pause. Environment-driven
/// pause reasons such as DND, idle, and app exclusion remain active.
///
/// # Errors
///
/// Returns an error if sending the command to the scheduler fails.
#[tauri::command]
pub async fn resume_scheduler(state: State<'_, SchedulerCmd>) -> Result<(), String> {
    state
        .send(Command::ResumeUserPauses)
        .await
        .map_err(|e| e.to_string())
}

/// Postpones the current or next break.
///
/// # Errors
///
/// Returns an error if:
/// - The scheduler is currently paused
/// - Sending the command to the scheduler fails
#[tauri::command]
pub async fn postpone_break(
    state: State<'_, SchedulerCmd>,
    shared_state: State<'_, SharedState>,
) -> Result<(), String> {
    validate_not_paused(&shared_state)?;
    state
        .send(Command::PostponeBreak)
        .await
        .map_err(|e| e.to_string())
}

/// Manually triggers a break event for testing purposes.
///
/// # Errors
///
/// Returns an error if:
/// - The scheduler is currently paused
/// - Sending the command to the scheduler fails
#[tauri::command]
pub async fn trigger_event(
    scheduler_cmd: State<'_, SchedulerCmd>,
    shared_state: State<'_, SharedState>,
    break_kind: SchedulerEvent,
) -> Result<(), String> {
    validate_not_paused(&shared_state)?;
    scheduler_cmd
        .send(Command::TriggerEvent(break_kind))
        .await
        .map_err(|e| e.to_string())
}

/// Skips the current break immediately.
///
/// # Errors
///
/// Returns an error if:
/// - The scheduler is currently paused
/// - Sending the command to the scheduler fails
#[tauri::command]
pub async fn skip_break(
    state: State<'_, SchedulerCmd>,
    shared_state: State<'_, SharedState>,
) -> Result<(), String> {
    validate_not_paused(&shared_state)?;
    state
        .send(Command::SkipBreak)
        .await
        .map_err(|e| e.to_string())
}

/// Notifies that a break has finished normally.
///
/// # Errors
///
/// Returns an error if sending the command to the scheduler fails.
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

/// Validates that the scheduler is not paused, returning a descriptive error if it is.
///
/// # Errors
///
/// Returns an error if the scheduler is currently paused.
fn validate_not_paused(shared_state: &SharedState) -> Result<(), String> {
    if shared_state.read().is_paused() {
        let reasons = shared_state
            .read()
            .pause_reasons()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "Cannot perform action while scheduler is paused (reasons: {reasons})"
        ));
    }
    Ok(())
}
