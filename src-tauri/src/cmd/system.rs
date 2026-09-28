//! Tauri commands for system-level operations.
//!
//! This module provides commands to open application directories in the
//! system file explorer.

#[cfg(target_os = "windows")]
use std::env;
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

/// File name of the uninstaller placed next to the executable by the NSIS installer.
#[cfg(target_os = "windows")]
const NSIS_UNINSTALLER: &str = "uninstall.exe";

/// Returns whether the app runs as the Windows portable build.
///
/// The NSIS installer places its uninstaller next to the executable, while
/// the portable ZIP does not. Portable builds lack a registered
/// `AppUserModelID`, so Windows silently drops their toast notifications.
/// Debug builds also report `true` so the hint can be previewed for check purposes.
#[must_use]
#[tauri::command]
pub fn is_windows_portable() -> bool {
    #[cfg(target_os = "windows")]
    {
        env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|dir| !dir.join(NSIS_UNINSTALLER).exists()))
            .unwrap_or(false)
    }

    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}
