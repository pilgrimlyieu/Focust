use std::ops::Deref;

use tauri::State;
use tokio::sync::{mpsc::Sender, watch};

use crate::scheduler::models::{Command, EventKind, PauseReason};

pub struct SchedulerCmd(pub Sender<Command>);

impl Deref for SchedulerCmd {
    type Target = Sender<Command>;

    fn deref(&self) -> &Self::Target {
        &self.0
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
#[tauri::command]
pub async fn pause_scheduler(state: State<'_, SchedulerCmd>) -> Result<(), String> {
    state
        .send(Command::Pause(PauseReason::Manual)) // TODO: allow specifying reason
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
    break_kind: EventKind,
) -> Result<(), String> {
    state
        .send(Command::TriggerBreak(break_kind))
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
