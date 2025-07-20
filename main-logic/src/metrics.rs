//! Metrics module: tracks and summarizes app usage, blocked events, and session statistics.

use std::collections::HashMap;
use std::time::{Instant};
use crate::session::SessionManager;
use crate::error::SynapseError;

/// Tracks metrics for app usage and focus sessions.
pub struct Metrics {
    /// Total number of app checks performed.
    pub total_checks: u64,
    /// Number of times a blocked app was detected.
    pub blocked_count: u64,
    /// Frequency of each app seen.
    pub app_frequency: HashMap<String, u64>,
    /// Time of the last summary log.
    pub last_summary: Instant,
}

impl Metrics {
    /// Creates a new, empty metrics tracker.
    pub fn new() -> Self {
        Self {
            total_checks: 0,
            blocked_count: 0,
            app_frequency: HashMap::new(),
            last_summary: Instant::now(),
        }
    }

    /// Updates metrics for a single app check.
    ///
    /// # Arguments
    /// * `process` - Name of the process checked
    /// * `is_blocked` - Whether the process was blocked
    pub fn update(&mut self, process: &str, is_blocked: bool) {
        self.total_checks += 1;
        if is_blocked {
            self.blocked_count += 1;
        }
        *self.app_frequency.entry(process.to_string()).or_insert(0) += 1;
    }

    /// Updates metrics from the current session manager state.
    ///
    /// # Arguments
    /// * `session_mgr` - Reference to the session manager
    pub fn update_from_session(&mut self, session_mgr: &SessionManager) {
        if let Some(proc) = &session_mgr.last_checked_process {
            self.update(proc, session_mgr.last_blocked);
        }
        if let Some(session) = &session_mgr.current_session {
            for app in &session.work_apps {
                *self.app_frequency.entry(app.clone()).or_insert(0) += 1;
            }
        }
    }

    /// Returns true if it is time to log a summary (every 60 seconds).
    pub fn should_log_summary(&self) -> bool {
        self.last_summary.elapsed().as_secs() >= 60
    }

    /// Logs a summary of metrics to stdout and resets the timer.
    ///
    /// # Errors
    /// Returns `SynapseError` if logging fails (should not happen for stdout).
    pub fn log_summary(&mut self) -> Result<(), SynapseError> {
        println!("\n----- Focus Summary -----");
        println!("Total Checks: {}", self.total_checks);
        println!("Blocked Detections: {}", self.blocked_count);
        println!("Most Frequent Apps: ");

        let mut entries: Vec<_> = self.app_frequency.iter().collect();
        entries.sort_by_key(|&(_, count)| std::cmp::Reverse(*count));

        for (name, count) in entries.iter().take(5) {
            println!("    {} -> {} times", name, count);
        }

        println!("-------------------------\n");

        // reset timer
        self.last_summary = Instant::now();
        Ok(()) // If future logging is added, wrap errors with context here
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{SessionManager, FocusSession};
    use std::time::{Instant, SystemTime};

    fn setup_metrics() -> Metrics {
        Metrics::new()
    }

    #[test]
    fn test_update_increments_counts() {
        let mut metrics = setup_metrics();
        metrics.update("notepad.exe", false);
        metrics.update("chrome.exe", true);
        assert_eq!(metrics.total_checks, 2);
        assert_eq!(metrics.blocked_count, 1);
        assert_eq!(*metrics.app_frequency.get("notepad.exe").unwrap(), 1);
        assert_eq!(*metrics.app_frequency.get("chrome.exe").unwrap(), 1);
    }

    #[test]
    fn test_update_from_session_adds_apps() {
        let mut metrics = setup_metrics();
        let mut mgr = SessionManager::new(
            crate::apprules::AppRules::test_with_rules(vec!["notepad.exe".to_string()], vec![]),
            crate::db::DbHandle::test_in_memory(),
        );
        mgr.last_checked_process = Some("notepad.exe".to_string());
        mgr.last_blocked = false;
        mgr.current_session = Some(FocusSession {
            start_time: SystemTime::now(),
            end_time: None,
            work_apps: vec!["notepad.exe".to_string(), "word.exe".to_string()],
            is_active: true,
            distraction_attempts: 0,
        });
        metrics.update_from_session(&mgr);
        assert_eq!(metrics.total_checks, 1);
        assert_eq!(*metrics.app_frequency.get("notepad.exe").unwrap(), 2); // once from last_checked_process, once from work_apps
        assert_eq!(*metrics.app_frequency.get("word.exe").unwrap(), 1);
    }

    #[test]
    fn test_should_log_summary_false_initially() {
        let metrics = setup_metrics();
        assert!(!metrics.should_log_summary());
    }

    #[test]
    fn test_should_log_summary_true_after_time() {
        let mut metrics = setup_metrics();
        // Simulate last_summary in the past
        metrics.last_summary = Instant::now() - std::time::Duration::from_secs(61);
        assert!(metrics.should_log_summary());
    }

    #[test]
    fn test_log_summary_prints_and_resets_timer() {
        let mut metrics = setup_metrics();
        metrics.update("notepad.exe", false);
        metrics.update("chrome.exe", true);
        metrics.last_summary = Instant::now() - std::time::Duration::from_secs(61);
        let before = metrics.last_summary;
        metrics.log_summary().unwrap();
        let after = metrics.last_summary;
        assert!(after > before);
    }

    #[test]
    fn test_log_summary_with_no_data() {
        let mut metrics = setup_metrics();
        metrics.last_summary = Instant::now() - std::time::Duration::from_secs(61);
        assert!(metrics.log_summary().is_ok());
    }
}
