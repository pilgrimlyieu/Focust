use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::Context;
use rand::Rng;
use tauri::{AppHandle, State};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tokio::task;

use crate::{
    cmd::SchedulerCmd,
    config::{self, AppConfig, SharedConfig},
    platform::register_shortcuts,
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
    config::save_config(&app_handle, &config)
        .await
        .map_err(|e| {
            tracing::error!("Failed to save config file: {e}");
            e.to_string()
        })?;

    // Update the scheduler with the new config
    scheduler_cmd
        .send(Command::UpdateConfig(config.clone()))
        .await
        .map_err(|e| {
            tracing::error!("Failed to send update_config command to scheduler: {e}");
            e.to_string()
        })?;

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
        app_handle
            .global_shortcut()
            .unregister_all()
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to unregister existing shortcuts: {e}");
            });

        // Re-register with new shortcut
        register_shortcuts(&app_handle).await.map_err(|e| {
            let err_str = e.clone();
            tracing::error!("Failed to re-register shortcuts: {err_str}");
            err_str
        })?;

        tracing::info!("Shortcuts re-registered successfully");
    }

    Ok(())
}

#[tauri::command]
pub async fn pick_background_image(folder: String) -> Result<Option<String>, String> {
    use anyhow::{Result as AnyhowResult, anyhow};

    let folder = PathBuf::from(folder);
    if !folder.exists() {
        tracing::warn!("Background folder does not exist: {}", folder.display());
        return Ok(None);
    }

    let result = task::spawn_blocking(move || -> AnyhowResult<Option<PathBuf>> {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(&folder)
            .with_context(|| format!("Failed to read folder {}", folder.display()))?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| path.is_file() && is_image(path))
            .collect();

        if entries.is_empty() {
            return Ok(None);
        }

        let mut rng = rand::rng();
        let index = rng.random_range(0..entries.len());
        Ok(Some(entries.swap_remove(index)))
    })
    .await
    .map_err(|e| anyhow!("Background picker task panicked: {e}"))
    .and_then(|r| r)
    .map_err(|e| e.to_string())?;

    Ok(result.map(|path| path.to_string_lossy().to_string()))
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
