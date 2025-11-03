pub mod hotkey;
pub mod notifications;
pub mod tray;
pub mod window;

pub use hotkey::register_shortcuts;
pub use notifications::{send_break_notification, send_notification};
pub use tray::setup_tray;
pub use window::{create_break_windows, create_settings_window};
