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
            // Initialize logging system
            let log_dir = app
                .path()
                .app_log_dir()
                .expect("Failed to get app log directory");

            if let Err(e) = utils::init_logging(log_dir, "info") {
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

                let (cmd_tx, shutdown_tx) = init_scheduler(handle.clone()).await;
                handle.manage(SchedulerCmd(cmd_tx)); // keep alive
                handle.manage(ShutdownTx(shutdown_tx));
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event
                && window.label() == "settings"
            {
                api.prevent_close();
                if let Err(e) = window.hide() {
                    tracing::error!("Failed to hide settings window: {e}");
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            cmd::config::get_config,
            cmd::scheduler::pause_scheduler,
            cmd::scheduler::resume_scheduler,
            cmd::scheduler::postpone_break,
            cmd::scheduler::trigger_break,
            cmd::scheduler::skip_break,
            cmd::scheduler::request_scheduler_status,
            cmd::config::save_config,
            cmd::config::pick_background_image,
            cmd::audio::play_audio,
            cmd::audio::play_builtin_audio,
            cmd::audio::stop_audio,
            cmd::system::open_config_directory,
            cmd::suggestions::get_suggestions,
            cmd::suggestions::get_suggestions_for_language,
            cmd::suggestions::save_suggestions,
            cmd::payload::store_break_payload,
            cmd::payload::get_break_payload,
            cmd::payload::remove_break_payload,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
