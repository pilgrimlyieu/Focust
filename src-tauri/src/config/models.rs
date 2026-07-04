use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use ts_rs::TS;

use super::app_exclusion::AppExclusion;
use crate::{
    core::schedule::{AttentionSettings, ScheduleSettings},
    platform::i18n::LANGUAGE_FALLBACK,
    scheduler::PauseReason,
    utils::LogLevel,
};

/// Advanced configuration settings
///
/// These settings are for advanced users and debugging.
/// They are stored in a separate `[advanced]` section in the TOML config file.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct AdvancedConfig {
    /// Log level for the application
    /// Overrides the default log level (info in release, trace in debug)
    #[serde(default = "LogLevel::default_for_build")]
    pub log_level: LogLevel,
    /// Pause reasons that reset the mini-break counter used for long-break cadence.
    #[serde(default)]
    pub reset_mini_break_counter_on_pause_reasons: Vec<PauseReason>,
    /// Pause durations shown in the tray menu, in minutes.
    #[serde(default = "default_tray_pause_durations_minutes")]
    pub tray_pause_durations_minutes: Vec<u32>,
}

fn default_tray_pause_durations_minutes() -> Vec<u32> {
    vec![15, 30, 60]
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            log_level: LogLevel::default_for_build(),
            reset_mini_break_counter_on_pause_reasons: Vec::new(),
            tray_pause_durations_minutes: default_tray_pause_durations_minutes(),
        }
    }
}

/// Application configuration structure
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct AppConfig {
    /// If launch on system startup
    pub autostart: bool,
    /// If monitor DND status and pause breaks
    pub monitor_dnd: bool,
    /// Inactive time in seconds before pausing breaks
    pub inactive_s: u32,
    /// If breaks should be shown on all screens
    pub all_screens: bool,
    /// If system tray icon should be displayed (requires restart)
    pub show_tray_icon: bool,
    /// Language code, e.g., "en-US"
    #[ts(type = "import('@/i18n').LocaleKey")]
    pub language: String,
    /// UI theme mode: "light", "dark", or "system"
    pub theme_mode: String,
    /// Shortcut to postpone breaks, e.g., "Ctrl+Shift+X"
    pub postpone_shortcut: String,
    /// Prompt window size percentage (0.1 to 1.0, where 1.0 is fullscreen)
    pub window_size: f32,
    /// List of schedules
    pub schedules: Vec<ScheduleSettings>,
    /// List of attention reminders
    pub attentions: Vec<AttentionSettings>,
    /// Application exclusion rules
    pub app_exclusions: Vec<AppExclusion>,
    /// Advanced configuration (not exported to TypeScript, internal only)
    #[serde(skip)]
    #[ts(skip)]
    pub advanced: AdvancedConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            autostart: false,
            monitor_dnd: true,
            inactive_s: 300,
            all_screens: false,
            show_tray_icon: true,
            language: detect_system_language(),
            theme_mode: "system".to_owned(),
            postpone_shortcut: String::new(),
            window_size: 0.8, // Default 80% of screen size
            schedules: vec![ScheduleSettings::default()],
            attentions: vec![],
            app_exclusions: vec![],
            advanced: AdvancedConfig::default(),
        }
    }
}

/// Detect system language and return appropriate locale code
fn detect_system_language() -> String {
    sys_locale::get_locale().unwrap_or_else(|| LANGUAGE_FALLBACK.to_owned())
}

/// Shared application configuration wrapped in a `RwLock` for thread-safe access
pub type SharedConfig = RwLock<AppConfig>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_toml_serialization() {
        let config = AppConfig::default();
        let toml_string = toml::to_string_pretty(&config).expect("Failed to serialize config");
        let deserialized: AppConfig =
            toml::from_str(&toml_string).expect("Failed to deserialize config");

        assert_eq!(deserialized.schedules.len(), 1, "Should have 1 schedule");
    }

    #[test]
    fn default_config_json_serialization() {
        let config = AppConfig::default();
        let json_string =
            serde_json::to_string_pretty(&config).expect("Failed to serialize config to JSON");
        let deserialized: AppConfig =
            serde_json::from_str(&json_string).expect("Failed to deserialize config from JSON");

        assert_eq!(deserialized.schedules.len(), 1);
        assert!(deserialized.schedules[0].mini_breaks.interval_s > 0);
    }

    #[test]
    fn config_camel_case_fields() {
        let config = AppConfig::default();
        let json_string = serde_json::to_string(&config).expect("Failed to serialize");

        // Verify camelCase naming
        assert!(json_string.contains("\"autostart\""));
        assert!(json_string.contains("\"monitorDnd\""));
        assert!(json_string.contains("\"inactiveS\""));
        assert!(json_string.contains("\"allScreens\""));
        assert!(json_string.contains("\"themeMode\""));
        assert!(json_string.contains("\"postponeShortcut\""));
    }

    #[test]
    fn config_default_values() {
        let config = AppConfig::default();

        assert!(!config.autostart);
        assert!(config.monitor_dnd);
        assert_eq!(config.inactive_s, 300);
        assert!(!config.all_screens);
        assert!(!config.language.is_empty());
        assert_eq!(config.postpone_shortcut, "");
        assert_eq!(config.schedules.len(), 1);
        assert_eq!(config.attentions.len(), 0);
        assert_eq!(
            config.advanced.tray_pause_durations_minutes,
            vec![15, 30, 60]
        );
    }

    #[test]
    fn config_clone() {
        let config = AppConfig::default();
        let cloned = config.clone();

        assert_eq!(config.language, cloned.language);
        assert_eq!(config.schedules.len(), cloned.schedules.len());
    }

    #[test]
    fn advanced_config_default_values() {
        let config = AdvancedConfig::default();

        assert_eq!(config.log_level, LogLevel::default_for_build());
        assert_eq!(config.reset_mini_break_counter_on_pause_reasons, vec![]);
        assert_eq!(config.tray_pause_durations_minutes, vec![15, 30, 60]);
    }

    #[test]
    fn advanced_config_missing_tray_pause_durations_uses_default() {
        let config: AdvancedConfig =
            toml::from_str("logLevel = \"info\"").expect("Failed to deserialize advanced config");

        assert_eq!(config.tray_pause_durations_minutes, vec![15, 30, 60]);
    }

    #[test]
    fn advanced_config_allows_empty_tray_pause_durations() {
        let config: AdvancedConfig = toml::from_str("trayPauseDurationsMinutes = []")
            .expect("Failed to deserialize advanced config");

        assert!(config.tray_pause_durations_minutes.is_empty());
    }
}
