use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use tauri::AppHandle;
use tauri::Manager;
use tokio::fs as a_fs;

use crate::config::AppConfig;

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
        tracing::info!("No config file found at {config_path:?}. Creating a default one.");
        let config = AppConfig::default();
        if let Err(e) = save_config(app_handle, &config).await {
            tracing::error!("Failed to save the default config file: {e}");
        }
        return Ok(config);
    }

    let content = tokio::fs::read_to_string(&config_path)
        .await
        .with_context(|| format!("Failed to read config file from {}", config_path.display()))?;
    let config = toml::from_str(&content).context("Failed to parse config file content as TOML")?;

    tracing::info!("Config loaded successfully from {}", config_path.display());
    Ok(config)
}

pub async fn load_config(app_handle: &AppHandle) -> AppConfig {
    try_load_or_create_config(app_handle)
        .await
        .unwrap_or_else(|e| {
            tracing::error!(
                "A critical error occurred during config loading: {e:?}. Using default config."
            );
            AppConfig::default()
        })
}

pub async fn save_config(app_handle: &AppHandle, config: &AppConfig) -> Result<()> {
    let config_path = get_config_path(app_handle)?;
    let toml_string =
        toml::to_string_pretty(config).context("Failed to serialize config to TOML")?;
    a_fs::write(&config_path, toml_string)
        .await
        .context(format!(
            "Failed to write config to {}",
            config_path.display()
        ))?;
    tracing::info!("Config saved successfully to {}", config_path.display());
    Ok(())
}
