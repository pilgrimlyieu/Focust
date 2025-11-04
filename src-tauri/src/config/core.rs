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

    // Try to parse the config file
    match toml::from_str::<AppConfig>(&content) {
        Ok(config) => {
            tracing::info!("Config loaded successfully from {}", config_path.display());
            Ok(config)
        }
        Err(parse_err) => {
            // If parsing fails, try to merge with default config
            tracing::warn!(
                "Config file parsing failed: {parse_err}. Attempting partial config load..."
            );

            // Parse as a generic toml::Value to extract valid fields
            let parsed_toml: toml::Value = toml::from_str(&content).with_context(|| {
                format!("Config file is not valid TOML at {}", config_path.display())
            })?;

            // Start with default config
            let mut config = AppConfig::default();

            // Try to merge each field individually
            if let toml::Value::Table(table) = parsed_toml {
                merge_config_field(&mut config, &table);
            }

            tracing::info!(
                "Partial config loaded from {}. Missing fields filled with defaults.",
                config_path.display()
            );

            // Save the merged config back to ensure all fields are present
            if let Err(e) = save_config(app_handle, &config).await {
                tracing::warn!("Failed to save merged config: {e}");
            }

            Ok(config)
        }
    }
}

/// Merge individual config fields from TOML table into `AppConfig`
fn merge_config_field(config: &mut AppConfig, table: &toml::map::Map<String, toml::Value>) {
    // Helper macro to safely extract and set fields
    macro_rules! merge_field {
        ($field:ident, $toml_key:expr, bool) => {
            if let Some(toml::Value::Boolean(val)) = table.get($toml_key) {
                config.$field = *val;
            }
        };
        ($field:ident, $toml_key:expr, u32) => {
            if let Some(toml::Value::Integer(val)) = table.get($toml_key) {
                config.$field = (*val).max(0) as u32;
            }
        };
        ($field:ident, $toml_key:expr, f32) => {
            if let Some(toml::Value::Float(val)) = table.get($toml_key) {
                config.$field = *val as f32;
            }
        };
        ($field:ident, $toml_key:expr, String) => {
            if let Some(toml::Value::String(val)) = table.get($toml_key) {
                config.$field = val.clone();
            }
        };
    }

    // Merge top-level fields
    merge_field!(check_for_updates, "checkForUpdates", bool);
    merge_field!(autostart, "autostart", bool);
    merge_field!(monitor_dnd, "monitorDnd", bool);
    merge_field!(inactive_s, "inactiveS", u32);
    merge_field!(all_screens, "allScreens", bool);
    merge_field!(language, "language", String);
    merge_field!(theme_mode, "themeMode", String);
    merge_field!(postpone_shortcut, "postponeShortcut", String);
    merge_field!(window_size, "windowSize", f32);

    // Merge schedules array
    if let Some(toml::Value::Array(schedules_array)) = table.get("schedules")
        && let Ok(schedules) = schedules_array
            .iter()
            .map(|v| toml::from_str::<crate::core::schedule::ScheduleSettings>(&v.to_string()))
            .collect::<Result<Vec<_>, _>>()
        && !schedules.is_empty()
    {
        config.schedules = schedules;
    }

    // Merge attentions array
    if let Some(toml::Value::Array(attentions_array)) = table.get("attentions")
        && let Ok(attentions) = attentions_array
            .iter()
            .map(|v| toml::from_str::<crate::core::schedule::AttentionSettings>(&v.to_string()))
            .collect::<Result<Vec<_>, _>>()
    {
        config.attentions = attentions;
    }
}

pub async fn load_config(app_handle: &AppHandle) -> AppConfig {
    try_load_or_create_config(app_handle)
        .await
        .unwrap_or_else(|e| {
            tracing::error!(
                "A critical error occurred during config loading: {e}. Using default config."
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

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_config_field_basic_types() {
        let mut config = AppConfig::default();

        // Create TOML table with mixed fields
        let toml_str = r#"
            checkForUpdates = false
            inactiveS = 600
            windowSize = 0.5
            language = "zh-CN"
        "#;

        let parsed: toml::Value = toml::from_str(toml_str).unwrap();
        if let toml::Value::Table(table) = parsed {
            merge_config_field(&mut config, &table);
        }

        assert!(!config.check_for_updates);
        assert_eq!(config.inactive_s, 600);
        assert_eq!(config.window_size, 0.5);
        assert_eq!(config.language, "zh-CN");
    }

    #[test]
    fn test_merge_config_field_preserves_defaults() {
        let mut config = AppConfig::default();
        let original_schedules_count = config.schedules.len();

        // Merge with only one field
        let toml_str = r#"
            language = "en-US"
        "#;

        let parsed: toml::Value = toml::from_str(toml_str).unwrap();
        if let toml::Value::Table(table) = parsed {
            merge_config_field(&mut config, &table);
        }

        // Only language should change, others remain default
        assert_eq!(config.language, "en-US");
        assert!(config.check_for_updates); // Default is true
        assert_eq!(config.schedules.len(), original_schedules_count);
    }

    #[test]
    fn test_merge_config_field_invalid_types_ignored() {
        let mut config = AppConfig::default();
        let original_inactive = config.inactive_s;

        // Try to set invalid type (string instead of integer)
        let toml_str = r#"
            inactiveS = "invalid"
        "#;

        let parsed: toml::Value = toml::from_str(toml_str).unwrap();
        if let toml::Value::Table(table) = parsed {
            merge_config_field(&mut config, &table);
        }

        // Should remain unchanged
        assert_eq!(config.inactive_s, original_inactive);
    }
}
