//! System tray icon and menu management.
//!
//! This module manages the system tray icon, handles menu updates based on
//! scheduler state, and provides tray-related event handling.

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration as StdDuration;

use chrono::{DateTime, Utc};
use tauri::async_runtime::spawn as tauri_spawn;
use tauri::{
    AppHandle, Listener, Manager, Runtime,
    menu::{IsMenuItem, Menu, MenuBuilder, MenuItemBuilder, Submenu, SubmenuBuilder},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
};
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::config::{AdvancedConfig, SharedConfig};
use crate::core::break_kind::BreakKind;
use crate::platform::{
    create_settings_window, get_strings,
    i18n::{LANGUAGE_FALLBACK, LanguageStrings, TrayStrings},
};
use crate::scheduler::models::{Command, SchedulerStatus};
use crate::{cmd::SchedulerCmd, scheduler::PauseReason};

/// Menu id prefix for timed pause items; the suffix carries the duration in minutes.
const PAUSE_FOR_MENU_ID_PREFIX: &str = "pause_for_";

/// Interval between tray menu refreshes while a timed pause countdown is shown.
const COUNTDOWN_REFRESH_INTERVAL: StdDuration = StdDuration::from_mins(1);

/// Global state to track scheduler pause status and tray reference for menu updates.
#[derive(Clone)]
pub struct TrayState {
    /// Atomic flag indicating whether the scheduler is paused.
    pub scheduler_paused: Arc<AtomicBool>,
    /// Configured timed pause durations shown in the tray menu.
    pub pause_durations_minutes: Vec<u32>,
    /// Sender for tray menu update messages.
    pub tray_sender: Arc<Mutex<Option<mpsc::UnboundedSender<TrayUpdate>>>>,
}

/// Messages for updating the tray menu.
#[non_exhaustive]
pub enum TrayUpdate {
    /// Rebuilds the menu to reflect the given scheduler state.
    UpdateMenu {
        /// Whether the scheduler is paused.
        paused: bool,
        /// Expiration time of the active timed pause, if any.
        timed_pause_until: Option<DateTime<Utc>>,
    },
}

/// Sets up the system tray icon with menu.
///
/// Should be called after configuration is loaded.
/// If `show_tray_icon` is disabled in config, this function does nothing.
///
/// # Errors
///
/// Returns an error if:
/// - Building the tray menu fails
/// - Creating the tray icon fails
pub async fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    // Check if tray icon should be shown
    let (show_tray, pause_durations_minutes) =
        if let Some(config_state) = app.try_state::<SharedConfig>() {
            let config = config_state.read().await;
            (
                config.show_tray_icon,
                sanitize_pause_durations(&config.advanced.tray_pause_durations_minutes),
            )
        } else {
            tracing::warn!("Config not yet loaded, defaulting to show tray icon");
            (true, AdvancedConfig::default().tray_pause_durations_minutes)
        };

    if !show_tray {
        tracing::info!("Tray icon disabled in config, skipping tray setup");
        return Ok(());
    }

    let (tray_state, tray_rx) = initialize_tray_state(app, pause_durations_minutes);

    let strings = get_localized_strings(app).await;
    let tray_text = &strings.tray;

    let initial_menu = build_tray_menu(
        app,
        &strings,
        false,
        None,
        tray_state.pause_durations_minutes.as_slice(),
    )?;

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

    spawn_tray_update_task(
        app.clone(),
        tray.clone(),
        tray_rx,
        strings,
        tray_state.pause_durations_minutes.clone(),
    );

    listen_for_scheduler_status(app, tray_state);

    tracing::info!("System tray icon created successfully");
    Ok(())
}

/// Initialize tray state and return receiver for updates
fn initialize_tray_state<R: Runtime>(
    app: &AppHandle<R>,
    pause_durations_minutes: Vec<u32>,
) -> (TrayState, mpsc::UnboundedReceiver<TrayUpdate>) {
    let (tray_tx, tray_rx) = mpsc::unbounded_channel::<TrayUpdate>();
    let tray_state = TrayState {
        scheduler_paused: Arc::new(AtomicBool::new(false)),
        pause_durations_minutes,
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
        LANGUAGE_FALLBACK.to_owned()
    };
    get_strings(&lang)
}

/// Build tray menu with localized text and current pause state
fn build_tray_menu<R: Runtime>(
    app: &AppHandle<R>,
    strings: &LanguageStrings,
    paused: bool,
    timed_pause_until: Option<DateTime<Utc>>,
    pause_durations_minutes: &[u32],
) -> tauri::Result<Menu<R>> {
    let tray_text = &strings.tray;
    let has_timed_pause = timed_pause_until.is_some();
    let pause_resume_text = if paused {
        timed_pause_until.map_or_else(
            || tray_text.resume.clone(),
            |until| {
                format!(
                    "{} ({})",
                    tray_text.resume,
                    format_timed_pause_remaining(&strings.tray, until)
                )
            },
        )
    } else {
        tray_text.pause.clone()
    };

    let show_item = MenuItemBuilder::with_id("show", &tray_text.show).build(app)?;
    let pause_item = MenuItemBuilder::with_id("pause_or_resume", pause_resume_text).build(app)?;
    let mini_break_item =
        MenuItemBuilder::with_id("start_mini_break_now", &strings.notification.mini_break)
            .build(app)?;
    let long_break_item =
        MenuItemBuilder::with_id("start_long_break_now", &strings.notification.long_break)
            .build(app)?;
    let start_break_now_menu =
        SubmenuBuilder::with_id(app, "start_break_now", &tray_text.start_break_now)
            .enabled(!paused)
            .items(&[&mini_break_item, &long_break_item])
            .build()?;
    let restart_item = MenuItemBuilder::with_id("restart", &tray_text.restart).build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", &tray_text.quit).build(app)?;

    let pause_for_menu = build_pause_for_submenu(
        app,
        tray_text,
        pause_durations_minutes,
        !paused || has_timed_pause,
    )?;

    let mut builder = MenuBuilder::new(app).items(&[&show_item, &pause_item]);
    if let Some(menu) = &pause_for_menu {
        builder = builder.item(menu);
    }
    builder
        .item(&start_break_now_menu)
        .separator()
        .items(&[&restart_item, &quit_item])
        .build()
}

/// Build the timed pause submenu, or `None` when no durations are configured
fn build_pause_for_submenu<R: Runtime>(
    app: &AppHandle<R>,
    tray_text: &TrayStrings,
    pause_durations_minutes: &[u32],
    enabled: bool,
) -> tauri::Result<Option<Submenu<R>>> {
    if pause_durations_minutes.is_empty() {
        return Ok(None);
    }

    let items = pause_durations_minutes
        .iter()
        .map(|minutes| {
            MenuItemBuilder::with_id(
                format!("{PAUSE_FOR_MENU_ID_PREFIX}{minutes}"),
                format_pause_duration(tray_text, *minutes),
            )
            .build(app)
        })
        .collect::<tauri::Result<Vec<_>>>()?;
    let item_refs = items
        .iter()
        .map(|item| item as &dyn IsMenuItem<R>)
        .collect::<Vec<_>>();
    SubmenuBuilder::with_id(app, "pause_for", &tray_text.pause_for)
        .enabled(enabled)
        .items(&item_refs)
        .build()
        .map(Some)
}

/// Spawn a task to handle tray menu updates
///
/// While a timed pause countdown is displayed, the menu is additionally
/// rebuilt once a minute so the remaining time stays fresh; otherwise the
/// task sleeps until the next status-driven update arrives.
fn spawn_tray_update_task<R: Runtime>(
    app_handle: AppHandle<R>,
    tray: TrayIcon<R>,
    mut tray_rx: mpsc::UnboundedReceiver<TrayUpdate>,
    strings: LanguageStrings,
    pause_durations_minutes: Vec<u32>,
) {
    tokio::spawn(async move {
        let mut paused = false;
        let mut timed_pause_until = None;
        loop {
            let update = if paused && timed_pause_until.is_some() {
                tokio::select! {
                    update = tray_rx.recv() => update,
                    () = sleep(COUNTDOWN_REFRESH_INTERVAL) => Some(TrayUpdate::UpdateMenu {
                        paused,
                        timed_pause_until,
                    }),
                }
            } else {
                tray_rx.recv().await
            };
            let Some(update) = update else {
                break;
            };

            match update {
                TrayUpdate::UpdateMenu {
                    paused: new_paused,
                    timed_pause_until: new_timed_pause_until,
                } => {
                    paused = new_paused;
                    timed_pause_until = new_timed_pause_until;
                    if let Ok(menu) = build_tray_menu(
                        &app_handle,
                        &strings,
                        paused,
                        timed_pause_until,
                        pause_durations_minutes.as_slice(),
                    ) {
                        let _ = tray.set_menu(Some(menu));
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
            tray_state
                .scheduler_paused
                .store(status.paused, Ordering::Relaxed);
            let timed_pause_until = status
                .timed_pause_until
                .as_deref()
                .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
                .map(|value| value.with_timezone(&Utc));

            // Send update message to tray update task
            if let Ok(sender_option) = tray_state.tray_sender.lock()
                && let Some(sender) = sender_option.as_ref()
            {
                sender
                    .send(TrayUpdate::UpdateMenu {
                        paused: status.paused,
                        timed_pause_until,
                    })
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
    if let Some(minutes) = event_id
        .strip_prefix(PAUSE_FOR_MENU_ID_PREFIX)
        .and_then(|value| value.parse::<u32>().ok())
    {
        pause_for_minutes(app, minutes).unwrap_or_else(|e| {
            tracing::error!("Failed to start timed pause from tray menu: {e}");
        });
        return;
    }

    match event_id {
        "show" => {
            show_settings_window(app);
        }
        "pause_or_resume" => {
            toggle_pause(app).unwrap_or_else(|e| {
                tracing::error!("Failed to toggle pause: {e}");
            });
        }
        "start_mini_break_now" => {
            start_break_now(app, BreakKind::Mini).unwrap_or_else(|e| {
                tracing::error!("Failed to start mini break from tray menu: {e}");
            });
        }
        "start_long_break_now" => {
            start_break_now(app, BreakKind::Long).unwrap_or_else(|e| {
                tracing::error!("Failed to start long break from tray menu: {e}");
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
    tauri_spawn(async move {
        create_settings_window(&app_clone).unwrap_or_else(|e| {
            tracing::error!("Failed to open settings window: {e}");
        });
    });
}

/// Toggle scheduler pause state
fn toggle_pause<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let scheduler_cmd = app.state::<SchedulerCmd>();

    // Get current pause state from tray state
    let is_paused = if let Some(tray_state) = app.try_state::<TrayState>() {
        tray_state.scheduler_paused.load(Ordering::Relaxed)
    } else {
        false
    };

    if is_paused {
        // Currently paused, clear all user-started pauses
        scheduler_cmd
            .try_send(Command::ResumeUserPauses)
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

fn pause_for_minutes<R: Runtime>(app: &AppHandle<R>, minutes: u32) -> Result<(), String> {
    let scheduler_cmd = app.state::<SchedulerCmd>();
    scheduler_cmd
        .try_send(Command::PauseForMinutes(minutes))
        .map_err(|e| format!("Failed to send timed pause command: {e}"))?;
    tracing::info!("Timed pause for {minutes} minutes sent from tray menu");
    Ok(())
}

/// Starts a break immediately from the tray menu.
fn start_break_now<R: Runtime>(app: &AppHandle<R>, kind: BreakKind) -> Result<(), String> {
    let scheduler_cmd = app.state::<SchedulerCmd>();
    scheduler_cmd
        .try_send(Command::TriggerBreakNow(kind))
        .map_err(|e| format!("Failed to send start break command: {e}"))?;
    tracing::info!("Start {kind} break sent from tray menu");
    Ok(())
}

/// Drop non-positive values, sort ascending, and remove duplicates
fn sanitize_pause_durations(durations: &[u32]) -> Vec<u32> {
    let mut sanitized = durations
        .iter()
        .copied()
        .filter(|minutes| *minutes > 0)
        .collect::<Vec<_>>();
    sanitized.sort_unstable();
    sanitized.dedup();
    sanitized
}

/// Format a timed pause menu entry, e.g. "15 min"
fn format_pause_duration(tray_text: &TrayStrings, minutes: u32) -> String {
    format!("{} {}", minutes, tray_text.minute_short)
}

/// Format the remaining time of a timed pause, rounded up to whole minutes
fn format_timed_pause_remaining(tray_text: &TrayStrings, until: DateTime<Utc>) -> String {
    // max(1) keeps expired pauses at "1 min"; unsigned_abs is then a lossless
    // i64 -> u64 conversion (signed div_ceil is unstable, `int_roundings`)
    let remaining_seconds = (until - Utc::now()).num_seconds().max(1).unsigned_abs();
    let minutes = remaining_seconds.div_ceil(60);
    tray_text
        .remaining_minutes
        .replace("{minutes}", &minutes.to_string())
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    #[test]
    fn sanitize_pause_durations_sorts_dedups_and_drops_zero() {
        assert_eq!(
            sanitize_pause_durations(&[60, 15, 0, 30, 15]),
            vec![15, 30, 60]
        );
        assert_eq!(sanitize_pause_durations(&[]), Vec::<u32>::new());
        assert_eq!(sanitize_pause_durations(&[0]), Vec::<u32>::new());
    }

    #[test]
    fn format_timed_pause_remaining_rounds_up_to_minutes() {
        let tray_text = TrayStrings::default();

        // 30 seconds left rounds up to 1 minute
        let until = Utc::now() + Duration::seconds(30);
        assert_eq!(
            format_timed_pause_remaining(&tray_text, until),
            "1 min left"
        );

        // A hair over 14 minutes rounds up to 15
        let until = Utc::now() + Duration::seconds(14 * 60 + 5);
        assert_eq!(
            format_timed_pause_remaining(&tray_text, until),
            "15 min left"
        );
    }

    #[test]
    fn format_timed_pause_remaining_clamps_expired_to_one_minute() {
        let tray_text = TrayStrings::default();
        let until = Utc::now() - Duration::minutes(5);
        assert_eq!(
            format_timed_pause_remaining(&tray_text, until),
            "1 min left"
        );
    }
}
