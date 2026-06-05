//! Tauri command handlers for frontend-backend communication.
//!
//! This module contains all Tauri commands that can be invoked from the frontend
//! via the IPC bridge. Commands are organized into logical submodules:
//!
//! - **`audio`**: Audio playback control
//! - **`autostart`**: System autostart management
//! - **`config`**: Application configuration CRUD
//! - **`payload`**: Break prompt payload storage
//! - **`scheduler`**: Scheduler state management and control
//! - **`suggestions`**: Break suggestion management
//! - **`system`**: System-level operations (directory access, application lifecycle)
//! - **`window`**: Window management (open/close)

pub mod audio;
pub mod autostart;
pub mod config;
pub mod payload;
pub mod scheduler;
pub mod suggestions;
pub mod system;
pub mod window;

pub use audio::{play_audio, play_builtin_audio, stop_audio};
pub use autostart::{is_autostart_enabled, set_autostart_enabled};
pub use config::{get_config, pick_background_image, save_config};
pub use payload::{get_prompt_payload, remove_prompt_payload, store_prompt_payload};
pub use scheduler::{
    SchedulerCmd, ShutdownTx, pause_scheduler, postpone_break, prompt_finished, resume_scheduler,
};
pub use suggestions::{get_suggestions, save_suggestions};
pub use system::{
    exit_application, open_config_directory, open_log_directory, restart_application,
};
pub use window::close_all_prompt_windows;
