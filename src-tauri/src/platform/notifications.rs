//! System notification functionality.
//!
//! This module provides functions to send desktop notifications to users,
//! including localized break notifications.

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_notification::NotificationExt;

use crate::platform::i18n;
use crate::{config::SharedConfig, platform::i18n::LANGUAGE_FALLBACK};

/// Sends a notification to the user.
///
/// # Errors
///
/// Returns an error if the notification fails to display.
pub fn send_notification<R: Runtime>(
    app: &AppHandle<R>,
    title: &str,
    body: &str,
) -> Result<(), String> {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| format!("Failed to show notification: {e}"))?;

    tracing::debug!("Notification sent: {title} - {body}");
    Ok(())
}

/// Sends a localized notification before a break starts.
///
/// The notification uses the user's configured language and displays
/// the break type and countdown in a localized format.
///
/// # Errors
///
/// Returns an error if the notification fails to display.
pub async fn send_break_notification<R: Runtime>(
    app: &AppHandle<R>,
    break_type: &str,
    seconds: u32,
) -> Result<(), String> {
    // Get language from config
    let lang = if let Some(config_state) = app.try_state::<SharedConfig>() {
        // Read config asynchronously
        let config = config_state.read().await;
        config.language.clone()
    } else {
        tracing::warn!("Config not yet loaded, using default language {LANGUAGE_FALLBACK}");
        LANGUAGE_FALLBACK.to_owned()
    };

    let strings = i18n::get_strings(&lang);
    let notif = &strings.notification;

    // Get localized break type name
    let break_type_localized = match break_type {
        "MiniBreak" => &notif.mini_break,
        "LongBreak" => &notif.long_break,
        "Attention" => &notif.attention,
        _ => break_type,
    };

    // Format the notification title
    let title = notif
        .starting_soon
        .replace("{breakType}", break_type_localized)
        .replace("{seconds}", &seconds.to_string());

    let body = &notif.message;

    send_notification(app, &title, body)
}
