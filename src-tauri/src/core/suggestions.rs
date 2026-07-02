use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use rand::prelude::IndexedRandom;
use serde::{Deserialize, Deserializer, Serialize};
use tauri::AppHandle;
use tokio::fs as tokio_fs;
use tokio::sync::RwLock;
use ts_rs::TS;

use crate::core::break_kind::BreakKind;
use crate::platform::i18n::LANGUAGE_FALLBACK;
use crate::utils;

/// Settings for displaying suggestions during breaks
///
/// This controls whether suggestions are shown to the user during prompt windows.
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
pub type SharedSuggestions = RwLock<SuggestionsConfig>;

/// Suggestions for a specific language
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct LanguageSuggestions {
    /// Legacy list kept empty in new saves so older versions can still parse the file
    pub suggestions: Vec<String>,
    /// Suggestions suitable for short breaks
    pub short_suggestions: Vec<String>,
    /// Suggestions suitable for long breaks
    pub long_suggestions: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawLanguageSuggestions {
    suggestions: Option<Vec<String>>,
    short_suggestions: Option<Vec<String>>,
    long_suggestions: Option<Vec<String>>,
}

impl<'de> Deserialize<'de> for LanguageSuggestions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawLanguageSuggestions::deserialize(deserializer)?;
        let legacy_suggestions = raw.suggestions.unwrap_or_default();
        let short_suggestions = raw
            .short_suggestions
            .unwrap_or_else(|| legacy_suggestions.clone());
        let long_suggestions = raw
            .long_suggestions
            .unwrap_or_else(|| legacy_suggestions.clone());

        let mut suggestions = LanguageSuggestions {
            suggestions: legacy_suggestions,
            short_suggestions,
            long_suggestions,
        };
        suggestions.clear_legacy_suggestions_for_output();
        Ok(suggestions)
    }
}

impl LanguageSuggestions {
    /// Return the list for a specific break duration.
    #[must_use]
    pub fn for_break(&self, break_kind: BreakKind) -> &[String] {
        match break_kind {
            BreakKind::Mini => &self.short_suggestions,
            BreakKind::Long => &self.long_suggestions,
        }
    }

    fn clear_legacy_suggestions_for_output(&mut self) {
        self.suggestions.clear();
    }
}

impl SuggestionsConfig {
    /// Clear the legacy compatibility lists before returning or writing config.
    pub fn clear_legacy_suggestions_for_output(&mut self) {
        for language_suggestions in self.by_language.values_mut() {
            language_suggestions.clear_legacy_suggestions_for_output();
        }
    }
}

impl Default for SuggestionsConfig {
    fn default() -> Self {
        // Load from embedded resource file
        // This should never fail
        load_default_suggestions()
            .inspect_err(|e| {
                tracing::error!("Failed to load embedded suggestions.toml: {e}");
                unreachable!("This should never happen because the resource is embedded");
            })
            .unwrap_or_default()
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
            tracing::warn!("Failed to load suggestions, using defaults: {e}");
            let default = SuggestionsConfig::default();

            // Try to save default config
            if let Err(e) = save_suggestions_internal(app_handle, &default).await {
                tracing::error!("Failed to save default suggestions: {e}");
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

    let content = tokio_fs::read_to_string(&suggestions_path)
        .await
        .with_context(|| {
            format!(
                "Failed to read suggestions from {}",
                suggestions_path.display()
            )
        })?;

    toml::from_str(&content).context("Failed to parse suggestions.toml")
}

/// Saves suggestions to file.
///
/// # Errors
///
/// Returns an error if:
/// - Getting the suggestions path fails
/// - Creating the parent directory fails
/// - Serializing the configuration to TOML fails
/// - Writing the file fails
pub async fn save_suggestions_internal(
    app_handle: &AppHandle,
    config: &SuggestionsConfig,
) -> Result<SuggestionsConfig> {
    let suggestions_path = get_suggestions_path(app_handle)?;
    let mut config = config.clone();
    config.clear_legacy_suggestions_for_output();

    // Ensure parent directory exists
    if let Some(parent) = suggestions_path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).context("Failed to create suggestions directory")?;
    }

    let toml_string =
        toml::to_string_pretty(&config).context("Failed to serialize suggestions to TOML")?;

    tokio_fs::write(&suggestions_path, toml_string)
        .await
        .with_context(|| {
            format!(
                "Failed to write suggestions to {}",
                suggestions_path.display()
            )
        })?;

    tracing::info!(
        "Suggestions saved successfully to {}",
        suggestions_path.display()
    );
    Ok(config)
}

/// Get suggestions for a specific language and break duration.
/// Falls back to en-US if language not found.
#[must_use]
pub fn get_suggestions_for_break_internal(
    config: &SuggestionsConfig,
    language: &str,
    break_kind: BreakKind,
) -> Vec<String> {
    if let Some(lang_suggestions) = config.by_language.get(language) {
        return lang_suggestions.for_break(break_kind).to_vec();
    }

    if let Some(en_suggestions) = config.by_language.get(LANGUAGE_FALLBACK) {
        return en_suggestions.for_break(break_kind).to_vec();
    }

    vec![]
}

/// Sample a random suggestion for a specific break duration.
/// Returns None if no suggestions are available.
#[must_use]
pub fn sample_suggestion_for_break(
    config: &SuggestionsConfig,
    language: &str,
    break_kind: BreakKind,
) -> Option<String> {
    let suggestions = get_suggestions_for_break_internal(config, language, break_kind);
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
    toml::from_str(default_toml).context("Failed to parse default suggestions.toml")
}

/// Get the path to suggestions.toml file
fn get_suggestions_path(app_handle: &AppHandle) -> Result<PathBuf> {
    let config_dir =
        utils::get_app_config_dir(app_handle).context("Failed to get app config directory")?;
    Ok(config_dir.join("suggestions.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn suggestions_settings_default() {
        let settings = SuggestionsSettings::default();
        assert!(settings.show, "Default should show suggestions");
    }

    #[test]
    fn suggestions_settings_disabled() {
        let settings = SuggestionsSettings { show: false };
        assert!(!settings.show, "Should not show suggestions when disabled");
    }

    #[test]
    fn suggestions_settings_clone() {
        let settings = SuggestionsSettings::default();
        let cloned = settings.clone();
        assert_eq!(
            settings.show, cloned.show,
            "Clone should preserve show value"
        );
    }

    #[test]
    fn suggestions_settings_serialization() {
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
    fn suggestions_settings_camel_case_serialization() {
        let settings = SuggestionsSettings { show: true };
        let json = serde_json::to_string(&settings).expect("Failed to serialize");
        // Verify camelCase is used in JSON
        assert!(
            json.contains("\"show\""),
            "JSON should contain 'show' field"
        );
    }

    #[test]
    fn suggestions_settings_toml_serialization() {
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
    fn suggestions_config_default() {
        let config = SuggestionsConfig::default();

        // Test that all supported languages have suggestions
        let expected_languages = [
            "en-US", "zh-CN", "de-DE", "es-ES", "fr-FR", "it-IT", "ja-JP", "ko-KR", "pt-BR",
            "ru-RU",
        ];

        for lang in expected_languages {
            assert!(
                config.by_language.contains_key(lang),
                "Missing language: {lang}"
            );
            assert!(
                config.by_language[lang].suggestions.is_empty(),
                "Legacy suggestions should be empty for language: {lang}"
            );
            let short_suggestions = &config.by_language[lang].short_suggestions;
            assert!(
                !short_suggestions.is_empty(),
                "No short suggestions for language: {lang}"
            );
            let long_suggestions = &config.by_language[lang].long_suggestions;
            assert!(
                !long_suggestions.is_empty(),
                "No long suggestions for language: {lang}"
            );
        }
    }

    #[test]
    fn suggestions_config_serialization() {
        let config = SuggestionsConfig::default();

        let toml_string = toml::to_string_pretty(&config).expect("Failed to serialize");

        // Check for actual structure (camelCase)
        assert!(toml_string.contains("byLanguage"));
        assert!(toml_string.contains("suggestions = []"));
        assert!(toml_string.contains("shortSuggestions"));
        assert!(toml_string.contains("longSuggestions"));

        let deserialized: SuggestionsConfig =
            toml::from_str(&toml_string).expect("Failed to deserialize");

        assert_eq!(config.by_language.len(), deserialized.by_language.len());
    }

    #[test]
    fn legacy_suggestions_are_used_for_both_break_types() {
        let toml = r#"
[byLanguage.en-US]
suggestions = ["Legacy 1", "Legacy 2"]
"#;

        let config: SuggestionsConfig =
            toml::from_str(toml).expect("Failed to deserialize legacy suggestions");
        let en_suggestions = config
            .by_language
            .get("en-US")
            .expect("Missing en-US suggestions");

        assert_eq!(
            en_suggestions.short_suggestions,
            strings(&["Legacy 1", "Legacy 2"])
        );
        assert_eq!(
            en_suggestions.long_suggestions,
            strings(&["Legacy 1", "Legacy 2"])
        );
        assert!(
            en_suggestions.suggestions.is_empty(),
            "New saves should keep legacy suggestions empty"
        );
    }

    #[test]
    fn missing_split_pool_falls_back_to_legacy_per_pool() {
        let toml = r#"
[byLanguage.en-US]
suggestions = ["Legacy long"]
shortSuggestions = ["Explicit short"]

[byLanguage.zh-CN]
suggestions = ["Legacy short"]
longSuggestions = ["Explicit long"]
"#;

        let config: SuggestionsConfig =
            toml::from_str(toml).expect("Failed to deserialize mixed suggestions");
        let en_suggestions = config
            .by_language
            .get("en-US")
            .expect("Missing en-US suggestions");
        let zh_suggestions = config
            .by_language
            .get("zh-CN")
            .expect("Missing zh-CN suggestions");

        assert_eq!(
            en_suggestions.short_suggestions,
            strings(&["Explicit short"])
        );
        assert_eq!(en_suggestions.long_suggestions, strings(&["Legacy long"]));
        assert!(en_suggestions.suggestions.is_empty());

        assert_eq!(zh_suggestions.short_suggestions, strings(&["Legacy short"]));
        assert_eq!(zh_suggestions.long_suggestions, strings(&["Explicit long"]));
        assert!(zh_suggestions.suggestions.is_empty());
    }

    #[test]
    fn split_suggestions_keep_separate_pools_without_serializing_legacy_values() {
        let toml = r#"
[byLanguage.en-US]
shortSuggestions = ["Blink", "Breathe"]
longSuggestions = ["Stand up", "Drink water", "Breathe"]
"#;

        let config: SuggestionsConfig =
            toml::from_str(toml).expect("Failed to deserialize split suggestions");
        let en_suggestions = config
            .by_language
            .get("en-US")
            .expect("Missing en-US suggestions");

        assert_eq!(
            en_suggestions.short_suggestions,
            strings(&["Blink", "Breathe"])
        );
        assert_eq!(
            en_suggestions.long_suggestions,
            strings(&["Stand up", "Drink water", "Breathe"])
        );
        assert!(en_suggestions.suggestions.is_empty());
    }

    #[test]
    fn explicit_empty_pool_does_not_fall_back_to_legacy() {
        let toml = r#"
[byLanguage.en-US]
suggestions = ["Legacy"]
shortSuggestions = []
longSuggestions = ["Walk"]
"#;

        let config: SuggestionsConfig =
            toml::from_str(toml).expect("Failed to deserialize mixed suggestions");
        let en_suggestions = config
            .by_language
            .get("en-US")
            .expect("Missing en-US suggestions");

        assert!(en_suggestions.short_suggestions.is_empty());
        assert_eq!(en_suggestions.long_suggestions, strings(&["Walk"]));
        assert!(en_suggestions.suggestions.is_empty());
    }

    #[test]
    fn split_suggestions_keep_empty_legacy_shape_after_save() {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct LegacyConfig {
            by_language: HashMap<String, LegacyLanguageSuggestions>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct LegacyLanguageSuggestions {
            suggestions: Vec<String>,
        }

        let mut config = SuggestionsConfig {
            by_language: HashMap::from([(
                "en-US".to_owned(),
                LanguageSuggestions {
                    suggestions: strings(&["Legacy"]),
                    short_suggestions: strings(&["Blink"]),
                    long_suggestions: strings(&["Walk"]),
                },
            )]),
        };
        config.clear_legacy_suggestions_for_output();

        let saved_toml = toml::to_string_pretty(&config).expect("Failed to serialize suggestions");
        let legacy_config: LegacyConfig =
            toml::from_str(&saved_toml).expect("Legacy config failed to parse");
        let legacy_suggestions = legacy_config
            .by_language
            .get("en-US")
            .expect("Missing legacy en-US suggestions");

        assert!(legacy_suggestions.suggestions.is_empty());
    }

    #[test]
    fn break_specific_lookup_uses_requested_pool() {
        let config: SuggestionsConfig = toml::from_str(
            r#"
[byLanguage.en-US]
shortSuggestions = ["Short only"]
longSuggestions = ["Long only"]
"#,
        )
        .expect("Failed to deserialize split suggestions");

        let short_suggestions =
            get_suggestions_for_break_internal(&config, "en-US", BreakKind::Mini);
        let long_suggestions =
            get_suggestions_for_break_internal(&config, "en-US", BreakKind::Long);

        assert_eq!(short_suggestions, strings(&["Short only"]));
        assert_eq!(long_suggestions, strings(&["Long only"]));
    }

    #[test]
    fn break_specific_lookup_falls_back_to_default_language() {
        let config: SuggestionsConfig = toml::from_str(
            r#"
[byLanguage.en-US]
shortSuggestions = ["Default short"]
longSuggestions = ["Default long"]
"#,
        )
        .expect("Failed to deserialize split suggestions");

        let suggestions = get_suggestions_for_break_internal(&config, "missing", BreakKind::Long);

        assert_eq!(suggestions, strings(&["Default long"]));
    }
}
