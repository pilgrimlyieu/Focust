use tauri::{AppHandle, State, command};

use crate::{
    cmd::SchedulerCmd,
    config,
    config::{AppConfig, SharedConfig},
    scheduler::models::Command,
};

#[command]
pub async fn get_config(config_state: State<'_, SharedConfig>) -> Result<AppConfig, String> {
    Ok(config_state.read().await.clone())
}

#[command]
pub async fn save_config(
    config: AppConfig,
    app_handle: AppHandle,
    scheduler_cmd: State<'_, SchedulerCmd>,
    config_state: State<'_, SharedConfig>,
) -> Result<(), String> {
    if let Err(e) = config::save_config(&app_handle, &config).await {
        let err_msg = format!("Failed to save config file: {e}");
        log::error!("{err_msg}");
        return Err(err_msg);
    }

    // Update the scheduler with the new config
    if let Err(e) = scheduler_cmd
        .0
        .send(Command::UpdateConfig(config.clone()))
        .await
    {
        let err_msg = format!("Failed to send update_config command to scheduler: {e}");
        log::error!("{err_msg}");
        return Err(err_msg);
    }

    // Update the shared config state
    let mut config_guard = config_state.write().await;
    *config_guard = config;

    Ok(())
}
