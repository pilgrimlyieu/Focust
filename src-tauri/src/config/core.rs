use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use figment::{
    Figment,
    providers::{Format, Serialized, Toml},
};
use tauri::{AppHandle, Manager};
use tokio::fs as a_fs;

use crate::config::AppConfig;

pub async fn load_config(app_handle: &AppHandle) -> AppConfig {
    try_load_or_create_config(app_handle)
        .await
        .inspect_err(|e| {
            tracing::error!(
                "A critical error occurred during config loading: {e}. Using default config."
            );
        })
        .unwrap_or_default()
}

pub async fn save_config(app_handle: &AppHandle, config: &AppConfig) -> Result<()> {
    let config_path = get_config_path(app_handle)?;

    // Serialize config to TOML string
    let toml_string =
        toml::to_string_pretty(config).context("Failed to serialize config to TOML")?;

    // Write to file
    a_fs::write(&config_path, toml_string)
        .await
        .with_context(|| format!("Failed to write config to {}", config_path.display()))?;

    tracing::info!("Config saved successfully to {}", config_path.display());
    Ok(())
}

/// Get the path to the config file, creating the config directory if it doesn't exist
fn get_config_path(app_handle: &AppHandle) -> Result<PathBuf> {
    let config_dir = app_handle
        .path()
        .app_config_dir()
        .context("Failed to get app config directory")?;
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
    }
    Ok(config_dir.join("config.toml"))
}

async fn try_load_or_create_config(app_handle: &AppHandle) -> Result<AppConfig> {
    let config_path = get_config_path(app_handle)?;

    if !config_path.exists() {
        tracing::info!(
            "No config file found at {}. Creating a default one.",
            config_path.display()
        );
        let config = AppConfig::default();
        save_config(app_handle, &config).await.unwrap_or_else(|e| {
            tracing::error!("Failed to save the default config file: {e}");
        });
        return Ok(config);
    }

    let config = Figment::new()
        .merge(Serialized::defaults(AppConfig::default()))
        .merge(Toml::file(&config_path))
        .extract()
        .with_context(|| format!("Failed to extract config from {}", config_path.display()))?;

    tracing::info!("Config loaded successfully from {}", config_path.display());
    Ok(config)
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_config_with_figment() {
        // Test that figment correctly fills missing fields with defaults
        let partial_toml = r#"
            # User's partial config
            language = "zh-CN"
            windowSize = 0.7
        "#;

        let config: AppConfig = Figment::new()
            .merge(Serialized::defaults(AppConfig::default()))
            .merge(Toml::string(partial_toml))
            .extract()
            .expect("Failed to extract config");

        // User-provided fields should be set
        assert_eq!(config.language, "zh-CN");
        assert_eq!(config.window_size, 0.7);

        // Missing fields should use defaults
        assert!(config.check_for_updates); // default is true
        assert_eq!(config.inactive_s, 300); // default value
        assert_eq!(config.schedules.len(), 1); // default schedule
    }

    #[test]
    fn test_figment_error_handling() {
        let invalid_toml = r#"
            checkForUpdates = "not a boolean"
        "#;

        let result = Figment::new()
            .merge(Serialized::defaults(AppConfig::default()))
            .merge(Toml::string(invalid_toml))
            .extract::<AppConfig>();

        // Should fail due to type mismatch
        assert!(result.is_err());
    }
}
