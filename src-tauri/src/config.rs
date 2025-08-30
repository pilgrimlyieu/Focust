use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri::Manager;
use tokio::fs as a_fs;
use tokio::sync::RwLock;
use ts_rs::TS;

use crate::core::schedule::{AttentionSettings, ScheduleSettings};

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(default)]
#[ts(export, rename_all = "camelCase")]
pub struct AppConfig {
    /// If auto check for updates on startup
    pub check_for_updates: bool,
    /// If monitor DND status and pause breaks
    pub monitor_dnd: bool,
    /// Inactive time in seconds before pausing breaks
    pub inactive_s: u64,
    /// If breaks should be shown on all screens
    pub all_screens: bool,
    /// Language code, e.g., "en-US"
    pub language: String,
    /// Shortcut to postpone breaks, e.g., "Ctrl+X"
    pub postpone_shortcut: String,
    /// List of schedules
    pub schedules: Vec<ScheduleSettings>,
    /// List of attention reminders
    pub attentions: Vec<AttentionSettings>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            check_for_updates: true,
            monitor_dnd: true,
            inactive_s: 300,
            all_screens: false,
            language: "en-US".to_string(), // TODO: Change to system language
            postpone_shortcut: "Ctrl+X".to_string(),
            schedules: vec![ScheduleSettings::default()], // Providing a default schedule
            attentions: vec![],                           // Providing no default attentions
        }
    }
}

pub type SharedConfig = RwLock<AppConfig>;

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

pub async fn load_config(app_handle: &AppHandle) -> AppConfig {
    let config_path = match get_config_path(app_handle) {
        Ok(path) => path,
        Err(e) => {
            log::error!("Failed to get config path: {e}. Using default config.");
            return AppConfig::default();
        }
    };

    if !config_path.exists() {
        log::info!("No config file found. Creating a default one.");
        let config = AppConfig::default();
        if let Err(e) = save_config(app_handle, &config).await {
            log::error!("Failed to save default config: {e}");
        }
        return config;
    }

    match a_fs::read_to_string(&config_path).await {
        Ok(content) => match toml::from_str(&content) {
            Ok(config) => {
                log::info!("Config loaded successfully from {config_path:?}");
                config
            }
            Err(e) => {
                log::error!("Failed to parse config file: {e}. Using default.");
                AppConfig::default()
            }
        },
        Err(e) => {
            log::error!("Failed to read config file: {e}. Using default.");
            AppConfig::default()
        }
    }
}

pub async fn save_config(app_handle: &AppHandle, config: &AppConfig) -> Result<()> {
    let config_path = get_config_path(app_handle)?;
    let toml_string =
        toml::to_string_pretty(config).context("Failed to serialize config to TOML")?;
    a_fs::write(&config_path, toml_string)
        .await
        .context(format!("Failed to write config to {config_path:?}"))?;
    log::info!("Config saved successfully to {config_path:?}");
    Ok(())
}
