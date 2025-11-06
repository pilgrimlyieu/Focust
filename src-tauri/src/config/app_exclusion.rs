/// Application exclusion configuration
///
/// Allows users to configure when breaks should be paused based on which
/// applications are currently running.
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Rule for application exclusion behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, Default)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub enum ExclusionRule {
    /// Pause breaks when any of the specified processes are running
    #[default]
    Pause,
    /// Pause breaks when NONE of the specified processes are running
    /// (i.e., only allow breaks when at least one specified process is running)
    Resume,
}

/// Application exclusion configuration
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct AppExclusion {
    /// The rule to apply (pause or resume)
    pub rule: ExclusionRule,
    /// Whether this exclusion is active
    pub active: bool,
    /// List of process names or paths to match
    /// Can be case-insensitive process names (e.g., "chrome.exe", "spotify")
    /// or full paths (e.g., "C:\\Program Files\\App\\app.exe")
    pub processes: Vec<String>,
}

impl Default for AppExclusion {
    fn default() -> Self {
        Self {
            rule: ExclusionRule::Pause,
            active: false,
            processes: Vec::new(),
        }
    }
}

impl AppExclusion {
    /// Create a new pause rule exclusion
    #[must_use]
    pub fn pause(processes: Vec<String>) -> Self {
        Self {
            rule: ExclusionRule::Pause,
            active: true,
            processes,
        }
    }

    /// Create a new resume rule exclusion
    #[must_use]
    pub fn resume(processes: Vec<String>) -> Self {
        Self {
            rule: ExclusionRule::Resume,
            active: true,
            processes,
        }
    }

    /// Check if a process name or path matches any of the configured processes
    #[must_use]
    pub fn matches(&self, process_name: &str) -> bool {
        if !self.active || self.processes.is_empty() {
            return false;
        }

        let process_lower = process_name.to_lowercase();

        self.processes.iter().any(|pattern| {
            let pattern_lower = pattern.to_lowercase();

            // Check for exact match
            if process_lower == pattern_lower {
                return true;
            }

            // Check if process name ends with the pattern (for paths)
            if process_lower.ends_with(&pattern_lower) {
                return true;
            }

            // Check if pattern is contained in process name
            process_lower.contains(&pattern_lower)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exclusion_rule_default() {
        assert_eq!(ExclusionRule::default(), ExclusionRule::Pause);
    }

    #[test]
    fn test_app_exclusion_default() {
        let exclusion = AppExclusion::default();
        assert_eq!(exclusion.rule, ExclusionRule::Pause);
        assert!(!exclusion.active);
        assert!(exclusion.processes.is_empty());
    }

    #[test]
    fn test_app_exclusion_pause() {
        let exclusion = AppExclusion::pause(vec!["chrome.exe".to_string()]);
        assert_eq!(exclusion.rule, ExclusionRule::Pause);
        assert!(exclusion.active);
        assert_eq!(exclusion.processes, vec!["chrome.exe"]);
    }

    #[test]
    fn test_app_exclusion_resume() {
        let exclusion = AppExclusion::resume(vec!["vscode.exe".to_string()]);
        assert_eq!(exclusion.rule, ExclusionRule::Resume);
        assert!(exclusion.active);
        assert_eq!(exclusion.processes, vec!["vscode.exe"]);
    }

    #[test]
    fn test_matches_exact() {
        let exclusion = AppExclusion::pause(vec!["chrome.exe".to_string()]);
        assert!(exclusion.matches("chrome.exe"));
        assert!(exclusion.matches("Chrome.exe")); // Case insensitive
        assert!(exclusion.matches("CHROME.EXE"));
    }

    #[test]
    fn test_matches_path() {
        let exclusion = AppExclusion::pause(vec!["chrome.exe".to_string()]);
        assert!(exclusion.matches("C:\\Program Files\\Google\\Chrome\\chrome.exe"));
        assert!(exclusion.matches("/usr/bin/chrome.exe"));
    }

    #[test]
    fn test_matches_partial() {
        let exclusion = AppExclusion::pause(vec!["chrome".to_string()]);
        assert!(exclusion.matches("chrome.exe"));
        assert!(exclusion.matches("google-chrome"));
        assert!(exclusion.matches("Chrome Browser.app"));
    }

    #[test]
    fn test_matches_multiple_patterns() {
        let exclusion = AppExclusion::pause(vec![
            "chrome.exe".to_string(),
            "firefox.exe".to_string(),
            "safari".to_string(),
        ]);

        assert!(exclusion.matches("chrome.exe"));
        assert!(exclusion.matches("firefox.exe"));
        assert!(exclusion.matches("Safari.app"));
        assert!(!exclusion.matches("notepad.exe"));
    }

    #[test]
    fn test_matches_inactive() {
        let mut exclusion = AppExclusion::pause(vec!["chrome.exe".to_string()]);
        exclusion.active = false;
        assert!(!exclusion.matches("chrome.exe"));
    }

    #[test]
    fn test_matches_empty_processes() {
        let exclusion = AppExclusion::pause(vec![]);
        assert!(!exclusion.matches("chrome.exe"));
    }

    #[test]
    fn test_serialization() {
        let exclusion = AppExclusion::pause(vec!["chrome.exe".to_string()]);
        let json = serde_json::to_string(&exclusion).unwrap();
        assert!(json.contains("\"rule\":\"pause\""));
        assert!(json.contains("\"active\":true"));
        assert!(json.contains("\"processes\""));
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{
            "rule": "pause",
            "active": true,
            "processes": ["chrome.exe", "firefox.exe"]
        }"#;

        let exclusion: AppExclusion = serde_json::from_str(json).unwrap();
        assert_eq!(exclusion.rule, ExclusionRule::Pause);
        assert!(exclusion.active);
        assert_eq!(exclusion.processes.len(), 2);
    }
}
