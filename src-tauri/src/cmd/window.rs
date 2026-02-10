//! Tauri commands for window management.
//!
//! This module provides commands to open and close application windows.

use tauri::{AppHandle, Manager, Runtime};

/// Closes all prompt windows with the given payload ID prefix.
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
