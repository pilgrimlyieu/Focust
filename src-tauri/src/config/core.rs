use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use figment::{
    Figment,
    providers::{Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::fs as a_fs;

use crate::config::{AdvancedConfig, AppConfig};

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
    // Create a wrapper struct to properly serialize with advanced section
    #[derive(Serialize)]
    struct ConfigFile<'a> {
        #[serde(flatten)]
        config: &'a AppConfig,
        advanced: &'a AdvancedConfig,
    }

    let config_path = get_config_path(app_handle)?;

    let config_file = ConfigFile {
        config,
        advanced: &config.advanced,
    };

    // Serialize config to TOML string
    let toml_string =
        toml::to_string_pretty(&config_file).context("Failed to serialize config to TOML")?;

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

    // Load main config (without advanced section)
    let mut config: AppConfig = Figment::new()
        .merge(Serialized::defaults(AppConfig::default()))
        .merge(Toml::file(&config_path))
        .extract()
        .map_err(|e| {
            tracing::error!(
                "Failed to parse config file at {}: {e:#}",
                config_path.display(),
            );

            anyhow::Error::new(e).context(format!(
                "Failed to load config from {}",
                config_path.display()
            ))
        })?;

    // Load advanced config from separate section
    config.advanced = load_advanced_config(&config_path).await?;

    tracing::info!("Config loaded successfully from {}", config_path.display());
    tracing::debug!("Advanced config: {:?}", config.advanced);

    Ok(config)
}

/// Load advanced configuration from the `[advanced]` section
async fn load_advanced_config(config_path: &PathBuf) -> Result<AdvancedConfig> {
    // Try to extract the advanced section, fall back to default if not present
    #[derive(Deserialize)]
    struct ConfigFile {
        #[serde(default)]
        advanced: AdvancedConfig,
    }

    let file_content = a_fs::read_to_string(config_path)
        .await
        .context("Failed to read config file for advanced section")?;

    let config_file: ConfigFile = toml::from_str(&file_content)
        .context("Failed to parse advanced section from config file")?;

    Ok(config_file.advanced)
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
        assert!(!config.autostart); // default is false
        assert_eq!(config.inactive_s, 300); // default value
        assert_eq!(config.schedules.len(), 1); // default schedule
    }

    #[test]
    fn test_figment_error_handling() {
        let invalid_toml = r#"
            autostart = "not a boolean"
        "#;

        let result = Figment::new()
            .merge(Serialized::defaults(AppConfig::default()))
            .merge(Toml::string(invalid_toml))
            .extract::<AppConfig>();

        // Should fail due to type mismatch
        assert!(result.is_err());
    }
}
