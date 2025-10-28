use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

/// Send a notification to the user
///
/// # Arguments
/// * `app` - Tauri app handle
/// * `title` - Notification title
/// * `body` - Notification body text
pub fn send_notification(app: &AppHandle, title: &str, body: &str) -> Result<(), String> {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| format!("Failed to show notification: {e}"))?;

    tracing::debug!("Notification sent: {title} - {body}");
    Ok(())
}

/// Send a notification before a break starts
///
/// # Arguments
/// * `app` - Tauri app handle
/// * `break_type` - Type of break (e.g., "Mini Break", "Long Break")
/// * `seconds` - Seconds until the break starts
pub fn send_break_notification(
    app: &AppHandle,
    break_type: &str,
    seconds: u32,
) -> Result<(), String> {
    let title = format!("{break_type} in {seconds} seconds"); // TODO: i18n
    let body = "Time to take a break and rest your eyes.";

    send_notification(app, &title, body)
}
