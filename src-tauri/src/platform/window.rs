use rand::{Rng, seq::IndexedRandom};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tauri::{AppHandle, Listener, Manager, Monitor, Runtime, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::Mutex;

use crate::core::{
    payload::store_payload_internal,
    suggestions::{SharedSuggestions, sample_suggestion},
    theme::BackgroundType,
};
use crate::core::{
    payload::{EventKind, PromptPayload},
    theme::{BackgroundSource, ResolvedBackground},
};
use crate::scheduler::SchedulerEvent;
use crate::{config::AppConfig, core::suggestions::SuggestionsConfig};
use crate::{config::SharedConfig, core::payload::PromptPayloadStore};

const ALLOWED_EXTENSIONS_LOWERCASE: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp", "gif"];

/// Create prompt windows for monitors based on configuration
pub async fn create_prompt_windows(
    app: &AppHandle,
    event: SchedulerEvent,
    postpone_count: u8,
) -> Result<(), String> {
    tracing::debug!("Creating break windows for event: {event}");

    let (payload_id, window_size, all_screens) = {
        let config = app.state::<SharedConfig>();
        let config_guard = config.read().await;

        let suggestions = app.state::<SharedSuggestions>();
        let suggestions_guard = suggestions.read().await;

        // Build prompt payload
        let payload =
            build_prompt_payload(&config_guard, &suggestions_guard, event, postpone_count)?;

        // Generate unique payload ID
        let payload_id = format!("break-{}", chrono::Utc::now().timestamp_millis());

        // Store payload for frontend retrieval
        let payload_store = app.state::<PromptPayloadStore>();
        store_payload_internal(&payload_store, payload.clone(), payload_id.clone())
            .await
            .map_err(|e| format!("Failed to store prompt payload: {e}"))?;

        let window_size = config_guard.window_size;
        let all_screens = config_guard.all_screens;

        (payload_id, window_size, all_screens)
    };

    // Get monitors
    let monitors = if all_screens {
        app.available_monitors()
            .map_err(|e| format!("Failed to get available monitors: {e}"))?
    } else {
        vec![
            app.primary_monitor()
                .map_err(|e| format!("Failed to get primary monitor: {e}"))?
                .ok_or("No primary monitor found")?,
        ]
    };

    tracing::debug!("Creating windows for {} monitor(s)", monitors.len());

    // Create windows for each monitor
    for (index, monitor) in monitors.iter().enumerate() {
        let label = format!("{payload_id}-{index}");
        create_break_window_for_monitor(
            app,
            &label,
            &payload_id,
            f64::from(window_size),
            monitor,
            index == 0,
        )?;
    }

    Ok(())
}

/// Create settings window (internal function used by both command and single instance)
pub fn create_settings_window<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    // Check if window already exists
    if let Some(window) = app.get_webview_window("settings") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        tracing::debug!("Settings window already exists, showing and focusing");
        return Ok(());
    }

    tracing::info!("Creating new settings window");

    // Register event listener BEFORE creating the window
    let (tx, rx) = tokio::sync::oneshot::channel();
    let tx = Arc::new(Mutex::new(Some(tx)));

    let tx_clone = tx.clone();
    let app_clone = app.clone();
    let unlisten = app.listen("settings-ready", move |_event| {
        tracing::info!("Received settings-ready event from frontend");
        let tx_clone = tx_clone.clone();
        tauri::async_runtime::spawn(async move {
            if let Some(sender) = tx_clone.lock().await.take() {
                let _ = sender.send(());
            }
        });
    });

    // Create window
    let _window =
        WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
            .title("Focust - Settings")
            .inner_size(1400.0, 900.0)
            .center()
            .visible(false) // Start hidden to avoid showing blank window
            .build()
            .map_err(|e| e.to_string())?;

    tracing::info!("Settings window created, waiting for ready event...");

    // Wait for ready event with timeout
    let app_clone2 = app.clone();
    tauri::async_runtime::spawn(async move {
        let ready = tokio::time::timeout(tokio::time::Duration::from_millis(2000), rx).await;
        app_clone.unlisten(unlisten);

        match ready {
            Ok(Ok(())) => {
                tracing::info!("Settings window content ready, showing window");
            }
            Ok(Err(e)) => {
                tracing::warn!("Error waiting for ready event: {e}, showing window anyway");
            }
            Err(_) => {
                tracing::warn!("Timeout waiting for ready event, showing window anyway");
            }
        }

        // Show the window now
        if let Some(win) = app_clone2.get_webview_window("settings") {
            let _ = win.show();
            let _ = win.set_focus();
        }
    });

    Ok(())
}

fn is_allowed_image_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext_str| {
            ALLOWED_EXTENSIONS_LOWERCASE
                .iter()
                .any(|&allowed_ext| ext_str.eq_ignore_ascii_case(allowed_ext))
        })
}

/// Create a single break window for a specific monitor
fn create_break_window_for_monitor(
    app: &AppHandle,
    label: &str,
    payload_id: &str,
    window_size: f64,
    monitor: &Monitor,
    is_primary: bool,
) -> Result<(), String> {
    let url = format!("/index.html?view=break&payloadId={payload_id}");

    // Calculate window dimensions
    let scale_factor = monitor.scale_factor();
    let monitor_width = f64::from(monitor.size().width) / scale_factor;
    let monitor_height = f64::from(monitor.size().height) / scale_factor;
    let monitor_x = f64::from(monitor.position().x) / scale_factor;
    let monitor_y = f64::from(monitor.position().y) / scale_factor;

    let is_fullscreen = window_size >= 1.0;

    let mut builder = WebviewWindowBuilder::new(app, label, WebviewUrl::App(url.into()))
        .title("Focust - Break")
        .always_on_top(true)
        .decorations(false)
        .skip_taskbar(true)
        .visible(false)
        .focused(true);

    // Transparency is platform-specific
    #[cfg(not(target_os = "macos"))]
    {
        builder = builder.transparent(true);
    }

    if is_fullscreen {
        builder = builder.fullscreen(true).position(monitor_x, monitor_y);
    } else {
        let window_width = (monitor_width * window_size).floor();
        let window_height = (monitor_height * window_size).floor();
        let window_x = monitor_x + ((monitor_width - window_width) / 2.0).floor();
        let window_y = monitor_y + ((monitor_height - window_height) / 2.0).floor();

        builder = builder
            .inner_size(window_width, window_height)
            .position(window_x, window_y);
    }

    let _window = builder
        .build()
        .map_err(|e| format!("Failed to create break window: {e}"))?;

    tracing::debug!("Break window created: {label} (primary: {is_primary})");

    Ok(())
}

/// Build prompt payload from configuration and event
fn build_prompt_payload(
    config: &AppConfig,
    suggestions: &SuggestionsConfig,
    event: SchedulerEvent,
    postpone_count: u8,
) -> Result<PromptPayload, String> {
    let (break_settings, schedule_name, kind) = match event {
        SchedulerEvent::MiniBreak(id) => {
            let schedule = config
                .schedules
                .iter()
                .find(|s| s.mini_breaks.base.id == id)
                .ok_or_else(|| format!("No schedule found for mini break id: {id}"))?;
            (
                &schedule.mini_breaks.base,
                schedule.name.clone(),
                EventKind::Mini,
            )
        }
        SchedulerEvent::LongBreak(id) => {
            let schedule = config
                .schedules
                .iter()
                .find(|s| s.long_breaks.base.id == id)
                .ok_or_else(|| format!("No schedule found for long break id: {id}"))?;
            (
                &schedule.long_breaks.base,
                schedule.name.clone(),
                EventKind::Long,
            )
        }
        SchedulerEvent::Attention(id) => {
            let attention = config
                .attentions
                .iter()
                .find(|a| a.id == id)
                .ok_or_else(|| format!("No attention found for id: {id}"))?;

            // Build payload for attention
            return Ok(PromptPayload {
                id: attention.id.into(),
                kind: EventKind::Attention,
                title: attention.title.clone(),
                message_key: "break.attentionMessage".to_string(),
                message: Some(attention.message.clone()),
                schedule_name: None,
                duration: attention.duration_s as i32,
                strict_mode: false,
                theme: attention.theme.clone(),
                background: resolve_background(&attention.theme.background),
                suggestion: None,
                audio: None,
                postpone_shortcut: if config.postpone_shortcut.is_empty() {
                    "P".to_string()
                } else {
                    config.postpone_shortcut.clone()
                },
                all_screens: config.all_screens,
                language: config.language.clone(),
                postpone_count: 0, // Attention reminders cannot be postponed
                max_postpone_count: 0,
            });
        }
    };

    let suggestion = break_settings
        .suggestions
        .show
        .then(|| sample_suggestion(suggestions, &config.language))
        .flatten();
    let background = resolve_background(&break_settings.theme.background);

    Ok(PromptPayload {
        id: break_settings.id.into(),
        kind,
        title: schedule_name.clone(),
        message_key: if kind.is_mini() {
            "break.miniBreakMessage".to_string()
        } else {
            "break.longBreakMessage".to_string()
        },
        message: None,
        schedule_name: Some(schedule_name),
        duration: break_settings.duration_s as i32,
        strict_mode: break_settings.strict_mode,
        theme: break_settings.theme.clone(),
        background,
        suggestion,
        audio: Some(break_settings.audio.clone()),
        postpone_shortcut: if config.postpone_shortcut.is_empty() {
            "P".to_string()
        } else {
            config.postpone_shortcut.clone()
        },
        all_screens: config.all_screens,
        language: config.language.clone(),
        postpone_count,
        max_postpone_count: break_settings.max_postpone_count,
    })
}

/// Resolve background source to actual background, or fallback to solid
fn resolve_background(source: &BackgroundSource) -> ResolvedBackground {
    match source.current {
        BackgroundType::Solid => source
            .get_solid()
            .map(|color| ResolvedBackground::new_solid(color.to_string()))
            .unwrap_or_default(),
        BackgroundType::ImagePath => source
            .get_image_path()
            .map(|path| ResolvedBackground::new_image(path.to_string()))
            .unwrap_or_default(),
        BackgroundType::ImageFolder => source
            .get_image_folder()
            .and_then(|folder| {
                let mut rng = rand::rng();
                resolve_random_image_from_folder(folder, &mut rng)
            })
            .unwrap_or_default(),
    }
}

/// Resolve a random image from the specified folder
fn resolve_random_image_from_folder(
    folder_str: &str,
    rng: &mut impl Rng,
) -> Option<ResolvedBackground> {
    let folder_path = PathBuf::from(folder_str);

    if !folder_path.exists() {
        tracing::warn!(
            "Background folder does not exist: {}",
            folder_path.display()
        );
        return None;
    }

    let entries: Vec<PathBuf> = match std::fs::read_dir(&folder_path) {
        Ok(read_dir) => read_dir
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| path.is_file() && is_allowed_image_extension(path))
            .collect(),
        Err(e) => {
            tracing::warn!(
                "Failed to read background folder {}: {e}",
                folder_path.display(),
            );
            return None;
        }
    };

    if entries.is_empty() {
        tracing::warn!("No images found in folder: {}", folder_path.display());
        return None;
    }

    let chosen_entry = entries
        .choose(rng)
        .expect("Should have chosen an entry as entries is not empty");

    let chosen_path_string = chosen_entry.to_string_lossy().to_string();
    tracing::debug!("Chosen background image: {chosen_path_string}");
    Some(ResolvedBackground::new_image(chosen_path_string))
}
