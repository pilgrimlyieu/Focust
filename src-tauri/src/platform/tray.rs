use serde::Deserialize;
use std::collections::HashMap;
use tauri::{
    AppHandle, Manager, Runtime,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

use crate::cmd::SchedulerCmd;
use crate::config::SharedConfig;
use crate::scheduler::models::Command;

#[derive(Debug, Deserialize)]
struct TrayTranslations {
    #[serde(flatten)]
    languages: HashMap<String, TrayText>,
}

#[derive(Debug, Deserialize, Clone)]
struct TrayText {
    show: String,
    pause: String,
    #[allow(dead_code)] // TODO: Will be used when implementing dynamic pause/resume toggle
    resume: String,
    quit: String,
    tooltip: String,
}

impl Default for TrayText {
    fn default() -> Self {
        Self {
            show: "Show Settings".to_string(),
            pause: "Pause Breaks".to_string(),
            resume: "Resume Breaks".to_string(),
            quit: "Quit".to_string(),
            tooltip: "Focust - Break Reminder".to_string(),
        }
    }
}

/// Load tray translations from embedded JSON file
fn load_tray_translations() -> TrayTranslations {
    const TRANSLATIONS_JSON: &str = include_str!("../../resources/tray_i18n.json");
    serde_json::from_str(TRANSLATIONS_JSON).unwrap_or_else(|e| {
        tracing::error!("Failed to parse tray translations: {e}");
        TrayTranslations {
            languages: HashMap::new(),
        }
    })
}

/// Get localized tray text based on language
fn get_tray_text(language: &str) -> TrayText {
    let translations = load_tray_translations();

    // Get translation for language, fallback to en-US, then default
    translations
        .languages
        .get(language)
        .or_else(|| translations.languages.get("en-US"))
        .cloned()
        .unwrap_or_default()
}

/// Setup system tray icon with menu (should be called after config is loaded)
pub async fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    // Get language from config
    let lang = if let Some(config_state) = app.try_state::<SharedConfig>() {
        // Read config asynchronously
        let config = config_state.read().await;
        config.language.clone()
    } else {
        tracing::warn!("Config not yet loaded, using default language en-US");
        "en-US".to_string()
    };

    let text = get_tray_text(&lang);

    // Build tray menu with localized text
    let show_item = MenuItemBuilder::with_id("show", &text.show).build(app)?;
    let pause_item = MenuItemBuilder::with_id("pause", &text.pause).build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", &text.quit).build(app)?;

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
        .tooltip(&text.tooltip)
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
                let app = tray.app_handle();
                if let Err(e) = show_settings_window(app) {
                    tracing::error!("Failed to show settings window: {e}");
                }
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
            if let Err(e) = show_settings_window(app) {
                tracing::error!("Failed to show settings window: {e}");
            }
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
fn show_settings_window<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = crate::cmd::window::open_settings_window(app_clone).await {
            tracing::error!("Failed to open settings window: {e}");
        }
    });
    Ok(())
}

/// Toggle scheduler pause state
fn toggle_pause<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    // TODO: Track pause state and toggle accordingly
    let scheduler_cmd = app.state::<SchedulerCmd>();

    // Send pause command
    scheduler_cmd
        .0
        .try_send(Command::Pause(
            crate::scheduler::models::PauseReason::Manual,
        ))
        .map_err(|e| format!("Failed to send pause command: {e}"))?;

    tracing::info!("Pause toggled from tray menu");
    Ok(())
}
