//! Tauri commands for system autostart management.
//!
//! This module provides commands to check and configure whether the application
//! launches automatically when the system starts.

use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;

use crate::config::{SharedConfig, save_config};

/// Checks if autostart is enabled in the configuration.
///
/// # Errors
///
/// This function does not return errors in normal operation.
#[tauri::command]
pub async fn is_autostart_enabled(app: tauri::AppHandle) -> Result<bool, String> {
    let config = app.state::<SharedConfig>();
    let config_guard = config.read().await;
    Ok(config_guard.autostart)
}

/// Enables or disables autostart for the application.
///
/// Updates both the system autostart configuration and the application's
/// configuration file. The configuration is saved only if the system autostart
/// operation succeeds.
///
/// # Errors
///
/// Returns an error if:
/// - Checking the current autostart status fails
/// - Enabling or disabling system autostart fails
#[tauri::command]
pub async fn set_autostart_enabled(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let autolaunch = app.autolaunch();

    let current_status = autolaunch.is_enabled().map_err(|e| {
        tracing::error!("Failed to check current autostart status: {e}");
        format!("Failed to check current autostart status: {e}")
    })?;
    tracing::info!("Current autostart status: {current_status}");

    if current_status == enabled {
        tracing::warn!("Autostart is already set to {enabled}");
        // Still update config to ensure sync with OS state
    } else {
        // Try to set system autostart
        let result = if enabled {
            autolaunch.enable()
        } else {
            autolaunch.disable()
        };

        result.map_err(|e| {
            tracing::error!("Failed to set system autostart: {e}");
            format!("Failed to set system autostart (but preference saved): {e}")
        })?;
    }

    // Update config
    {
        let config = app.state::<SharedConfig>();
        let mut config_guard = config.write().await;
        config_guard.autostart = enabled;

        // Save config to disk
        save_config(&app, &config_guard).await.unwrap_or_else(|e| {
            tracing::warn!("Failed to save autostart config: {e}");
        });
    }

    Ok(())
}
