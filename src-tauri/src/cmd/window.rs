use tauri::{AppHandle, Manager, Runtime};

use crate::platform::create_settings_window;

/// Open settings window (create if not exists, show if already exists)
#[tauri::command]
pub async fn open_settings_window<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    create_settings_window(&app)
}

/// Close all break windows with the given payload ID prefix
#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
pub fn close_all_break_windows<R: Runtime>(
    app: AppHandle<R>,
    payload_id: &str,
) -> Result<(), String> {
    tracing::debug!("Closing all break windows for payload: {payload_id}");

    // Get all windows
    let windows = app.webview_windows();

    // Close all windows that start with the payload_id
    for (label, window) in windows {
        if label.starts_with(payload_id) {
            tracing::debug!("Closing break window: {label}");
            if let Err(e) = window.close() {
                tracing::warn!("Failed to close window {label}: {e}");
            }
        }
    }

    Ok(())
}
