use std::collections::HashMap;
use std::time::{Instant};
use crate::session::SessionManager;

pub struct Metrics {
    pub total_checks: u64,
    pub blocked_count: u64,
    pub app_frequency: HashMap<String, u64>,
    last_summary: Instant,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            total_checks: 0,
            blocked_count: 0,
            app_frequency: HashMap::new(),
            last_summary: Instant::now(),
        }
    }

    pub fn update(&mut self, process: &str, is_blocked: bool) {
        self.total_checks += 1;
        if is_blocked {
            self.blocked_count += 1;
        }
        *self.app_frequency.entry(process.to_string()).or_insert(0) += 1;
    }

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

    pub fn should_log_summary(&self) -> bool {
        self.last_summary.elapsed().as_secs() >= 60
    }

    pub fn log_summary(&mut self) {
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
    }
}
