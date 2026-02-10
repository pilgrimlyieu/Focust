//! Tauri commands for system-level operations.
//!
//! This module provides commands to open application directories in the
//! system file explorer.

use std::path::Path;
use std::process::Command;

use tauri::AppHandle;

use crate::utils;

/// Opens the configuration directory in the system file explorer.
///
/// # Errors
///
/// Returns an error if:
/// - Getting the config directory path fails
/// - Opening the directory in the file explorer fails
#[tauri::command]
pub async fn open_config_directory(app: AppHandle) -> Result<(), String> {
    let config_dir = utils::get_app_config_dir(&app)
        .map_err(|e| format!("Failed to get config directory: {e}"))?;

    open_directory_in_explorer(&config_dir)
}

/// Opens the log directory in the system file explorer.
///
/// # Errors
///
/// Returns an error if:
/// - Getting the log directory path fails
/// - Opening the directory in the file explorer fails
#[tauri::command]
pub async fn open_log_directory(app: AppHandle) -> Result<(), String> {
    let log_dir =
        utils::get_app_log_dir(&app).map_err(|e| format!("Failed to get log directory: {e}"))?;

    open_directory_in_explorer(&log_dir)
}

/// Opens a directory in the system file explorer.
///
/// Uses platform-specific commands:
/// - Windows: `explorer`
/// - macOS: `open`
/// - Linux: `xdg-open`
///
/// # Errors
///
/// Returns an error if spawning the file explorer process fails.
fn open_directory_in_explorer(dir: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {e}"))?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {e}"))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {e}"))?;
    }

    Ok(())
}

/// Restarts the application.
///
/// This command closes all windows and relaunches the application.
/// The user's configuration and state are preserved.
#[expect(
    clippy::needless_pass_by_value,
    reason = "Reference will make the compilation fail."
)]
#[tauri::command]
pub fn restart_application(app: AppHandle) {
    tracing::info!("Application restart requested from settings");
    app.restart();
}

/// Exits the application.
///
/// This command gracefully shuts down the application,
/// closing all windows and stopping all background tasks.
#[expect(
    clippy::needless_pass_by_value,
    reason = "Reference will make the compilation fail."
)]
#[tauri::command]
pub fn exit_application(app: AppHandle) {
    tracing::info!("Application exit requested from settings");
    app.exit(0);
}
