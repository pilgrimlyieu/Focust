use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use ts_rs::TS;

use crate::core::schedule::{AttentionSettings, ScheduleSettings};

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct AppConfig {
    /// If auto check for updates on startup
    pub check_for_updates: bool,
    /// If monitor DND status and pause breaks
    pub monitor_dnd: bool,
    /// Inactive time in seconds before pausing breaks
    pub inactive_s: u32,
    /// If breaks should be shown on all screens
    pub all_screens: bool,
    /// Language code, e.g., "en-US"
    pub language: String,
    /// UI theme mode: "light", "dark", or "system"
    pub theme_mode: String,
    /// Shortcut to postpone breaks, e.g., "Ctrl+Shift+X"
    pub postpone_shortcut: String,
    /// Break window size percentage (0.1 to 1.0, where 1.0 is fullscreen)
    pub window_size: f32,
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
            language: detect_system_language(),
            theme_mode: "system".to_string(),
            postpone_shortcut: String::new(),
            window_size: 0.8, // Default 80% of screen size
            schedules: vec![ScheduleSettings::default()],
            attentions: vec![],
        }
    }
}

/// Detect system language and return appropriate locale code
fn detect_system_language() -> String {
    sys_locale::get_locale()
        .inspect(|loc| println!("Detected system locale: {loc:?}"))
        .unwrap_or_else(|| "en-US".to_string())
}

/// Shared application configuration wrapped in a RwLock for thread-safe access
pub type SharedConfig = RwLock<AppConfig>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_toml_serialization() {
        let config = AppConfig::default();
        let toml_string = toml::to_string_pretty(&config).expect("Failed to serialize config");
        let deserialized: AppConfig =
            toml::from_str(&toml_string).expect("Failed to deserialize config");

        assert_eq!(deserialized.schedules.len(), 1, "Should have 1 schedule");
    }

    #[test]
    fn test_default_config_json_serialization() {
        let config = AppConfig::default();
        let json_string =
            serde_json::to_string_pretty(&config).expect("Failed to serialize config to JSON");
        let deserialized: AppConfig =
            serde_json::from_str(&json_string).expect("Failed to deserialize config from JSON");

        assert_eq!(deserialized.schedules.len(), 1);
        assert!(deserialized.schedules[0].mini_breaks.interval_s > 0);
    }

    #[test]
    fn test_config_camel_case_fields() {
        let config = AppConfig::default();
        let json_string = serde_json::to_string(&config).expect("Failed to serialize");

        // Verify camelCase naming
        assert!(json_string.contains("\"checkForUpdates\""));
        assert!(json_string.contains("\"monitorDnd\""));
        assert!(json_string.contains("\"inactiveS\""));
        assert!(json_string.contains("\"allScreens\""));
        assert!(json_string.contains("\"themeMode\""));
        assert!(json_string.contains("\"postponeShortcut\""));
    }

    #[test]
    fn test_config_default_values() {
        let config = AppConfig::default();

        assert!(config.check_for_updates);
        assert!(config.monitor_dnd);
        assert_eq!(config.inactive_s, 300);
        assert!(!config.all_screens);
        assert!(!config.language.is_empty());
        assert_eq!(config.postpone_shortcut, "");
        assert_eq!(config.schedules.len(), 1);
        assert_eq!(config.attentions.len(), 0);
    }

    #[test]
    fn test_config_clone() {
        let config = AppConfig::default();
        let cloned = config.clone();

        assert_eq!(config.language, cloned.language);
        assert_eq!(config.schedules.len(), cloned.schedules.len());
    }
}
