use tauri::{State, command};
use tokio::sync::mpsc::Sender;

use crate::scheduler::models::{Command, PauseReason};

pub struct SchedulerCmd(pub Sender<Command>);

#[command]
pub async fn pause_scheduler(state: State<'_, SchedulerCmd>) -> Result<(), String> {
    state
        .0
        .send(Command::Pause(PauseReason::Manual))
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn resume_scheduler(state: State<'_, SchedulerCmd>) -> Result<(), String> {
    state
        .0
        .send(Command::Resume(PauseReason::Manual))
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn postpone_break(state: State<'_, SchedulerCmd>) -> Result<(), String> {
    state
        .0
        .send(Command::Postpone)
        .await
        .map_err(|e| e.to_string())
}
