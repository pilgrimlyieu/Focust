use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use rand::seq::SliceRandom;
use tauri::{AppHandle, State};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tokio::task;

use crate::{
    cmd::SchedulerCmd,
    config,
    config::{AppConfig, SharedConfig},
    scheduler::Command,
};

#[tauri::command]
pub async fn get_config(config_state: State<'_, SharedConfig>) -> Result<AppConfig, String> {
    Ok(config_state.read().await.clone())
}

#[tauri::command]
pub async fn save_config(
    config: AppConfig,
    app_handle: AppHandle,
    scheduler_cmd: State<'_, SchedulerCmd>,
    config_state: State<'_, SharedConfig>,
) -> Result<(), String> {
    // Get the old config to compare shortcuts
    let old_shortcut = {
        let config_guard = config_state.read().await;
        config_guard.postpone_shortcut.clone()
    };

    // Save config to file
    if let Err(e) = config::save_config(&app_handle, &config).await {
        let err_msg = format!("Failed to save config file: {e}");
        tracing::error!("{err_msg}");
        return Err(err_msg);
    }

    // Update the scheduler with the new config
    if let Err(e) = scheduler_cmd
        .send(Command::UpdateConfig(config.clone()))
        .await
    {
        let err_msg = format!("Failed to send update_config command to scheduler: {e}");
        tracing::error!("{err_msg}");
        return Err(err_msg);
    }

    // Update the shared config state
    {
        let mut config_guard = config_state.write().await;
        *config_guard = config.clone();
    }

    // Re-register shortcuts if they changed
    if old_shortcut != config.postpone_shortcut {
        tracing::info!(
            "Postpone shortcut changed from '{old_shortcut}' to '{}', re-registering shortcuts",
            config.postpone_shortcut
        );

        // Unregister all existing shortcuts
        // TODO: only unregister the changed one
        if let Err(e) = app_handle.global_shortcut().unregister_all() {
            tracing::warn!("Failed to unregister existing shortcuts: {e}");
        }

        // Re-register with new shortcut
        if let Err(e) = crate::platform::register_shortcuts(&app_handle).await {
            tracing::error!("Failed to re-register shortcuts: {e}");
            return Err(format!("Failed to re-register shortcuts: {e}"));
        }

        tracing::info!("Shortcuts re-registered successfully");
    }

    Ok(())
}

fn is_image(path: &Path) -> bool {
    match path.extension().and_then(OsStr::to_str) {
        Some(ext) => matches!(
            ext.to_lowercase().as_str(),
            "png" | "jpg" | "jpeg" | "bmp" | "gif" | "webp"
        ),
        None => false,
    }
}

#[tauri::command]
pub async fn pick_background_image(folder: String) -> Result<Option<String>, String> {
    let folder = PathBuf::from(folder);
    if !folder.exists() {
        tracing::warn!("Background folder does not exist: {}", folder.display());
        return Ok(None);
    }

    let result = task::spawn_blocking(move || -> Result<Option<PathBuf>, String> {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(&folder)
            .map_err(|err| format!("Failed to read folder {}: {err}", folder.display()))?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| path.is_file() && is_image(path))
            .collect();

        if entries.is_empty() {
            return Ok(None);
        }

        let mut rng = rand::rng();
        entries.shuffle(&mut rng);
        Ok(entries.into_iter().next())
    })
    .await
    .map_err(|err| format!("Background picker task panicked: {err}"))?;

    Ok(result?.map(|path| path.to_string_lossy().to_string()))
}
