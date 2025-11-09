use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_notification::NotificationExt;

use crate::platform::i18n;
use crate::{config::SharedConfig, platform::i18n::LANGUAGE_FALLBACK};

/// Send a notification to the user
///
/// # Arguments
/// * `app` - Tauri app handle
/// * `title` - Notification title
/// * `body` - Notification body text
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

/// Send a notification before a break starts
///
/// # Arguments
/// * `app` - Tauri app handle
/// * `break_type` - Type of break (e.g., "`MiniBreak`", "`LongBreak`", "Attention")
/// * `seconds` - Seconds until the break starts
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
        LANGUAGE_FALLBACK.to_string()
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
