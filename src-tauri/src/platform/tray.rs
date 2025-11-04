use std::sync::{Arc, Mutex};

use tauri::{
    AppHandle, Listener, Manager, Runtime,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

use crate::platform::get_strings;
use crate::scheduler::models::{Command, SchedulerStatus};
use crate::{
    cmd::{SchedulerCmd, open_settings_window},
    scheduler::PauseReason,
};
use crate::{config::SharedConfig, platform::i18n::LANGUAGE_FALLBACK};

/// Global state to track scheduler pause status and tray reference for menu updates
#[derive(Clone)]
pub struct TrayState {
    pub scheduler_paused: Arc<Mutex<bool>>,
    pub tray_sender: Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<TrayUpdate>>>>,
}

/// Messages for updating the tray menu
pub enum TrayUpdate {
    UpdateMenu(bool), // bool: paused state
}

/// Setup system tray icon with menu (should be called after config is loaded)
pub async fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    // Initialize tray state
    let (tray_tx, mut tray_rx) = tokio::sync::mpsc::unbounded_channel::<TrayUpdate>();
    let tray_state = TrayState {
        scheduler_paused: Arc::new(Mutex::new(false)),
        tray_sender: Arc::new(Mutex::new(Some(tray_tx))),
    };
    app.manage(tray_state.clone());

    let lang = if let Some(config_state) = app.try_state::<SharedConfig>() {
        let config = config_state.read().await;
        config.language.clone()
    } else {
        tracing::warn!("Config not yet loaded, using default language {LANGUAGE_FALLBACK}");
        LANGUAGE_FALLBACK.to_string()
    };

    let strings = get_strings(&lang);
    let tray_text = &strings.tray;

    // Build tray menu with initial localized text
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
    let tray = TrayIconBuilder::new()
        .menu(&menu)
        .icon(icon)
        .tooltip(&tray_text.tooltip)
        .on_menu_event(move |app, event| {
            handle_tray_menu_event(app, event.id.as_ref());
        })
        .on_tray_icon_event(|tray, event| {
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

    // Spawn a task to handle tray menu updates
    let app_handle_for_tray = app.clone();
    tokio::spawn(async move {
        let tray = tray.clone();
        while let Some(update) = tray_rx.recv().await {
            match update {
                TrayUpdate::UpdateMenu(paused) => {
                    let tray_text = &strings.tray;
                    let menu_text = if paused {
                        &tray_text.resume
                    } else {
                        &tray_text.pause
                    };

                    let show_item = MenuItemBuilder::with_id("show", &tray_text.show)
                        .build(&app_handle_for_tray);
                    let pause_item =
                        MenuItemBuilder::with_id("pause", menu_text).build(&app_handle_for_tray);
                    let quit_item = MenuItemBuilder::with_id("quit", &tray_text.quit)
                        .build(&app_handle_for_tray);

                    if let (Ok(show_item), Ok(pause_item), Ok(quit_item)) =
                        (show_item, pause_item, quit_item)
                    {
                        let menu = MenuBuilder::new(&app_handle_for_tray)
                            .items(&[&show_item, &pause_item])
                            .separator()
                            .items(&[&quit_item])
                            .build();

                        if let Ok(menu) = menu {
                            let _ = tray.set_menu(Some(menu));
                        }
                    }
                }
            }
        }
    });

    // Listen for scheduler status changes to update menu
    let tray_state_clone = tray_state.clone();
    app.listen("scheduler-status", move |event| {
        if let Ok(status) = serde_json::from_str::<SchedulerStatus>(event.payload()) {
            // Update stored state
            if let Ok(mut paused) = tray_state_clone.scheduler_paused.lock() {
                *paused = status.paused;
            }

            // Send update message to tray update task
            if let Ok(sender_option) = tray_state_clone.tray_sender.lock()
                && let Some(sender) = sender_option.as_ref()
            {
                let _ = sender.send(TrayUpdate::UpdateMenu(status.paused));
            }
        }
    });

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
    let scheduler_cmd = app.state::<SchedulerCmd>();

    // Get current pause state from tray state
    let is_paused = if let Some(tray_state) = app.try_state::<TrayState>()
        && let Ok(paused) = tray_state.scheduler_paused.lock()
    {
        *paused
    } else {
        false
    };

    if is_paused {
        // Currently paused, send resume command
        scheduler_cmd
            .0
            .try_send(Command::Resume(PauseReason::Manual))
            .map_err(|e| format!("Failed to send resume command: {e}"))?;
        tracing::info!("Resume sent from tray menu");
    } else {
        // Currently running, send pause command
        scheduler_cmd
            .0
            .try_send(Command::Pause(PauseReason::Manual))
            .map_err(|e| format!("Failed to send pause command: {e}"))?;
        tracing::info!("Pause sent from tray menu");
    }

    Ok(())
}
