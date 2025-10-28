use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use ts_rs::TS;

/// Settings for displaying suggestions during breaks
///
/// This controls whether suggestions are shown to the user during break windows.
/// The actual suggestion content is managed separately in the [SuggestionsConfig].
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct SuggestionsSettings {
    /// Whether to show suggestions during breaks
    pub show: bool,
}

impl Default for SuggestionsSettings {
    fn default() -> Self {
        SuggestionsSettings {
            show: true, // Show suggestions by default
        }
    }
}

/// Suggestions configuration loaded from suggestions.toml
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct SuggestionsConfig {
    /// Suggestions grouped by language code
    pub by_language: HashMap<String, LanguageSuggestions>,
}

/// Suggestions for a specific language
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct LanguageSuggestions {
    pub suggestions: Vec<String>,
}

impl Default for SuggestionsConfig {
    // TODO: Move i18n suggestions to a separate file for easier management
    fn default() -> Self {
        let mut by_language = HashMap::new();

        // English suggestions (default)
        by_language.insert(
            "en-US".to_string(),
            LanguageSuggestions {
                suggestions: vec![
                    "Look away from your screen and focus on a distant object.".to_string(),
                    "Roll your shoulders back and take a deep breath.".to_string(),
                    "Drink a glass of water.".to_string(),
                    "Stand up and stretch your legs.".to_string(),
                    "Relax your jaw and unclench your teeth.".to_string(),
                    "Gently stretch your wrists and fingers.".to_string(),
                    "Let your eyes rest by closing them for a few seconds.".to_string(),
                    "Take ten slow breaths, counting each inhale and exhale.".to_string(),
                    "Notice five things around you that you can see.".to_string(),
                    "Walk around your room or office.".to_string(),
                ],
            },
        );

        // Chinese suggestions
        by_language.insert(
            "zh-CN".to_string(),
            LanguageSuggestions {
                suggestions: vec![
                    "将目光从屏幕移开，专注于远处的物体。".to_string(),
                    "向后转动肩膀，深呼吸。".to_string(),
                    "喝一杯水。".to_string(),
                    "站起来伸展腿部。".to_string(),
                    "放松下巴，松开咬紧的牙齿。".to_string(),
                    "轻轻伸展手腕和手指。".to_string(),
                    "闭上眼睛几秒钟，让眼睛休息。".to_string(),
                    "慢慢呼吸十次，数每次吸气和呼气。".to_string(),
                    "注意周围你能看到的五样东西。".to_string(),
                    "在房间或办公室里走动。".to_string(),
                ],
            },
        );

        Self { by_language }
    }
}

/// Get the path to suggestions.toml file
fn get_suggestions_path(app_handle: &AppHandle) -> Result<PathBuf> {
    let config_dir = app_handle
        .path()
        .app_config_dir()
        .context("Failed to get app config directory")?;
    Ok(config_dir.join("suggestions.toml"))
}

/// Load suggestions from suggestions.toml or create default if not exists
pub async fn load_suggestions(app_handle: &AppHandle) -> SuggestionsConfig {
    match try_load_suggestions(app_handle).await {
        Ok(config) => {
            tracing::info!("Suggestions loaded successfully");
            config
        }
        Err(e) => {
            tracing::warn!("Failed to load suggestions, using defaults: {e:#}");
            let default = SuggestionsConfig::default();

            // Try to save default config
            if let Err(save_err) = save_suggestions(app_handle, &default).await {
                tracing::error!("Failed to save default suggestions: {save_err:#}");
            }

            default
        }
    }
}

/// Try to load suggestions from file
async fn try_load_suggestions(app_handle: &AppHandle) -> Result<SuggestionsConfig> {
    let suggestions_path = get_suggestions_path(app_handle)?;

    if !suggestions_path.exists() {
        anyhow::bail!("Suggestions file does not exist");
    }

    let content = tokio::fs::read_to_string(&suggestions_path)
        .await
        .with_context(|| format!("Failed to read suggestions from {suggestions_path:?}"))?;

    let config: SuggestionsConfig =
        toml::from_str(&content).context("Failed to parse suggestions.toml")?;

    Ok(config)
}

/// Save suggestions to file
pub async fn save_suggestions(app_handle: &AppHandle, config: &SuggestionsConfig) -> Result<()> {
    let suggestions_path = get_suggestions_path(app_handle)?;

    // Ensure parent directory exists
    if let Some(parent) = suggestions_path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).context("Failed to create suggestions directory")?;
    }

    let toml_string =
        toml::to_string_pretty(config).context("Failed to serialize suggestions to TOML")?;

    tokio::fs::write(&suggestions_path, toml_string)
        .await
        .with_context(|| format!("Failed to write suggestions to {suggestions_path:?}"))?;

    tracing::info!("Suggestions saved successfully to {suggestions_path:?}");
    Ok(())
}

/// Get suggestions for a specific language
/// Falls back to en-US if language not found
pub fn get_suggestions_for_language(config: &SuggestionsConfig, language: &str) -> Vec<String> {
    if let Some(lang_suggestions) = config.by_language.get(language) {
        return lang_suggestions.suggestions.clone();
    }

    // Fallback to en-US
    if let Some(en_suggestions) = config.by_language.get("en-US") {
        return en_suggestions.suggestions.clone();
    }

    // Last resort: empty vec
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggestions_settings_default() {
        let settings = SuggestionsSettings::default();
        assert!(settings.show, "Default should show suggestions");
    }

    #[test]
    fn test_suggestions_settings_disabled() {
        let settings = SuggestionsSettings { show: false };
        assert!(!settings.show, "Should not show suggestions when disabled");
    }

    #[test]
    fn test_suggestions_settings_clone() {
        let settings = SuggestionsSettings::default();
        let cloned = settings.clone();
        assert_eq!(
            settings.show, cloned.show,
            "Clone should preserve show value"
        );
    }

    #[test]
    fn test_suggestions_settings_serialization() {
        let settings = SuggestionsSettings { show: true };
        let json = serde_json::to_string(&settings).expect("Failed to serialize");
        let deserialized: SuggestionsSettings =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(
            settings.show, deserialized.show,
            "Serialization roundtrip failed"
        );
    }

    #[test]
    fn test_suggestions_settings_camel_case_serialization() {
        let settings = SuggestionsSettings { show: true };
        let json = serde_json::to_string(&settings).expect("Failed to serialize");
        // Verify camelCase is used in JSON
        assert!(
            json.contains("\"show\""),
            "JSON should contain 'show' field"
        );
    }

    #[test]
    fn test_suggestions_settings_toml_serialization() {
        let settings = SuggestionsSettings { show: false };
        let toml = toml::to_string(&settings).expect("Failed to serialize to TOML");
        let deserialized: SuggestionsSettings =
            toml::from_str(&toml).expect("Failed to deserialize from TOML");
        assert_eq!(
            settings.show, deserialized.show,
            "TOML serialization roundtrip failed"
        );
    }

    #[test]
    fn test_suggestions_config_default() {
        let config = SuggestionsConfig::default();

        assert!(config.by_language.contains_key("en-US"));
        assert!(config.by_language.contains_key("zh-CN"));

        let en_suggestions = &config.by_language["en-US"].suggestions;
        assert!(!en_suggestions.is_empty());
    }

    #[test]
    fn test_get_suggestions_for_language() {
        let config = SuggestionsConfig::default();

        // Test existing language
        let en_suggestions = get_suggestions_for_language(&config, "en-US");
        assert!(!en_suggestions.is_empty());

        let zh_suggestions = get_suggestions_for_language(&config, "zh-CN");
        assert!(!zh_suggestions.is_empty());

        // Test fallback to en-US
        let unknown_suggestions = get_suggestions_for_language(&config, "unknown");
        assert_eq!(unknown_suggestions, en_suggestions);
    }

    #[test]
    fn test_suggestions_config_serialization() {
        let config = SuggestionsConfig::default();

        let toml_string = toml::to_string_pretty(&config).expect("Failed to serialize");

        // Check for actual structure (camelCase)])
        assert!(toml_string.contains("byLanguage"));
        assert!(toml_string.contains("suggestions"));

        let deserialized: SuggestionsConfig =
            toml::from_str(&toml_string).expect("Failed to deserialize");

        assert_eq!(config.by_language.len(), deserialized.by_language.len());
    }
}
