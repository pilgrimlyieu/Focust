pub mod error;
pub mod logging;

pub use error::{AudioError, ConfigError, FocustError, IntoTauriError, to_tauri_result};
pub use logging::{LogLevel, init_logging};
