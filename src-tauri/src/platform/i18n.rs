use serde::Deserialize;
use std::collections::HashMap;

pub const LANGUAGE_FALLBACK: &str = "en-US";

#[derive(Debug, Deserialize, Clone)]
pub struct PlatformI18n {
    #[serde(flatten)]
    languages: HashMap<String, LanguageStrings>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct LanguageStrings {
    pub tray: TrayStrings,
    pub notification: NotificationStrings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TrayStrings {
    pub show: String,
    pub pause: String,
    pub resume: String,
    pub quit: String,
    pub tooltip: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NotificationStrings {
    pub mini_break: String,
    pub long_break: String,
    pub attention: String,
    pub starting_soon: String,
    pub message: String,
}

impl Default for TrayStrings {
    fn default() -> Self {
        Self {
            show: "Show Settings".to_string(),
            pause: "Pause Breaks".to_string(),
            resume: "Resume Breaks".to_string(),
            quit: "Quit".to_string(),
            tooltip: "Focust - Break Reminder".to_string(),
        }
    }
}

impl Default for NotificationStrings {
    fn default() -> Self {
        Self {
            mini_break: "Mini Break".to_string(),
            long_break: "Long Break".to_string(),
            attention: "Attention".to_string(),
            starting_soon: "{breakType} in {seconds} seconds".to_string(),
            message: "Time to take a break and rest your eyes.".to_string(),
        }
    }
}

/// Load platform translations from embedded JSON file
fn load_translations() -> PlatformI18n {
    const TRANSLATIONS_JSON: &str = include_str!("../../resources/i18n.json");
    serde_json::from_str(TRANSLATIONS_JSON).unwrap_or_else(|e| {
        // This should never fail
        tracing::error!("Failed to parse platform translations: {e}");
        unreachable!()
    })
}

/// Get localized strings based on language
#[must_use]
pub fn get_strings(language: &str) -> LanguageStrings {
    let translations = load_translations();

    // Get translation for language, fallback to LANGUAGE_FALLBACK, then default
    translations
        .languages
        .get(language)
        .or_else(|| translations.languages.get(LANGUAGE_FALLBACK))
        .cloned()
        .unwrap_or_default()
}
