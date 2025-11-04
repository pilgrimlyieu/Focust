use tauri::{
    AppHandle, Manager, Runtime,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

use crate::platform::i18n;
use crate::scheduler::models::Command;
use crate::{
    cmd::{SchedulerCmd, open_settings_window},
    scheduler::PauseReason,
};
use crate::{config::SharedConfig, platform::i18n::LANGUAGE_FALLBACK};

/// Setup system tray icon with menu (should be called after config is loaded)
pub async fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
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
    let tray_text = &strings.tray;

    // Build tray menu with localized text
    let show_item = MenuItemBuilder::with_id("show", &tray_text.show).build(app)?;
    let pause_item = MenuItemBuilder::with_id("pause", &tray_text.pause).build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", &tray_text.quit).build(app)?;

    let menu = MenuBuilder::new(app)
        .items(&[&show_item, &pause_item])
        .separator()
        .items(&[&quit_item])
        .build()?;

    // Get default icon or return error if not available
    let icon = app
        .default_window_icon()
        .ok_or_else(|| anyhow::anyhow!("No default window icon available"))?
        .clone();

    // Build tray icon
    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .icon(icon)
        .tooltip(&tray_text.tooltip)
        .on_menu_event(move |app, event| {
            handle_tray_menu_event(app, event.id.as_ref());
        })
        .on_tray_icon_event(|tray, event| {
            // Handle tray icon click events
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_settings_window(tray.app_handle());
            }
        })
        .build(app)?;

    tracing::info!("System tray icon created successfully");
    Ok(())
}

/// Handle tray menu item clicks
fn handle_tray_menu_event<R: Runtime>(app: &AppHandle<R>, event_id: &str) {
    match event_id {
        "show" => {
            show_settings_window(app);
        }
        "pause" => {
            if let Err(e) = toggle_pause(app) {
                tracing::error!("Failed to toggle pause: {e}");
            }
        }
        "quit" => {
            tracing::info!("Quit requested from tray menu");
            // Close all windows gracefully before exiting
            if let Some(window) = app.get_webview_window("settings") {
                let _ = window.close();
            }
            // Use std::process::exit for cleaner shutdown
            std::process::exit(0);
        }
        _ => {
            tracing::warn!("Unknown tray menu event: {event_id}");
        }
    }
}

/// Show or focus the settings window (create on demand)
fn show_settings_window<R: Runtime>(app: &AppHandle<R>) {
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = open_settings_window(app_clone).await {
            tracing::error!("Failed to open settings window: {e}");
        }
    });
}

/// Toggle scheduler pause state
fn toggle_pause<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    // TODO: Track pause state and toggle accordingly
    let scheduler_cmd = app.state::<SchedulerCmd>();

    // Send pause command
    scheduler_cmd
        .0
        .try_send(Command::Pause(PauseReason::Manual))
        .map_err(|e| format!("Failed to send pause command: {e}"))?;

    tracing::info!("Pause toggled from tray menu");
    Ok(())
}
