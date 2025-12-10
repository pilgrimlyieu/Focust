//! Tauri commands for system-level operations.
//!
//! This module provides commands to open application directories in the
//! system file explorer.

use std::path::Path;
use std::process::Command;

use tauri::{AppHandle, Manager};

/// Opens the configuration directory in the system file explorer.
///
/// # Errors
///
/// Returns an error if:
/// - Getting the config directory path fails
/// - Opening the directory in the file explorer fails
#[tauri::command]
pub async fn open_config_directory(app: AppHandle) -> Result<(), String> {
    let config_dir = app
        .path()
        .app_config_dir()
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
    let log_dir = app
        .path()
        .app_log_dir()
        .map_err(|e| format!("Failed to get log directory: {e}"))?;

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
