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
#[serde(default, rename_all = "camelCase")]
pub struct TrayStrings {
    pub show: String,
    pub pause: String,
    pub resume: String,
    pub pause_for: String,
    pub start_break_now: String,
    pub restart: String,
    pub quit: String,
    pub tooltip: String,
    pub minute_short: String,
    pub remaining_minutes: String,
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
            show: "Show Settings".to_owned(),
            pause: "Pause Breaks".to_owned(),
            resume: "Resume Breaks".to_owned(),
            pause_for: "Pause for".to_owned(),
            start_break_now: "Start Break Now".to_owned(),
            restart: "Restart".to_owned(),
            quit: "Quit".to_owned(),
            tooltip: "Focust - Break Reminder".to_owned(),
            minute_short: "min".to_owned(),
            remaining_minutes: "{minutes} min left".to_owned(),
        }
    }
}

impl Default for NotificationStrings {
    fn default() -> Self {
        Self {
            mini_break: "Mini Break".to_owned(),
            long_break: "Long Break".to_owned(),
            attention: "Attention".to_owned(),
            starting_soon: "{breakType} in {seconds} seconds".to_owned(),
            message: "Time to take a break and rest your eyes.".to_owned(),
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
