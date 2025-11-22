/// Application exclusion configuration
///
/// Allows users to configure when breaks should be paused based on which
/// applications are currently running.
use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Internal index structure for fast process matching
#[derive(Debug, Clone, Default)]
struct ProcessIndex {
    /// Exact match cache for O(1) lookup
    exact_matches: HashSet<String>,
    /// Patterns that need partial matching (`ends_with`/`contains`)
    partial_patterns: Vec<String>,
}

impl ProcessIndex {
    /// Build index from process patterns
    fn build(processes: &[String]) -> Self {
        let mut exact_matches = HashSet::new();
        let mut partial_patterns = Vec::new();

        for pattern in processes {
            let lower = pattern.to_lowercase();
            // Add to exact match set for O(1) lookup
            exact_matches.insert(lower.clone());
            // Keep for partial matching (ends_with/contains)
            partial_patterns.push(lower);
        }

        Self {
            exact_matches,
            partial_patterns,
        }
    }
}

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
#[expect(clippy::partial_pub_fields)]
pub struct AppExclusion {
    /// The rule to apply (pause or resume)
    pub rule: ExclusionRule,
    /// Whether this exclusion is active
    pub active: bool,
    /// List of process names or paths to match
    /// Can be case-insensitive process names (e.g., "chrome.exe", "spotify")
    /// or full paths (e.g., "C:\\Program Files\\App\\app.exe")
    pub processes: Vec<String>,
    /// Internal index for fast matching (rebuilt on load)
    #[serde(skip)]
    #[ts(skip)]
    process_index: ProcessIndex,
}

impl Default for AppExclusion {
    fn default() -> Self {
        Self {
            rule: ExclusionRule::Pause,
            active: false,
            processes: Vec::new(),
            process_index: ProcessIndex::default(),
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
            process_index: ProcessIndex::build(&processes),
            processes,
        }
    }

    /// Create a new resume rule exclusion
    #[must_use]
    pub fn resume(processes: Vec<String>) -> Self {
        Self {
            rule: ExclusionRule::Resume,
            active: true,
            process_index: ProcessIndex::build(&processes),
            processes,
        }
    }

    /// Rebuild internal index (call after deserialization or modification)
    pub fn rebuild_index(&mut self) {
        self.process_index = ProcessIndex::build(&self.processes);
    }

    /// Check if a process name or path matches any of the configured processes
    #[must_use]
    pub fn matches(&self, process_name: &str) -> bool {
        if !self.active || self.processes.is_empty() {
            return false;
        }

        let process_lower = process_name.to_lowercase();
        self.matches_lowercase(&process_lower)
    }

    /// Check if a lowercase process name or path matches any of the configured processes
    /// This method assumes the input is already lowercase, avoiding redundant conversions
    #[must_use]
    pub fn matches_lowercase(&self, process_lower: &str) -> bool {
        if !self.active || self.processes.is_empty() {
            return false;
        }

        // Fast path: O(1) exact match
        if self.process_index.exact_matches.contains(process_lower) {
            return true;
        }

        // Slow path: O(n) pattern matching for ends_with/contains
        self.process_index.partial_patterns.iter().any(|pattern| {
            // Check if process name ends with the pattern (for paths)
            process_lower.ends_with(pattern) ||
            // Check if pattern is contained in process name
            process_lower.contains(pattern)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exclusion_rule_default() {
        assert_eq!(ExclusionRule::default(), ExclusionRule::Pause);
    }

    #[test]
    fn app_exclusion_default() {
        let exclusion = AppExclusion::default();
        assert_eq!(exclusion.rule, ExclusionRule::Pause);
        assert!(!exclusion.active);
        assert!(exclusion.processes.is_empty());
    }

    #[test]
    fn app_exclusion_pause() {
        let exclusion = AppExclusion::pause(vec!["chrome.exe".to_owned()]);
        assert_eq!(exclusion.rule, ExclusionRule::Pause);
        assert!(exclusion.active);
        assert_eq!(exclusion.processes, vec!["chrome.exe"]);
    }

    #[test]
    fn app_exclusion_resume() {
        let exclusion = AppExclusion::resume(vec!["vscode.exe".to_owned()]);
        assert_eq!(exclusion.rule, ExclusionRule::Resume);
        assert!(exclusion.active);
        assert_eq!(exclusion.processes, vec!["vscode.exe"]);
    }

    #[test]
    fn matches_exact() {
        let exclusion = AppExclusion::pause(vec!["chrome.exe".to_owned()]);
        assert!(exclusion.matches("chrome.exe"));
        assert!(exclusion.matches("Chrome.exe")); // Case insensitive
        assert!(exclusion.matches("CHROME.EXE"));
    }

    #[test]
    fn matches_path() {
        let exclusion = AppExclusion::pause(vec!["chrome.exe".to_owned()]);
        assert!(exclusion.matches("C:\\Program Files\\Google\\Chrome\\chrome.exe"));
        assert!(exclusion.matches("/usr/bin/chrome.exe"));
    }

    #[test]
    fn matches_partial() {
        let exclusion = AppExclusion::pause(vec!["chrome".to_owned()]);
        assert!(exclusion.matches("chrome.exe"));
        assert!(exclusion.matches("google-chrome"));
        assert!(exclusion.matches("Chrome Browser.app"));
    }

    #[test]
    fn matches_multiple_patterns() {
        let exclusion = AppExclusion::pause(vec![
            "chrome.exe".to_owned(),
            "firefox.exe".to_owned(),
            "safari".to_owned(),
        ]);

        assert!(exclusion.matches("chrome.exe"));
        assert!(exclusion.matches("firefox.exe"));
        assert!(exclusion.matches("Safari.app"));
        assert!(!exclusion.matches("notepad.exe"));
    }

    #[test]
    fn matches_inactive() {
        let mut exclusion = AppExclusion::pause(vec!["chrome.exe".to_owned()]);
        exclusion.active = false;
        assert!(!exclusion.matches("chrome.exe"));
    }

    #[test]
    fn matches_empty_processes() {
        let exclusion = AppExclusion::pause(vec![]);
        assert!(!exclusion.matches("chrome.exe"));
    }

    #[test]
    fn serialization() {
        let exclusion = AppExclusion::pause(vec!["chrome.exe".to_owned()]);
        let json = serde_json::to_string(&exclusion).unwrap();
        assert!(json.contains("\"rule\":\"pause\""));
        assert!(json.contains("\"active\":true"));
        assert!(json.contains("\"processes\""));
    }

    #[test]
    fn deserialization() {
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
