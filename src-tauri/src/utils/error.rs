use std::error;
/// Centralized error handling for Focust
///
/// This module defines custom error types and conversion utilities.
///
/// # Error Handling Strategy
///
/// - **Library code**: Use typed errors with `thiserror`
/// - **Application code**: Use `anyhow::Result` for convenience
/// - **Tauri commands**: Use `Result<T, String>` for fallible commands and
///   `InfallibleCommandResult<T>` for commands that cannot fail
/// - **Conversions**: Use `IntoTauriError` trait to convert at boundaries
use std::io;
use std::path::PathBuf;

use serde::{Serialize, Serializer};
use thiserror::Error;

/// Uninhabited error adapter for infallible Tauri commands.
///
/// Async commands borrowing `State` must return `Result`, with serializable
/// errors. Serde's stable features do not yet implement `Serialize` for `!`.
///
/// TODO: Once Serde implements `Serialize` for `!` without its `unstable`
/// feature, change `InfallibleCommandResult<T>` to `Result<T, !>` and remove
/// this wrapper, its manual `Serialize` implementation, and its re-export.
#[derive(Debug)]
pub struct InfallibleCommandError(pub !);

impl Serialize for InfallibleCommandError {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0
    }
}

/// Result for Tauri command implementations that cannot fail.
pub type InfallibleCommandResult<T> = Result<T, InfallibleCommandError>;

/// Main error type for Focust library operations
#[derive(Error, Debug)]
pub enum FocustError {
    /// IO errors (file operations, etc.)
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Audio playback errors
    #[error("Audio error: {0}")]
    Audio(String),

    /// Path resolution errors
    #[error("Path error: {0}")]
    Path(String),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Generic error with context
    #[error("{0}")]
    Other(String),
}

/// Configuration-specific errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file at {path}: {source}")]
    ReadError {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Failed to write config file at {path}: {source}")]
    WriteError {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Failed to parse config file at {path}: {message}")]
    ParseError { path: PathBuf, message: String },

    #[error("Failed to get config directory: {0}")]
    DirectoryError(String),

    #[error("Invalid config value: {0}")]
    ValidationError(String),
}

/// Audio-specific errors
#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Failed to initialize audio device: {0}")]
    DeviceError(String),

    #[error("Failed to decode audio file at {path}: {message}")]
    DecodeError { path: PathBuf, message: String },

    #[error("Failed to play audio: {0}")]
    PlaybackError(String),

    #[error("Audio file not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Invalid audio format: {0}")]
    InvalidFormat(String),
}

/// Trait for converting errors to Tauri-compatible string errors.
///
/// Automatically implemented for all types implementing `std::error::Error`.
pub trait IntoTauriError {
    /// Converts the error to a `String` for Tauri IPC.
    fn into_tauri_error(self) -> String;
}

impl<E: error::Error> IntoTauriError for E {
    fn into_tauri_error(self) -> String {
        self.to_string()
    }
}

/// Converts a `Result` with any error type to a Tauri-compatible `Result<T, String>`.
///
/// # Errors
///
/// Returns `Err(String)` if the input result is an error, with the error message
/// converted to a string.
pub fn to_tauri_result<T, E: error::Error>(result: Result<T, E>) -> Result<T, String> {
    result.map_err(|e| e.to_string())
}

/// Macro for quick error conversion in Tauri commands.
///
/// Converts errors to strings using `to_string()`.
///
/// # Examples
///
/// ```ignore
/// #[tauri::command]
/// pub async fn my_command() -> Result<(), String> {
///     tauri_error!(some_fallible_operation())
/// }
/// ```
#[macro_export]
macro_rules! tauri_error {
    ($result:expr) => {
        $result.map_err(|e| e.to_string())
    };
    ($result:expr, $context:expr) => {
        $result.map_err(|e| format!("{}: {}", $context, e))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focust_error_display() {
        let err = FocustError::Config("Invalid value".to_owned());
        assert_eq!(err.to_string(), "Configuration error: Invalid value");

        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = FocustError::Io(io_err);
        assert!(err.to_string().contains("IO error"));
    }

    #[test]
    fn config_error_display() {
        let err = ConfigError::ValidationError("Missing field".to_owned());
        assert_eq!(err.to_string(), "Invalid config value: Missing field");
    }

    #[test]
    fn audio_error_display() {
        let path = PathBuf::from("/test/audio.mp3");
        let err = AudioError::FileNotFound(path.clone());
        assert_eq!(
            err.to_string(),
            format!("Audio file not found: {}", path.display())
        );
    }

    #[test]
    fn to_tauri_result_works() {
        let ok_result: Result<i32, FocustError> = Ok(42);
        let tauri_result = to_tauri_result(ok_result);
        assert_eq!(tauri_result, Ok(42));

        let err_result: Result<i32, FocustError> = Err(FocustError::Config("test".to_owned()));
        let tauri_result = to_tauri_result(err_result);
        assert!(tauri_result.is_err());
        assert_eq!(tauri_result.unwrap_err(), "Configuration error: test");
    }
}
