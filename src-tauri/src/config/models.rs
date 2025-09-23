use serde::{Deserialize, Serialize};
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
