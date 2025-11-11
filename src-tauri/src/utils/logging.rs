use std::path::PathBuf;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize the logging system
///
/// Sets up tracing with both console and file output
/// Log files are rotated daily and stored in the app's data directory
///
/// # Arguments
/// * `log_dir` - Directory to store log files
/// * `log_level` - Default log level (e.g., "info", "debug", "trace")
pub fn init_logging(log_dir: &PathBuf, log_level: &str) -> Result<(), String> {
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

    // Parse log level from string
    let level = match log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    // Create an environment filter
    let mut env_filter = EnvFilter::from_default_env()
        .add_directive(level.into())
        .add_directive(
            // https://github.com/tauri-apps/tauri/issues/8494
            "tao::platform_impl::platform::event_loop::runner=off"
                .parse()
                .unwrap(),
        );

    // default log level for our own crate
    if cfg!(debug_assertions) {
        env_filter = env_filter.add_directive("focust=trace".parse().unwrap());
    } else {
        env_filter = env_filter.add_directive("focust=info".parse().unwrap());
    }

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
