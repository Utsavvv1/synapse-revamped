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
    pub start_time: SystemTime,
    /// Session end time (if ended).
    pub end_time: Option<SystemTime>,
    /// List of work apps used during the session.
    pub work_apps: Vec<String>,
    /// Whether the session is currently active.
    pub is_active: bool,
    /// Number of distraction attempts during the session.
    pub distraction_attempts: u32,
}

/// Manages the current focus session, tracks app usage, and interacts with the database.
pub struct SessionManager {
    /// Application rules for whitelisting/blacklisting.
    pub apprules: AppRules,
    /// The current focus session, if any.
    pub current_session: Option<FocusSession>,
    /// The last distraction app detected.
    pub last_distraction_app: Option<String>,
    /// The last checked process name.
    pub last_checked_process: Option<String>,
    /// Whether the last checked process was blocked.
    pub last_blocked: bool,
    /// Database handle for session/event logging.
    pub db_handle: DbHandle,
    /// The current session's database ID, if any.
    pub session_id: Option<i64>,
    /// The last app in focus.
    pub last_app: Option<String>,
    /// The start time of the last app in focus.
    pub last_app_start: Option<std::time::SystemTime>,
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
            .map_err(|e| SynapseError::Platform(format!("Failed to get foreground process: {}", e)))? {
            let is_work = self.apprules.is_work_app(&proc);
            let is_blocked = self.apprules.is_blocked(&proc);

            // Track app focus duration
            let now = std::time::SystemTime::now();
            if let Some(last_app) = &self.last_app {
                if last_app != &proc {
                    if let Some(start_time) = self.last_app_start {
                        let duration = now.duration_since(start_time)?.as_secs() as i64;
                        log_event(
                            Some(&self.db_handle),
                            last_app,
                            false, // is_blocked for previous app not tracked
                            None,
                            self.session_id,
                            Some(start_time.duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64),
                            Some(now.duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64),
                            Some(duration),
                        )?;
                    }
                    self.last_app = Some(proc.clone());
                    self.last_app_start = Some(now);
                }
            } else {
                self.last_app = Some(proc.clone());
                self.last_app_start = Some(now);
            }

            log_event(
                Some(&self.db_handle),
                &proc,
                is_blocked,
                Some(is_blocked),
                self.session_id,
                Some(now.duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64),
                None,
                None,
            )?;
            self.last_checked_process = Some(proc.clone());
            self.last_blocked = is_blocked;

            if is_blocked {
                println!("    Blocked app in focus: {}", proc);
                if let Some(session) = self.current_session.as_mut() {
                    session.distraction_attempts += 1;
                }
                if self.current_session.is_some() {
                    if self.last_distraction_app.as_deref() != Some(&proc) {
                        if let Err(e) = show_distraction_popup(&proc)
                            .map_err(|e| SynapseError::Platform(format!("Failed to show distraction popup: {}", e)))
                        {
                            crate::logger::log_error(&e);
                        }
                        self.last_distraction_app = Some(proc.clone());
                    }
                }
            } else if is_work {
                println!("    Work app in focus: {}", proc);
                self.last_distraction_app = None;
            } else {
                println!("    Neutral app in focus: {}", proc);
                self.last_distraction_app = None;
            }

            // Start a session if not already in one, any work app is running, and foreground app is not blocked
            if any_work_app_running && self.current_session.is_none() && !is_blocked {
                println!("\n--- Focus session started ---");
                let work_apps: Vec<String> = running_processes.iter().filter(|name| self.apprules.is_work_app(name)).cloned().collect();
                let session = FocusSession {
                    start_time: now,
                    end_time: None,
                    work_apps: work_apps.clone(),
                    is_active: true,
                    distraction_attempts: 0,
                };
                let session_id = self.db_handle.insert_session(now.duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64).ok();
                self.session_id = session_id;
                self.current_session = Some(session);
            }
            // If already in a session, update work_apps if new work app appears
            if let Some(session) = self.current_session.as_mut() {
                for name in running_processes.iter().filter(|name| self.apprules.is_work_app(name)) {
                    if !session.work_apps.contains(name) {
                        session.work_apps.push(name.clone());
                    }
                }
            }
        } else {
            println!("Could not detect foreground app.");
            self.last_checked_process = None;
            self.last_blocked = false;
            self.last_app = None;
            self.last_app_start = None;
            self.last_distraction_app = None;
        }

        // End session if no whitelisted app is running
        if let Some(session) = self.current_session.as_mut() {
            if !any_work_app_running {
                session.is_active = false;
                session.end_time = Some(std::time::SystemTime::now());
                println!("\n--- Focus session ended ---");
                println!("Apps used: {:?}", session.work_apps);
                if let Some(session_id) = self.session_id {
                    let end_time = session.end_time.unwrap().duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;
                    let work_apps_str = session.work_apps.join(",");
                    let distraction_attempts = session.distraction_attempts as i32;
                    let _ = self.db_handle.update_session(session_id, end_time, &work_apps_str, distraction_attempts)
                        .map_err(|e| SynapseError::Db(rusqlite::Error::ToSqlConversionFailure(Box::new(e))));
                }
                self.current_session = None;
                self.session_id = None;
            }
        }
        Ok(())
    }

    /// Ends the current active session, if any, and updates the database.
    ///
    /// # Errors
    /// Returns `SynapseError` if updating the session fails.
    pub fn end_active_session(&mut self) -> Result<(), SynapseError> {
        if let Some(session) = self.current_session.as_mut() {
            session.is_active = false;
            session.end_time = Some(SystemTime::now());
            println!("\n--- Focus session ended (graceful shutdown) ---");
            println!("Apps used: {:?}", session.work_apps);
            if let Some(session_id) = self.session_id {
                let end_time = session.end_time.unwrap().duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64;
                let work_apps_str = session.work_apps.join(",");
                let distraction_attempts = session.distraction_attempts as i32;
                let _ = self.db_handle.update_session(session_id, end_time, &work_apps_str, distraction_attempts)
                    .map_err(|e| SynapseError::Db(rusqlite::Error::ToSqlConversionFailure(Box::new(e))));
            }
            self.current_session = None;
            self.session_id = None;
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
