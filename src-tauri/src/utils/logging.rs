use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Log level configuration
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, EnumIter,
)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    /// Convert to tracing Level
    #[must_use]
    pub fn to_tracing_level(self) -> Level {
        match self {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }

    /// Get default log level based on build configuration
    #[must_use]
    pub fn default_for_build() -> Self {
        #[cfg(debug_assertions)]
        {
            LogLevel::Trace
        }
        #[cfg(not(debug_assertions))]
        {
            LogLevel::Info
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::default_for_build()
    }
}

/// Initialize the logging system
///
/// Sets up tracing with both console and file output
/// Log files are rotated daily and stored in the app's data directory
///
/// # Arguments
/// * `log_dir` - Directory to store log files
/// * `log_level` - Log level configuration
pub fn init_logging(log_dir: &PathBuf, log_level: LogLevel) -> Result<(), String> {
    // Create log directory if it doesn't exist
    std::fs::create_dir_all(log_dir).map_err(|e| format!("Failed to create log directory: {e}"))?;

    // Create a file appender that rotates daily
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_suffix("focust.log") // 2025-10-28.focust.log
        .max_log_files(1) // Keep only 1 day of logs, waiting for new version release: https://github.com/tokio-rs/tracing/pull/2966
        .build(log_dir)
        .expect("Failed to create log file appender");

    // Create a layer that writes to the file
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false) // Disable ANSI colors in file output
        .with_target(true)
        .with_line_number(true);

    // Create a layer that writes to stdout (for dev mode)
    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_target(true)
        .with_line_number(true);

    // Convert LogLevel to tracing Level
    let level = log_level.to_tracing_level();

    // Create an environment filter
    let mut env_filter = EnvFilter::from_default_env()
        .add_directive(level.into())
        .add_directive(
            // https://github.com/tauri-apps/tauri/issues/8494
            "tao::platform_impl::platform::event_loop::runner=off"
                .parse()
                .unwrap(),
        )
        .add_directive(
            // suppress symphonia core crate logs
            "symphonia_core::probe=off".parse().unwrap(),
        );

    // Default log level for our own crate based on configured level
    let focust_level = format!("focust={}", log_level.to_string().to_lowercase());
    env_filter = env_filter.add_directive(focust_level.parse().unwrap());

    // Initialize the subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(stdout_layer)
        .init();

    tracing::info!("Logging initialized at level: {log_level}");
    tracing::info!("Log directory: {}", log_dir.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_log_level_parsing() {
        // strum is configured with serialize_all = "lowercase"
        assert_eq!(LogLevel::from_str("trace").unwrap(), LogLevel::Trace);
        assert_eq!(LogLevel::from_str("debug").unwrap(), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("info").unwrap(), LogLevel::Info);
        assert_eq!(LogLevel::from_str("warn").unwrap(), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("error").unwrap(), LogLevel::Error);

        // Case sensitive - uppercase should fail with default strum config
        assert!(LogLevel::from_str("INFO").is_err());
        assert!(LogLevel::from_str("Debug").is_err());

        // Invalid input
        assert!(LogLevel::from_str("invalid").is_err());
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Trace.to_string(), "trace");
        assert_eq!(LogLevel::Debug.to_string(), "debug");
        assert_eq!(LogLevel::Info.to_string(), "info");
        assert_eq!(LogLevel::Warn.to_string(), "warn");
        assert_eq!(LogLevel::Error.to_string(), "error");
    }

    #[test]
    fn test_log_level_to_tracing_level() {
        assert_eq!(LogLevel::Trace.to_tracing_level(), Level::TRACE);
        assert_eq!(LogLevel::Debug.to_tracing_level(), Level::DEBUG);
        assert_eq!(LogLevel::Info.to_tracing_level(), Level::INFO);
        assert_eq!(LogLevel::Warn.to_tracing_level(), Level::WARN);
        assert_eq!(LogLevel::Error.to_tracing_level(), Level::ERROR);
    }

    #[test]
    fn test_log_level_serialization() {
        // JSON serialization
        let level = LogLevel::Info;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"info\"");

        let deserialized: LogLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, LogLevel::Info);
    }
}
