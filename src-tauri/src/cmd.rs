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
pub use suggestions::{get_suggestions, get_suggestions_for_language, save_suggestions};
pub use system::{open_config_directory, open_log_directory};
pub use window::{close_all_break_windows, open_settings_window};
