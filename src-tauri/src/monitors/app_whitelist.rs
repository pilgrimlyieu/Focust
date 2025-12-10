/// Monitor for application exclusion/whitelist
///
/// Checks if specified applications are running and triggers pause/resume
/// actions based on configured exclusion rules.
use std::future::Future;
use std::pin::Pin;

use sysinfo::{ProcessRefreshKind, System};

use super::{Monitor, MonitorAction, MonitorResult};
use crate::config::{AppExclusion, ExclusionRule};
use crate::scheduler::models::PauseReason;

const INTERVAL_SECS: u64 = 10;

/// Monitor that checks for running applications and applies exclusion rules
pub struct AppWhitelistMonitor {
    /// Configured exclusion rules
    exclusions: Vec<AppExclusion>,
    /// System information for process checking
    system: System,
    /// Whether we are currently paused due to app exclusion
    is_paused: bool,
}

impl AppWhitelistMonitor {
    /// Create a new app whitelist monitor with the given exclusion rules
    #[must_use]
    pub fn new(mut exclusions: Vec<AppExclusion>) -> Self {
        // Create system instance for process monitoring
        let system = System::new();

        // Rebuild indices for all exclusions (in case they were deserialized)
        for exclusion in &mut exclusions {
            exclusion.rebuild_index();
        }

        Self {
            exclusions,
            system,
            is_paused: false,
        }
    }

    /// Update the exclusion rules
    pub fn update_exclusions(&mut self, mut exclusions: Vec<AppExclusion>) {
        // Rebuild indices for all exclusions
        for exclusion in &mut exclusions {
            exclusion.rebuild_index();
        }
        self.exclusions = exclusions;
    }

    /// Check if any processes match the current exclusion rules
    fn check_processes(&mut self) -> bool {
        // Refresh system processes
        self.system.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::everything(),
        );

        // Find the first active exclusion rule
        let active_exclusion = self.exclusions.iter().find(|e| e.active);

        let Some(exclusion) = active_exclusion else {
            // No active exclusions, don't pause
            return false;
        };

        // Pre-extract and cache process information (lowercase for matching)
        let cached_processes: Vec<(String, Option<String>)> = self
            .system
            .processes()
            .values()
            .map(|process| {
                let name = process.name().to_string_lossy().to_lowercase();
                let path = process.exe().map(|p| p.to_string_lossy().to_lowercase());
                (name, path)
            })
            .collect();

        // Check if any cached process matches the exclusion patterns
        let has_matching_process = cached_processes.iter().any(|(name, path)| {
            // Check against process name using lowercase variant
            if exclusion.matches_lowercase(name) {
                tracing::debug!("Process '{name}' matched exclusion rule");
                return true;
            }

            // Check against full executable path if available
            if let Some(path) = path
                && exclusion.matches_lowercase(path)
            {
                tracing::debug!("Process path '{path}' matched exclusion rule");
                return true;
            }

            false
        });

        // Apply the rule logic
        match exclusion.rule {
            ExclusionRule::Pause => {
                // Pause when matching processes are running
                has_matching_process
            }
            ExclusionRule::Resume => {
                // Pause when matching processes are NOT running
                !has_matching_process
            }
        }
    }
}

impl Monitor for AppWhitelistMonitor {
    fn name(&self) -> &'static str {
        "AppWhitelistMonitor"
    }

    fn interval(&self) -> u64 {
        INTERVAL_SECS
    }

    fn check(&mut self) -> Pin<Box<dyn Future<Output = MonitorResult> + Send + '_>> {
        Box::pin(async move {
            if self.exclusions.is_empty() {
                return Ok(MonitorAction::None);
            }

            let should_pause = self.check_processes();

            if should_pause && !self.is_paused {
                // Need to pause
                tracing::info!("Application exclusion rule triggered, pausing scheduler");
                self.is_paused = true;
                Ok(MonitorAction::Pause(PauseReason::AppExclusion))
            } else if !should_pause && self.is_paused {
                // Need to resume
                tracing::info!("Application exclusion rule cleared, resuming scheduler");
                self.is_paused = false;
                Ok(MonitorAction::Resume(PauseReason::AppExclusion))
            } else {
                Ok(MonitorAction::None)
            }
        })
    }

    fn on_start(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {
            tracing::debug!(
                "AppWhitelistMonitor started with {} exclusion rule(s)",
                self.exclusions.len()
            );
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppExclusion;

    #[test]
    fn app_whitelist_monitor_creation() {
        let exclusions = vec![AppExclusion::pause(vec!["chrome.exe".to_owned()])];
        let monitor = AppWhitelistMonitor::new(exclusions);

        assert_eq!(monitor.name(), "AppWhitelistMonitor");
        assert_eq!(monitor.interval(), INTERVAL_SECS);
        assert!(!monitor.is_paused);
        assert_eq!(monitor.exclusions.len(), 1);
    }

    #[test]
    fn app_whitelist_monitor_empty_exclusions() {
        let monitor = AppWhitelistMonitor::new(vec![]);
        assert_eq!(monitor.exclusions.len(), 0);
    }

    #[test]
    fn app_whitelist_monitor_update_exclusions() {
        let mut monitor = AppWhitelistMonitor::new(vec![]);
        assert_eq!(monitor.exclusions.len(), 0);

        let new_exclusions = vec![
            AppExclusion::pause(vec!["chrome.exe".to_owned()]),
            AppExclusion::pause(vec!["firefox.exe".to_owned()]),
        ];

        monitor.update_exclusions(new_exclusions);
        assert_eq!(monitor.exclusions.len(), 2);
    }

    #[test]
    fn check_processes_no_exclusions() {
        let mut monitor = AppWhitelistMonitor::new(vec![]);
        assert!(!monitor.check_processes());
    }

    #[test]
    fn check_processes_inactive_exclusion() {
        let mut exclusion = AppExclusion::pause(vec!["nonexistent.exe".to_owned()]);
        exclusion.active = false;

        let mut monitor = AppWhitelistMonitor::new(vec![exclusion]);
        assert!(!monitor.check_processes());
    }

    #[tokio::test]
    async fn check_returns_none_for_empty_exclusions() {
        let mut monitor = AppWhitelistMonitor::new(vec![]);
        let result = monitor.check().await;
        assert!(matches!(result, Ok(MonitorAction::None)));
    }
}
