#![warn(clippy::pedantic)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::struct_excessive_bools)]

use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;

use crate::{
    cmd::{SchedulerCmd, ShutdownTx},
    core::payload::PromptPayloadStore,
    scheduler::manager::SchedulerManager,
};

pub mod cmd;
pub mod config;
pub mod core;
pub mod monitors;
pub mod platform;
pub mod scheduler;
pub mod utils;

#[allow(clippy::too_many_lines)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // When a second instance is launched, focus the existing settings window
            tracing::info!("Single instance: attempting to focus existing window");

            app.get_webview_window("settings")
                .map_or_else(|| {
                    tracing::warn!("Settings window not found, creating new one");
                    platform::create_settings_window(app).unwrap_or_else(|e| {
                        tracing::error!("Failed to create settings window: {e}");
                    });
                },
                |window| {
                    window.show().unwrap_or_else(|e| {
                        tracing::error!("Failed to show settings window: {e}");
                    });
                    window.set_focus().unwrap_or_else(|e| {
                        tracing::error!("Failed to focus settings window: {e}");
                    });
                    window.unminimize().unwrap_or_else(|e| {
                        tracing::error!("Failed to unminimize settings window: {e}");
                    });
                });
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let log_dir = app
                .path()
                .app_log_dir()
                .expect("Failed to get app log directory");

            let log_level = if cfg!(debug_assertions) {
                "info"
            } else {
                "warn"
            };

            utils::init_logging(&log_dir, log_level).unwrap_or_else(|e| {
                eprintln!("Failed to initialize logging: {e}");
            });

            let handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                // Audio initialization (platform-dependent)
                //
                // macOS: Audio temporarily disabled due to cpal Send trait issue
                // - Fixed in cpal PR https://github.com/RustAudio/cpal/pull/1021 (merged, awaiting release in 0.17.0+)
                // - See src/core/audio.rs for restoration plan
                // Windows/Linux: Full audio support
                #[cfg(not(target_os = "macos"))]
                {
                    match core::audio::init_audio_player() {
                        Ok(player_state) => {
                            handle.manage(player_state);
                            tracing::info!("Audio player initialized and managed by Tauri");
                        }
                        Err(e) => {
                            tracing::error!("Failed to initialize audio player: {e}");
                        }
                    }
                }

                #[cfg(target_os = "macos")]
                {
                    match core::audio::init_audio_player() {
                        Ok(()) => {
                            tracing::info!("Audio player initialization skipped on macOS (awaiting cpal 0.17.0+)");
                        }
                        Err(e) => {
                            tracing::error!("Failed to initialize audio player: {e}");
                        }
                    }
                }

                // Initialize prompt payload store
                handle.manage(PromptPayloadStore::new());

                // Load app config first
                let app_config = config::load_config(&handle).await;
                let shared_config = config::SharedConfig::new(app_config.clone());
                handle.manage(shared_config);

                // Sync system autostart with config preference
                if app_config.autostart
                    && let Err(e) = handle.autolaunch().enable()
                {
                    tracing::warn!("Failed to enable autostart on startup: {e}");
                }

                // Setup system tray after config is loaded
                platform::setup_tray(&handle).await.unwrap_or_else(|e| {
                    tracing::error!("Failed to setup system tray: {e}");
                });

                // Register global shortcuts (after config is managed)
                platform::register_shortcuts(&handle).await.unwrap_or_else(|e| {
                    tracing::error!("Failed to register shortcuts: {e}");
                });

                // Load suggestions
                let suggestions_config = core::suggestions::load_suggestions(&handle).await;
                let shared_suggestions =
                    core::suggestions::SharedSuggestions::new(suggestions_config);
                handle.manage(shared_suggestions);

                let (cmd_tx, shutdown_tx, shared_state) = SchedulerManager::init(&handle);

                // Spawn monitors based on configuration
                let mut monitors: Vec<Box<dyn monitors::Monitor>> = vec![];

                // Always add idle monitor (it will self-disable if detection fails)
                tracing::info!("Idle monitoring enabled (threshold: {}s)", app_config.inactive_s);
                monitors.push(Box::new(monitors::IdleMonitor::new(
                    app_config.inactive_s,
                )));

                // Add DND monitor if enabled
                if app_config.monitor_dnd {
                    tracing::info!("DND monitoring enabled");
                    let dnd_monitor = monitors::DndMonitor::new();
                    monitors.push(Box::new(dnd_monitor));
                } else {
                    tracing::info!("DND monitoring disabled");
                }

                // Add app whitelist monitor if there are exclusion rules
                if !app_config.app_exclusions.is_empty() {
                    let active_count = app_config
                        .app_exclusions
                        .iter()
                        .filter(|e| e.active)
                        .count();
                    tracing::info!(
                        "App whitelist monitoring enabled ({active_count} active rule(s))"
                    );
                    monitors.push(Box::new(monitors::AppWhitelistMonitor::new(
                        app_config.app_exclusions.clone(),
                    )));
                }

                if monitors.is_empty() {
                    tracing::info!("No monitors enabled");
                } else {
                    monitors::spawn_monitor_tasks(
                        monitors,
                        cmd_tx.clone(),
                        handle.clone(),
                        shared_state.clone(),
                    );
                }

                handle.manage(SchedulerCmd(cmd_tx)); // keep alive
                handle.manage(ShutdownTx(shutdown_tx));
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd::audio::play_audio,
            cmd::audio::play_builtin_audio,
            cmd::audio::stop_audio,
            cmd::autostart::is_autostart_enabled,
            cmd::autostart::set_autostart_enabled,
            cmd::config::get_config,
            cmd::config::pick_background_image,
            cmd::config::save_config,
            cmd::payload::get_prompt_payload,
            cmd::payload::remove_prompt_payload,
            cmd::payload::store_prompt_payload,
            cmd::scheduler::prompt_finished,
            cmd::scheduler::pause_scheduler,
            cmd::scheduler::postpone_break,
            cmd::scheduler::request_break_status,
            cmd::scheduler::resume_scheduler,
            cmd::scheduler::skip_break,
            cmd::scheduler::trigger_event,
            cmd::suggestions::get_suggestions,
            cmd::suggestions::get_suggestions_for_language,
            cmd::suggestions::save_suggestions,
            cmd::system::open_config_directory,
            cmd::system::open_log_directory,
            cmd::window::close_all_prompt_windows,
            cmd::window::open_settings_window,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            // Prevent exit when all window destroyed
            // See: https://github.com/tauri-apps/tauri/issues/13511
            if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                } else {
                    tracing::info!("Application exit code: {code:?}");
                }
            }
        });
}
