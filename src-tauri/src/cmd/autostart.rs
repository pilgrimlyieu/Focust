use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;

use crate::config::{SharedConfig, save_config};

/// Check if autostart is enabled
#[tauri::command]
pub async fn is_autostart_enabled(app: tauri::AppHandle) -> Result<bool, String> {
    // First check config
    let config = app.state::<SharedConfig>();
    let config_guard = config.read().await;
    Ok(config_guard.autostart)
}

/// Enable or disable autostart
#[tauri::command]
pub async fn set_autostart_enabled(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let autolaunch = app.autolaunch();

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
        if let Err(e) = save_config(&app, &config_guard).await {
            tracing::warn!("Failed to save autostart config: {e}");
        }
    }

    // Return system autostart result
    result.map_err(|e| {
        tracing::error!("Failed to set system autostart: {e}");
        format!("Failed to set system autostart (but preference saved): {e}")
    })
}
