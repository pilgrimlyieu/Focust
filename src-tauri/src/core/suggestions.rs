use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use rand::prelude::IndexedRandom;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use ts_rs::TS;

use crate::platform::i18n::LANGUAGE_FALLBACK;

/// Settings for displaying suggestions during breaks
///
/// This controls whether suggestions are shown to the user during break windows.
/// The actual suggestion content is managed separately in the [`SuggestionsConfig`].
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

/// Global shared suggestions state
pub type SharedSuggestions = tokio::sync::RwLock<SuggestionsConfig>;

/// Suggestions for a specific language
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct LanguageSuggestions {
    /// List of suggestions
    pub suggestions: Vec<String>,
}

impl Default for SuggestionsConfig {
    fn default() -> Self {
        // Load from embedded resource file
        // This should never fail
        load_default_suggestions().unwrap_or_else(|e| {
            tracing::error!("Failed to load embedded suggestions.toml: {e}");
            tracing::error!("This should never happen");
            // Return empty config as last resort
            Self {
                by_language: HashMap::new(),
            }
        })
    }
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
        .with_context(|| {
            format!(
                "Failed to read suggestions from {}",
                suggestions_path.display()
            )
        })?;

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
        .with_context(|| {
            format!(
                "Failed to write suggestions to {}",
                suggestions_path.display()
            )
        })?;

    tracing::info!("Suggestions saved successfully to {suggestions_path:?}");
    Ok(())
}

/// Get suggestions for a specific language
/// Falls back to en-US if language not found
#[must_use]
pub fn get_suggestions_for_language(config: &SuggestionsConfig, language: &str) -> Vec<String> {
    if let Some(lang_suggestions) = config.by_language.get(language) {
        return lang_suggestions.suggestions.clone();
    }

    // Fallback to en-US
    if let Some(en_suggestions) = config.by_language.get(LANGUAGE_FALLBACK) {
        return en_suggestions.suggestions.clone();
    }

    // Last resort: empty vec
    vec![]
}

/// Sample a random suggestion for a specific language
/// Returns None if no suggestions available
#[must_use]
pub fn sample_suggestion(config: &SuggestionsConfig, language: &str) -> Option<String> {
    let suggestions = get_suggestions_for_language(config, language);
    if suggestions.is_empty() {
        return None;
    }

    let mut rng = rand::rng();
    suggestions.choose(&mut rng).cloned()
}

/// Load default suggestions from embedded resource file
fn load_default_suggestions() -> Result<SuggestionsConfig> {
    // The resource file will be embedded in the binary by Tauri
    // and available at runtime via the resource protocol
    let default_toml = include_str!("../../resources/suggestions.toml");
    let config: SuggestionsConfig =
        toml::from_str(default_toml).context("Failed to parse default suggestions.toml")?;
    Ok(config)
}

/// Get the path to suggestions.toml file
fn get_suggestions_path(app_handle: &AppHandle) -> Result<PathBuf> {
    let config_dir = app_handle
        .path()
        .app_config_dir()
        .context("Failed to get app config directory")?;
    Ok(config_dir.join("suggestions.toml"))
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

        // Check for actual structure (camelCase)
        assert!(toml_string.contains("byLanguage"));
        assert!(toml_string.contains("suggestions"));

        let deserialized: SuggestionsConfig =
            toml::from_str(&toml_string).expect("Failed to deserialize");

        assert_eq!(config.by_language.len(), deserialized.by_language.len());
    }
}
