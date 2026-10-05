//! Tauri commands for window management.
//!
//! This module provides commands to open and close application windows.

use tauri::{AppHandle, Runtime};

use crate::platform::close_prompt_windows;

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
    close_prompt_windows(&app, &payload_id);
    Ok(())
}
