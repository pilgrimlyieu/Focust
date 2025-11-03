#![warn(clippy::pedantic)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_possible_truncation)]

use tauri::Manager;

use crate::{
    cmd::{BreakPayloadStore, SchedulerCmd, ShutdownTx},
    scheduler::init_scheduler,
};

pub mod cmd;
pub mod config;
pub mod core;
pub mod platform;
pub mod scheduler;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
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

            if let Err(e) = utils::init_logging(&log_dir, log_level) {
                eprintln!("Failed to initialize logging: {e}");
            }

            let handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                // Initialize audio player
                if let Err(e) = core::audio::init_audio_player().await {
                    tracing::error!("Failed to initialize audio player: {e}");
                }

                // Initialize break payload store
                handle.manage(BreakPayloadStore::new());

                // Load app config first
                let app_config = config::load_config(&handle).await;
                let shared_config = config::SharedConfig::new(app_config.clone());
                handle.manage(shared_config);

                // Setup system tray after config is loaded
                if let Err(e) = platform::setup_tray(&handle).await {
                    tracing::error!("Failed to setup system tray: {e}");
                }

                // Register global shortcuts (after config is managed)
                if let Err(e) = platform::register_shortcuts(&handle).await {
                    tracing::error!("Failed to register global shortcuts: {e}");
                }

                // Load suggestions
                let suggestions_config = core::suggestions::load_suggestions(&handle).await;
                let shared_suggestions = cmd::SharedSuggestions::new(suggestions_config);
                handle.manage(shared_suggestions);

                let (cmd_tx, shutdown_tx) = init_scheduler(&handle);
                handle.manage(SchedulerCmd(cmd_tx)); // keep alive
                handle.manage(ShutdownTx(shutdown_tx));
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd::audio::play_audio,
            cmd::audio::play_builtin_audio,
            cmd::audio::stop_audio,
            cmd::config::get_config,
            cmd::config::pick_background_image,
            cmd::config::save_config,
            cmd::payload::get_break_payload,
            cmd::payload::remove_break_payload,
            cmd::payload::store_break_payload,
            cmd::scheduler::pause_scheduler,
            cmd::scheduler::postpone_break,
            cmd::scheduler::request_scheduler_status,
            cmd::scheduler::resume_scheduler,
            cmd::scheduler::skip_break,
            cmd::scheduler::trigger_break,
            cmd::suggestions::get_suggestions,
            cmd::suggestions::get_suggestions_for_language,
            cmd::suggestions::save_suggestions,
            cmd::system::open_config_directory,
            cmd::system::open_log_directory,
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
