use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;

use crate::config::{SharedConfig, save_config};

/// Check if autostart is enabled
#[tauri::command]
pub async fn is_autostart_enabled(app: tauri::AppHandle) -> Result<bool, String> {
    let config = app.state::<SharedConfig>();
    let config_guard = config.read().await;
    Ok(config_guard.autostart)
}

/// Enable or disable autostart
#[tauri::command]
pub async fn set_autostart_enabled(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let autolaunch = app.autolaunch();

    let current_status = autolaunch.is_enabled().map_err(|e| {
        tracing::error!("Failed to check current autostart status: {e}");
        format!("Failed to check current autostart status: {e}")
    })?;
    tracing::info!("Current autostart status: {}", current_status);

    if current_status == enabled {
        tracing::warn!("Autostart is already set to {}", enabled);
        return Ok(());
    }

    // Try to set system autostart
    let result = if enabled {
        autolaunch.enable()
    } else {
        autolaunch.disable()
    };

    // Update config regardless of system autostart result
    {
        let config = app.state::<SharedConfig>();
        let mut config_guard = config.write().await;
        config_guard.autostart = enabled;

        // Save config to disk
        save_config(&app, &config_guard).await.unwrap_or_else(|e| {
            tracing::warn!("Failed to save autostart config: {e}");
        });
    }

    // Return system autostart result
    result.map_err(|e| {
        tracing::error!("Failed to set system autostart: {e}");
        format!("Failed to set system autostart (but preference saved): {e}")
    })
}
