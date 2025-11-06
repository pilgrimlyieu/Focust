/// Monitor for application exclusion/whitelist
///
/// Checks if specified applications are running and triggers pause/resume
/// actions based on configured exclusion rules.
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use sysinfo::{ProcessRefreshKind, System};

use super::{Monitor, MonitorAction, MonitorResult};
use crate::config::AppExclusion;
use crate::scheduler::models::PauseReason;

const CHECK_INTERVAL: Duration = Duration::from_secs(10);

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
    pub fn new(exclusions: Vec<AppExclusion>) -> Self {
        // Create system instance for process monitoring
        let system = System::new();

        Self {
            exclusions,
            system,
            is_paused: false,
        }
    }

    /// Update the exclusion rules
    pub fn update_exclusions(&mut self, exclusions: Vec<AppExclusion>) {
        self.exclusions = exclusions;
    }

    /// Check if any processes match the current exclusion rules
    fn check_processes(&mut self) -> bool {
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

        // Check if any running process matches the exclusion patterns
        let has_matching_process = self.system.processes().values().any(|process| {
            let process_name = process.name().to_string_lossy();
            let exe_path = process.exe().map(|p| p.to_string_lossy().to_string());

            // Check against process name
            if exclusion.matches(&process_name) {
                tracing::debug!(
                    "Process '{process_name}' matched exclusion rule",
                );
                return true;
            }

            // Check against full executable path if available
            if let Some(path) = exe_path
                && exclusion.matches(&path)
            {
                tracing::debug!(
                    "Process path '{path}' matched exclusion rule",
                );
                return true;
            }

            false
        });

        // Apply the rule logic
        match exclusion.rule {
            crate::config::ExclusionRule::Pause => {
                // Pause when matching processes are running
                has_matching_process
            }
            crate::config::ExclusionRule::Resume => {
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

    fn interval(&self) -> Duration {
        CHECK_INTERVAL
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
    fn test_app_whitelist_monitor_creation() {
        let exclusions = vec![AppExclusion::pause(vec!["chrome.exe".to_string()])];
        let monitor = AppWhitelistMonitor::new(exclusions);

        assert_eq!(monitor.name(), "AppWhitelistMonitor");
        assert_eq!(monitor.interval(), CHECK_INTERVAL);
        assert!(!monitor.is_paused);
        assert_eq!(monitor.exclusions.len(), 1);
    }

    #[test]
    fn test_app_whitelist_monitor_empty_exclusions() {
        let monitor = AppWhitelistMonitor::new(vec![]);
        assert_eq!(monitor.exclusions.len(), 0);
    }

    #[test]
    fn test_app_whitelist_monitor_update_exclusions() {
        let mut monitor = AppWhitelistMonitor::new(vec![]);
        assert_eq!(monitor.exclusions.len(), 0);

        let new_exclusions = vec![
            AppExclusion::pause(vec!["chrome.exe".to_string()]),
            AppExclusion::pause(vec!["firefox.exe".to_string()]),
        ];

        monitor.update_exclusions(new_exclusions);
        assert_eq!(monitor.exclusions.len(), 2);
    }

    #[test]
    fn test_check_processes_no_exclusions() {
        let mut monitor = AppWhitelistMonitor::new(vec![]);
        assert!(!monitor.check_processes());
    }

    #[test]
    fn test_check_processes_inactive_exclusion() {
        let mut exclusion = AppExclusion::pause(vec!["nonexistent.exe".to_string()]);
        exclusion.active = false;

        let mut monitor = AppWhitelistMonitor::new(vec![exclusion]);
        assert!(!monitor.check_processes());
    }

    #[tokio::test]
    async fn test_check_returns_none_for_empty_exclusions() {
        let mut monitor = AppWhitelistMonitor::new(vec![]);
        let result = monitor.check().await;
        assert!(matches!(result, Ok(MonitorAction::None)));
    }
}
