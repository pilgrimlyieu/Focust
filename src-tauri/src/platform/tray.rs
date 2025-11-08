use std::sync::{Arc, Mutex};

use tauri::{
    AppHandle, Listener, Manager, Runtime,
    menu::{Menu, MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
};
use tokio::sync::mpsc;

use crate::platform::{
    get_strings,
    i18n::{LanguageStrings, TrayStrings},
};
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
    let (tray_state, tray_rx) = initialize_tray_state(app);

    let strings = get_localized_strings(app).await;
    let tray_text = &strings.tray;

    let initial_menu = build_tray_menu(app, tray_text, false)?;

    let icon = app
        .default_window_icon()
        .ok_or_else(|| anyhow::anyhow!("No default window icon available"))?
        .clone();

    let tray = TrayIconBuilder::new()
        .menu(&initial_menu)
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

    spawn_tray_update_task(app.clone(), tray.clone(), tray_rx, strings);

    listen_for_scheduler_status(app, tray_state);

    tracing::info!("System tray icon created successfully");
    Ok(())
}

/// Initialize tray state and return receiver for updates
fn initialize_tray_state<R: Runtime>(
    app: &AppHandle<R>,
) -> (TrayState, mpsc::UnboundedReceiver<TrayUpdate>) {
    let (tray_tx, tray_rx) = mpsc::unbounded_channel::<TrayUpdate>();
    let tray_state = TrayState {
        scheduler_paused: Arc::new(Mutex::new(false)),
        tray_sender: Arc::new(Mutex::new(Some(tray_tx))),
    };
    app.manage(tray_state.clone());
    (tray_state, tray_rx)
}

/// Get localized strings based on current config language
async fn get_localized_strings<R: Runtime>(app: &AppHandle<R>) -> LanguageStrings {
    let lang = if let Some(config_state) = app.try_state::<SharedConfig>() {
        let config = config_state.read().await;
        config.language.clone()
    } else {
        tracing::warn!("Config not yet loaded, using default language {LANGUAGE_FALLBACK}");
        LANGUAGE_FALLBACK.to_string()
    };
    get_strings(&lang)
}

/// Build tray menu with localized text and current pause state
fn build_tray_menu<R: Runtime>(
    app: &AppHandle<R>,
    tray_text: &TrayStrings,
    paused: bool,
) -> tauri::Result<Menu<R>> {
    let pause_resume_text = if paused {
        &tray_text.resume
    } else {
        &tray_text.pause
    };

    let show_item = MenuItemBuilder::with_id("show", &tray_text.show).build(app)?;
    let pause_item = MenuItemBuilder::with_id("pause_or_resume", pause_resume_text).build(app)?;
    let restart_item = MenuItemBuilder::with_id("restart", &tray_text.restart).build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", &tray_text.quit).build(app)?;

    MenuBuilder::new(app)
        .items(&[&show_item, &pause_item])
        .separator()
        .items(&[&restart_item, &quit_item])
        .build()
}

/// Spawn a task to handle tray menu updates
fn spawn_tray_update_task<R: Runtime>(
    app_handle: AppHandle<R>,
    tray: TrayIcon<R>,
    mut tray_rx: mpsc::UnboundedReceiver<TrayUpdate>,
    strings: LanguageStrings,
) {
    tokio::spawn(async move {
        let tray_clone = tray.clone();
        while let Some(update) = tray_rx.recv().await {
            match update {
                TrayUpdate::UpdateMenu(paused) => {
                    if let Ok(menu) = build_tray_menu(&app_handle, &strings.tray, paused) {
                        let _ = tray_clone.set_menu(Some(menu));
                    } else {
                        tracing::error!("Failed to build tray menu for update.");
                    }
                }
            }
        }
    });
}

/// Listen for scheduler status events to update tray menu
fn listen_for_scheduler_status<R: Runtime>(app: &AppHandle<R>, tray_state: TrayState) {
    app.listen("scheduler-status", move |event| {
        if let Ok(status) = serde_json::from_str::<SchedulerStatus>(event.payload()) {
            // Update stored state
            if let Ok(mut paused) = tray_state.scheduler_paused.lock() {
                *paused = status.paused;
            }

            // Send update message to tray update task
            if let Ok(sender_option) = tray_state.tray_sender.lock()
                && let Some(sender) = sender_option.as_ref()
            {
                sender
                    .send(TrayUpdate::UpdateMenu(status.paused))
                    .unwrap_or_else(|e| {
                        tracing::warn!("Failed to send tray update: {e}");
                    });
            }
        } else {
            tracing::warn!("Failed to parse 'scheduler-status' event payload.");
        }
    });
}

/// Handle tray menu item clicks
fn handle_tray_menu_event<R: Runtime>(app: &AppHandle<R>, event_id: &str) {
    match event_id {
        "show" => {
            show_settings_window(app);
        }
        "pause_or_resume" => {
            toggle_pause(app).unwrap_or_else(|e| {
                tracing::error!("Failed to toggle pause: {e}");
            });
        }
        "restart" => {
            tracing::info!("Restart requested from tray menu");
            app.restart();
        }
        "quit" => {
            tracing::info!("Quit requested from tray menu");
            app.exit(0);
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
        open_settings_window(app_clone).await.unwrap_or_else(|e| {
            tracing::error!("Failed to open settings window: {e}");
        });
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
            .try_send(Command::Resume(PauseReason::Manual))
            .map_err(|e| format!("Failed to send resume command: {e}"))?;
        tracing::info!("Resume sent from tray menu");
    } else {
        // Currently running, send pause command
        scheduler_cmd
            .try_send(Command::Pause(PauseReason::Manual))
            .map_err(|e| format!("Failed to send pause command: {e}"))?;
        tracing::info!("Pause sent from tray menu");
    }

    Ok(())
}
