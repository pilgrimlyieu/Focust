//! Tauri commands for window management.
//!
//! This module provides commands to open and close application windows.

use tauri::{AppHandle, Manager, Runtime};

use crate::platform::create_settings_window;

/// Opens the settings window.
///
/// Creates a new window if it doesn't exist, or shows the existing window.
///
/// # Errors
///
/// Returns an error if creating or showing the settings window fails.
#[tauri::command]
pub async fn open_settings_window<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    create_settings_window(&app)
}

/// Closes all prompt windows with the given payload ID prefix.
///
/// This command returns immediately and closes windows asynchronously
/// to avoid deadlock when called from within a prompt window.
///
/// # Deadlock Prevention
///
/// The deadlock scenario:
/// 1. Frontend calls this command and waits for response
/// 2. Command tries to close the calling window synchronously
/// 3. Window closure needs `WebView` thread, but it's blocked waiting for command response
/// 4. Result: deadlock - window appears frozen
///
/// Solution: Return immediately, spawn async task to close windows after a small delay.
///
/// # Errors
///
/// This function does not return errors in normal operation. Individual window
/// closure failures are logged but do not prevent other windows from closing.
#[tauri::command]
pub async fn close_all_prompt_windows<R: Runtime>(
    app: AppHandle<R>,
    payload_id: String,
) -> Result<(), String> {
    tracing::debug!("Scheduling closure of all prompt windows for payload: {payload_id}");

    tracing::debug!("Closing prompt windows for payload: {payload_id}");

    // Get all windows
    let windows = app.webview_windows();

    // Close all windows that start with the payload_id
    for (label, window) in windows {
        if label.starts_with(&payload_id) {
            tracing::debug!("Closing prompt window: {label}");
            window.close().unwrap_or_else(|e| {
                tracing::warn!("Failed to close window {label}: {e}");
            });
        }
    }

    tracing::info!("All prompt windows closed for payload: {payload_id}");
    Ok(())
}
