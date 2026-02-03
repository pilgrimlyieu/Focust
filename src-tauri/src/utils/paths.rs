//! Application path utilities.
//!
//! This module provides utilities for getting application directories
//! with automatic debug/release environment separation.

use std::path::PathBuf;

use anyhow::{Context, Result};
use tauri::{AppHandle, Manager};

/// Gets the application configuration directory.
///
/// In debug builds, returns `com.fesmoph.focust.dev` to separate
/// development and production configurations.
/// In release builds, returns the standard `com.fesmoph.focust`.
///
/// # Errors
///
/// Returns an error if getting the app config directory fails.
pub fn get_app_config_dir(app_handle: &AppHandle) -> Result<PathBuf> {
    let mut config_dir = app_handle
        .path()
        .app_config_dir()
        .context("Failed to get app config directory")?;

    // In debug builds, append .dev to the directory name to separate environments
    #[cfg(debug_assertions)]
    {
        config_dir = modify_app_dir_for_debug(config_dir);
    }

    Ok(config_dir)
}

/// Gets the application log directory.
///
/// In debug builds, the final path component is suffixed with `.dev`.
/// This results in different behavior depending on the platform:
/// - **Windows & Linux:** `.../com.fesmoph.focust/logs` → `.../com.fesmoph.focust/logs.dev`
/// - **macOS:** `~/Library/Logs/com.fesmoph.focust` → `~/Library/Logs/com.fesmoph.focust.dev`
/// In release builds, returns the standard platform paths (no `.dev` suffix).
///
/// # Errors
///
/// Returns an error if getting the app log directory fails.
pub fn get_app_log_dir(app_handle: &AppHandle) -> Result<PathBuf> {
    let mut log_dir = app_handle
        .path()
        .app_log_dir()
        .context("Failed to get app log directory")?;

    // In debug builds, append .dev to the directory name to separate environments
    #[cfg(debug_assertions)]
    {
        log_dir = modify_app_dir_for_debug(log_dir);
    }

    Ok(log_dir)
}

/// Modifies an app directory path to append a `.dev` suffix to the final path component for debug builds.
///
/// Appends `.dev` to the last segment of the path. Examples:
/// - `%APPDATA%\com.fesmoph.focust` → `%APPDATA%\com.fesmoph.focust.dev`
/// - `%LOCALAPPDATA%\com.fesmoph.focust\logs` → `%LOCALAPPDATA%\com.fesmoph.focust\logs.dev`
/// - `~/Library/Logs/com.fesmoph.focust` → `~/Library/Logs/com.fesmoph.focust.dev`
/// - `~/.config/com.fesmoph.focust` → `~/.config/com.fesmoph.focust.dev`
#[cfg(debug_assertions)]
fn modify_app_dir_for_debug(path: PathBuf) -> PathBuf {
    if let Some(dir_name) = path.file_name()
        && let Some(dir_name_str) = dir_name.to_str() {
            let new_dir_name = format!("{dir_name_str}.dev");
            if let Some(parent) = path.parent() {
                return parent.join(new_dir_name);
            }
        }
    // Fallback: if path manipulation fails, return original path
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(debug_assertions)]
    #[test]
    fn test_modify_app_dir_for_debug() {
        // Test Windows-style path
        #[cfg(target_os = "windows")]
        {
            let original = PathBuf::from(r"C:\Users\TestUser\AppData\Roaming\com.fesmoph.focust");
            let modified = modify_app_dir_for_debug(original);
            assert_eq!(
                modified,
                PathBuf::from(r"C:\Users\TestUser\AppData\Roaming\com.fesmoph.focust.dev")
            );
        }

        // Test macOS-style path
        #[cfg(target_os = "macos")]
        {
            let original =
                PathBuf::from("/Users/TestUser/Library/Application Support/com.fesmoph.focust");
            let modified = modify_app_dir_for_debug(original);
            assert_eq!(
                modified,
                PathBuf::from("/Users/TestUser/Library/Application Support/com.fesmoph.focust.dev")
            );
        }

        // Test Linux-style path
        #[cfg(target_os = "linux")]
        {
            let original = PathBuf::from("/home/testuser/.config/com.fesmoph.focust");
            let modified = modify_app_dir_for_debug(original);
            assert_eq!(
                modified,
                PathBuf::from("/home/testuser/.config/com.fesmoph.focust.dev")
            );
        }
    }
}
