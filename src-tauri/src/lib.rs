use tauri::Manager;

use crate::{cmd::SchedulerCmd, scheduler::core::init_scheduler};

pub mod cmd;
pub mod config;
pub mod core;
pub mod scheduler;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            tokio::spawn(async move {
                let app_config = config::load_config(&handle).await;
                let shared_config = config::SharedConfig::new(app_config.clone());
                handle.manage(shared_config);

                let (cmd_tx, shutdown_tx) = init_scheduler(handle.clone()).await;
                handle.manage(SchedulerCmd(cmd_tx));
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd::pause_scheduler,
            cmd::resume_scheduler,
            cmd::postpone_break,
            cmd::save_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
