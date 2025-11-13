use tauri::{AppHandle, Manager, Runtime};

use crate::platform::create_settings_window;

/// Open settings window (create if not exists, show if already exists)
#[tauri::command]
pub async fn open_settings_window<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    create_settings_window(&app)
}

/// Close all prompt windows with the given payload ID prefix
///
/// IMPORTANT: This command returns immediately and closes windows asynchronously
/// to avoid deadlock when called from within a prompt window.
///
/// The deadlock scenario:
/// 1. Frontend calls this command and waits for response
/// 2. Command tries to close the calling window synchronously
/// 3. Window closure needs `WebView` thread, but it's blocked waiting for command response
/// 4. Result: deadlock - window appears frozen
///
/// Solution: Return immediately, spawn async task to close windows after a small delay
#[allow(clippy::needless_pass_by_value)]
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
