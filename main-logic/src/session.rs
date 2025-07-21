//! Session module: manages focus sessions, tracks app usage, and handles session state transitions.

use crate::apprules::AppRules;
use crate::platform::{get_foreground_process_name, list_running_process_names, show_distraction_popup};
use crate::logger::log_event;
use crate::db::DbHandle;
use crate::error::SynapseError;
use std::time::SystemTime;

/// Represents a single focus session, including timing, apps used, and distraction attempts.
#[derive(Debug, Clone)]
pub struct FocusSession {
    /// Session start time.
    start_time: SystemTime,
    /// Session end time (if ended).
    end_time: Option<SystemTime>,
    /// List of work apps used during the session.
    work_apps: Vec<String>,
    /// Whether the session is currently active.
    is_active: bool,
    /// Number of distraction attempts during the session.
    distraction_attempts: u32,
}

impl FocusSession {
    /// Creates a new `FocusSession`.
    pub fn new(start_time: SystemTime, work_apps: Vec<String>) -> Self {
        Self {
            start_time,
            end_time: None,
            work_apps,
            is_active: true,
            distraction_attempts: 0,
        }
    }
    /// Returns the session start time.
    pub fn start_time(&self) -> &SystemTime { &self.start_time }
    /// Returns the session end time, if set.
    pub fn end_time(&self) -> Option<&SystemTime> { self.end_time.as_ref() }
    /// Returns a reference to the list of work apps.
    pub fn work_apps(&self) -> &Vec<String> { &self.work_apps }
    /// Returns true if the session is active.
    pub fn is_active(&self) -> bool { self.is_active }
    /// Returns the number of distraction attempts.
    pub fn distraction_attempts(&self) -> u32 { self.distraction_attempts }

    /// Increments the distraction attempts counter.
    pub fn increment_distraction_attempts(&mut self) {
        self.distraction_attempts += 1;
    }
}

/// Manages the current focus session, tracks app usage, and interacts with the database.
pub struct SessionManager {
    /// Application rules for whitelisting/blacklisting.
    apprules: AppRules,
    /// The current focus session, if any.
    current_session: Option<FocusSession>,
    /// The last distraction app detected.
    last_distraction_app: Option<String>,
    /// The last checked process name.
    last_checked_process: Option<String>,
    /// Whether the last checked process was blocked.
    last_blocked: bool,
    /// Database handle for session/event logging.
    db_handle: DbHandle,
    /// The current session's database ID, if any.
    session_id: Option<i64>,
    /// The last app in focus.
    last_app: Option<String>,
    /// The start time of the last app in focus.
    last_app_start: Option<std::time::SystemTime>,
}

impl SessionManager {
    /// Creates a new session manager with the given rules and database handle.
    pub fn new(apprules: AppRules, db_handle: DbHandle) -> Self {
        Self {
            apprules,
            current_session: None,
            last_distraction_app: None,
            last_checked_process: None,
            last_blocked: false,
            db_handle,
            session_id: None,
            last_app: None,
            last_app_start: None,
        }
    }

    /// Polls the current foreground app, updates session state, logs events, and handles distractions.
    ///
    /// # Errors
    /// Returns `SynapseError` if any platform or logging operation fails.
    pub fn poll(&mut self) -> Result<(), SynapseError> {
        let running_processes = list_running_process_names()
            .map_err(|e| SynapseError::Platform(format!("Failed to list running processes: {}", e)))?;
        let any_work_app_running = running_processes.iter().any(|name| self.apprules.is_work_app(name));

        if let Some(proc) = get_foreground_process_name()
            .map_err(|e| SynapseError::Platform(format!("Failed to get foreground process: {}", e)))?
        {
            self.handle_foreground_process(proc, &running_processes, any_work_app_running)?;
        } else {
            self.handle_no_foreground_process();
        }

        self.check_and_end_session(any_work_app_running)?;
        Ok(())
    }

    /// Ends the current active session, if any, and updates the database.
    ///
    /// # Errors
    /// Returns `SynapseError` if updating the session fails.
    pub fn end_active_session(&mut self) -> Result<(), SynapseError> {
        if let Some(session) = self.current_session.take() {
            println!("\n--- Focus session ended (graceful shutdown) ---");
            println!("Apps used: {:?}", session.work_apps());
            if let Some(session_id) = self.session_id.take() {
                let end_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64;
                let work_apps_str = session.work_apps().join(",");
                let distraction_attempts = session.distraction_attempts() as i32;
                self.db_handle.update_session(session_id, end_time, &work_apps_str, distraction_attempts)
                    .map_err(|e| SynapseError::Db(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
            }
        }
        Ok(())
    }

    /// Returns the last checked process name, if any.
    pub fn last_checked_process(&self) -> Option<&String> {
        self.last_checked_process.as_ref()
    }
    /// Returns true if the last checked process was blocked.
    pub fn last_blocked(&self) -> bool {
        self.last_blocked
    }
    /// Returns the current focus session, if any.
    pub fn current_session(&self) -> Option<&FocusSession> {
        self.current_session.as_ref()
    }

    /// Returns a reference to the database handle.
    pub fn db_handle(&self) -> &DbHandle {
        &self.db_handle
    }
    /// Returns the current session ID, if any.
    pub fn session_id(&self) -> Option<i64> {
        self.session_id
    }
    /// Returns a mutable reference to the current focus session, if any.
    pub fn current_session_mut(&mut self) -> Option<&mut FocusSession> {
        self.current_session.as_mut()
    }

    /// Test-only: sets the last checked process.
    #[cfg(test)]
    pub fn set_last_checked_process(&mut self, val: String) { self.last_checked_process = Some(val); }
    /// Test-only: sets the last blocked status.
    #[cfg(test)]
    pub fn set_last_blocked(&mut self, val: bool) { self.last_blocked = val; }
    /// Sets the current session (for tests and integration).
    pub fn set_current_session(&mut self, session: FocusSession) { self.current_session = Some(session); }
    /// Sets the session ID (for tests and integration).
    pub fn set_session_id(&mut self, id: i64) {
        self.session_id = Some(id);
    }

    // --- Private Helper Methods ---

    fn handle_foreground_process(&mut self, proc_name: String, running_processes: &[String], any_work_app_running: bool) -> Result<(), SynapseError> {
        let is_blocked = self.apprules.is_blocked(&proc_name);

        self.update_app_focus_duration(&proc_name)?;
        self.log_app_event(&proc_name, is_blocked)?;
        self.handle_distraction(&proc_name, is_blocked)?;

        if any_work_app_running && !is_blocked {
            self.start_new_session_if_needed(running_processes)?;
        }
        
        if self.current_session.is_some() {
            self.update_work_apps_in_current_session(running_processes);
        }

        Ok(())
    }

    fn handle_no_foreground_process(&mut self) {
        println!("Could not detect foreground app.");
        self.last_checked_process = None;
        self.last_blocked = false;
        self.last_app = None;
        self.last_app_start = None;
        self.last_distraction_app = None;
    }

    fn update_app_focus_duration(&mut self, proc_name: &str) -> Result<(), SynapseError> {
        let now = SystemTime::now();
        if let Some(last_app) = self.last_app.take() {
            if last_app != proc_name {
                if let Some(start_time) = self.last_app_start.take() {
                    let duration = now.duration_since(start_time)?.as_secs() as i64;
                    log_event(
                        Some(&self.db_handle),
                        &last_app,
                        false,
                        None,
                        self.session_id,
                        Some(start_time.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64),
                        Some(now.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64),
                        Some(duration),
                    )?;
                }
            }
        }
        self.last_app = Some(proc_name.to_string());
        self.last_app_start = Some(now);
        Ok(())
    }

    fn log_app_event(&mut self, proc_name: &str, is_blocked: bool) -> Result<(), SynapseError> {
        let now = SystemTime::now();
        log_event(
            Some(&self.db_handle),
            proc_name,
            is_blocked,
            Some(is_blocked),
            self.session_id,
            Some(now.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64),
            None,
            None,
        )?;
        self.last_checked_process = Some(proc_name.to_string());
        self.last_blocked = is_blocked;
        Ok(())
    }

    fn handle_distraction(&mut self, proc_name: &str, is_blocked: bool) -> Result<(), SynapseError> {
        if is_blocked {
            println!("    Blocked app in focus: {}", proc_name);
            if let Some(session) = self.current_session.as_mut() {
                session.distraction_attempts += 1;
            }
            if self.current_session.is_some() && self.last_distraction_app.as_deref() != Some(proc_name) {
                show_distraction_popup(proc_name)
                    .map_err(|e| SynapseError::Platform(format!("Failed to show distraction popup: {}", e)))?;
                self.last_distraction_app = Some(proc_name.to_string());
            }
        } else {
            self.last_distraction_app = None;
        }
        Ok(())
    }

    fn start_new_session_if_needed(&mut self, running_processes: &[String]) -> Result<(), SynapseError> {
        if self.current_session.is_none() {
            println!("\n--- Focus session started ---");
            let work_apps: Vec<String> = running_processes.iter().filter(|name| self.apprules.is_work_app(name)).cloned().collect();
            let session = FocusSession {
                start_time: SystemTime::now(),
                end_time: None,
                work_apps: work_apps.clone(),
                is_active: true,
                distraction_attempts: 0,
            };
            let session_id = self.db_handle.insert_session(session.start_time.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64)?;
            self.session_id = Some(session_id);
            self.current_session = Some(session);
        }
        Ok(())
    }

    fn update_work_apps_in_current_session(&mut self, running_processes: &[String]) {
        if let Some(session) = self.current_session.as_mut() {
            for name in running_processes.iter().filter(|name| self.apprules.is_work_app(name)) {
                if !session.work_apps.contains(name) {
                    session.work_apps.push(name.clone());
                }
            }
        }
    }

    fn check_and_end_session(&mut self, any_work_app_running: bool) -> Result<(), SynapseError> {
        if self.current_session.is_some() && !any_work_app_running {
            if let Some(session) = self.current_session.take() {
                println!("\n--- Focus session ended ---");
                println!("Apps used: {:?}", session.work_apps());
                if let Some(session_id) = self.session_id.take() {
                    let end_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64;
                    let work_apps_str = session.work_apps().join(",");
                    let distraction_attempts = session.distraction_attempts() as i32;
                    self.db_handle.update_session(session_id, end_time, &work_apps_str, distraction_attempts)
                        .map_err(|e| SynapseError::Db(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apprules::AppRules;
    use crate::db::DbHandle;
    use std::time::{SystemTime, Duration};

    fn setup_manager() -> SessionManager {
        let rules = AppRules::test_with_rules(
            vec!["notepad.exe".to_string(), "word.exe".to_string()],
            vec!["chrome.exe".to_string(), "game.exe".to_string()],
        );
        let mut db = DbHandle::test_in_memory();
        db.test_conn().execute(
            "CREATE TABLE IF NOT EXISTS focus_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                work_apps TEXT,
                distraction_attempts INTEGER
            )",
            [],
        ).unwrap();
        db.test_conn().execute(
            "CREATE TABLE IF NOT EXISTS app_usage_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                process_name TEXT NOT NULL,
                is_blocked BOOLEAN NOT NULL,
                distraction BOOLEAN,
                session_id INTEGER,
                start_time INTEGER,
                end_time INTEGER,
                duration_secs INTEGER
            )",
            [],
        ).unwrap();
        SessionManager::new(rules, db)
    }

    #[test]
    fn test_new_manager_initial_state() {
        let mgr = setup_manager();
        assert!(mgr.current_session.is_none());
        assert!(mgr.last_distraction_app.is_none());
        assert!(mgr.last_checked_process.is_none());
        assert!(!mgr.last_blocked);
        assert!(mgr.session_id.is_none());
        assert!(mgr.last_app.is_none());
        assert!(mgr.last_app_start.is_none());
    }

    #[test]
    fn test_start_and_end_session() {
        let mut mgr = setup_manager();
        // Simulate starting a session
        let now = SystemTime::now();
        mgr.current_session = Some(FocusSession {
            start_time: now,
            end_time: None,
            work_apps: vec!["notepad.exe".to_string()],
            is_active: true,
            distraction_attempts: 0,
        });
        mgr.session_id = Some(1);
        assert!(mgr.current_session.is_some());
        // End session
        mgr.end_active_session().unwrap();
        assert!(mgr.current_session.is_none());
        assert!(mgr.session_id.is_none());
    }

    #[test]
    fn test_distraction_attempts_increment() {
        let mut mgr = setup_manager();
        let now = SystemTime::now();
        mgr.current_session = Some(FocusSession {
            start_time: now,
            end_time: None,
            work_apps: vec!["notepad.exe".to_string()],
            is_active: true,
            distraction_attempts: 0,
        });
        if let Some(session) = mgr.current_session.as_mut() {
            session.distraction_attempts += 1;
        }
        assert_eq!(mgr.current_session.as_ref().unwrap().distraction_attempts, 1);
    }

    #[test]
    fn test_end_active_session_no_session() {
        let mut mgr = setup_manager();
        // Should not panic or error if no session is active
        assert!(mgr.end_active_session().is_ok());
    }

    #[test]
    fn test_focus_session_clone_and_debug() {
        let now = SystemTime::now();
        let session = FocusSession {
            start_time: now,
            end_time: Some(now + Duration::from_secs(3600)),
            work_apps: vec!["notepad.exe".to_string(), "word.exe".to_string()],
            is_active: false,
            distraction_attempts: 2,
        };
        let session2 = session.clone();
        assert_eq!(session.work_apps, session2.work_apps);
        let debug_str = format!("{:?}", session2);
        assert!(debug_str.contains("notepad.exe"));
    }
}
