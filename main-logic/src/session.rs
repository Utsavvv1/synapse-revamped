//! Session module: manages focus sessions, tracks app usage, and handles session state transitions.

use crate::apprules::AppRules;
use crate::db::DbHandle;
use crate::error::SynapseError;
use crate::logger::log_event;
use crate::platform::{
    get_foreground_process_name, list_running_process_names, show_distraction_popup,
};
use crate::sync::SupabaseSync;
use crate::types::AppUsageEvent;
use crate::types::SessionId;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

/// Represents a single focus session, including timing, apps used, and distraction attempts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusSession {
    pub id: Uuid, // Add this line
    /// Session start time.
    #[serde(with = "crate::session::serde_system_time")]
    pub start_time: SystemTime,
    /// Session end time (if ended).
    #[serde(with = "crate::session::serde_option_system_time")]
    pub end_time: Option<SystemTime>,
    /// List of work apps used during the session.
    pub work_apps: Vec<String>,
    /// Number of distraction attempts during the session.
    pub distraction_attempts: u32,
}

impl FocusSession {
    /// Creates a new `FocusSession`.
    pub fn new(start_time: SystemTime, work_apps: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4(), // Generate a new uuid
            start_time,
            end_time: None,
            work_apps,
            distraction_attempts: 0,
        }
    }
    /// Returns the session start time.
    pub fn start_time(&self) -> &SystemTime {
        &self.start_time
    }
    /// Returns the session end time, if set.
    pub fn end_time(&self) -> Option<&SystemTime> {
        self.end_time.as_ref()
    }
    /// Returns a reference to the list of work apps.
    pub fn work_apps(&self) -> &Vec<String> {
        &self.work_apps
    }
    /// Returns the number of distraction attempts.
    pub fn distraction_attempts(&self) -> u32 {
        self.distraction_attempts
    }

    /// Increments the distraction attempts counter.
    pub fn increment_distraction_attempts(&mut self) {
        self.distraction_attempts += 1;
    }
}

use std::collections::HashMap;

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
    session_id: Option<SessionId>,
    /// The last app in focus.
    last_app: Option<String>,
    /// The last app start time.
    last_app_start: Option<std::time::SystemTime>,
    /// The row ID of the last app usage event in the database.
    last_app_event_id: Option<Uuid>,
    supabase_sync: Option<SupabaseSync>,
    on_distraction: Option<Box<dyn Fn(&str) + Send + Sync>>,
    /// Temporary allowances for blocked apps (App Name -> Allowed Until).
    temporary_allowances: HashMap<String, SystemTime>,
}

impl SessionManager {
    /// Creates a new session manager with the given rules and database handle.
    pub fn new(
        apprules: AppRules,
        db_handle: DbHandle,
        supabase_sync: Option<SupabaseSync>,
        on_distraction: Option<Box<dyn Fn(&str) + Send + Sync>>,
    ) -> Self {
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
            last_app_event_id: None,
            supabase_sync,
            on_distraction,
            temporary_allowances: HashMap::new(),
        }
    }

    /// Polls the current foreground app, updates session state, logs events, and handles distractions.
    ///
    /// # Errors
    /// Returns `SynapseError` if any platform or logging operation fails.
    pub fn poll(&mut self) -> Result<Option<FocusSession>, SynapseError> {
        let running_processes = list_running_process_names().map_err(|e| {
            SynapseError::Platform(format!("Failed to list running processes: {}", e))
        })?;
        let any_work_app_running = running_processes
            .iter()
            .any(|name| self.apprules.is_work_app(name));

        // NEW: Start session if any work app is running and no session is active
        if any_work_app_running && self.current_session.is_none() {
            self.start_new_session_if_needed(&running_processes)?;
        }

        if let Some(proc) = get_foreground_process_name().map_err(|e| {
            SynapseError::Platform(format!("Failed to get foreground process: {}", e))
        })? {
            self.handle_foreground_process(proc, &running_processes, any_work_app_running)?;
        } else {
            self.handle_no_foreground_process();
        }

        self.check_and_end_session(any_work_app_running)
    }

    /// Ends the current active session, if any, and updates the database.
    ///
    /// # Errors
    /// Returns `SynapseError` if updating the session fails.
    pub fn end_active_session(&mut self) -> Result<Option<FocusSession>, SynapseError> {
        println!(
            "[SessionManager] end_active_session: supabase_sync is_some: {}",
            self.supabase_sync.is_some()
        );
        // Finalize last app usage event if any
        self.finalize_last_app_usage_event()?;
        if let Some(mut session) = self.current_session.take() {
            println!("\n--- Focus session ended (graceful shutdown) ---");
            println!("Apps used: {:?}", session.work_apps());
            let now = SystemTime::now();
            session.end_time = Some(now);
            if let Some(session_id) = self.session_id.take() {
                let end_time = now.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64;
                let work_apps_str = session.work_apps.join(",");
                let distraction_attempts = session.distraction_attempts as i32;
                self.db_handle.execute_sql(
                    "UPDATE focus_sessions SET end_time = ?1, work_apps = ?2, distraction_attempts = ?3 WHERE id = ?4",
                    &[
                        &end_time.to_string(),
                        &work_apps_str,
                        &distraction_attempts.to_string(),
                        &session_id.0.to_string(),
                    ],
                )?;
            }
            // Supabase: update session at end
            println!(
                "[Supabase][update_focus_session] supabase_sync is_some: {}",
                self.supabase_sync.is_some()
            );
            if let Some(sync) = &self.supabase_sync {
                let session_clone = session.clone();
                let sync = sync.clone();
                println!("[Supabase][update_focus_session] About to update session in Supabase...");
                let handle = std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let _ = rt.block_on(sync.update_focus_session(&session_clone));
                });
                let _ = handle.join(); // Wait for thread to finish so logs are printed
            }
            Ok(Some(session))
        } else {
            Ok(None)
        }
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
    pub fn session_id(&self) -> Option<&SessionId> {
        self.session_id.as_ref()
    }
    /// Returns a mutable reference to the current focus session, if any.
    pub fn current_session_mut(&mut self) -> Option<&mut FocusSession> {
        self.current_session.as_mut()
    }

    /// Test-only: sets the last checked process.
    #[cfg(test)]
    pub fn set_last_checked_process(&mut self, val: String) {
        self.last_checked_process = Some(val);
    }
    /// Test-only: sets the last blocked status.
    #[cfg(test)]
    pub fn set_last_blocked(&mut self, val: bool) {
        self.last_blocked = val;
    }
    /// Sets the current session (for tests and integration).
    pub fn set_current_session(&mut self, session: FocusSession) {
        self.current_session = Some(session);
    }
    /// Sets the session ID (for tests and integration).
    pub fn set_session_id(&mut self, id: SessionId) {
        self.session_id = Some(id);
    }

    /// Sets the application rules for the session manager.
    pub fn set_apprules(&mut self, apprules: crate::apprules::AppRules) {
        self.apprules = apprules;
    }

    // --- Private Helper Methods ---

    /// Snoozes a blocked app for a specified duration.
    pub fn snooze_app(&mut self, app_name: String, duration: std::time::Duration) {
        let allowed_until = SystemTime::now() + duration;
        println!(
            "[SessionManager] Snoozing app '{}' until {:?}",
            app_name, allowed_until
        );
        self.temporary_allowances
            .insert(app_name.to_lowercase(), allowed_until);
    }

    fn handle_foreground_process(
        &mut self,
        proc_name: String,
        running_processes: &[String],
        any_work_app_running: bool,
    ) -> Result<(), SynapseError> {
        let mut is_blocked = self.apprules.is_blocked(&proc_name);
        let is_work_app = self.apprules.is_work_app(&proc_name);

        // check temporary allowances
        if is_blocked {
            if let Some(allowed_until) = self.temporary_allowances.get(&proc_name.to_lowercase()) {
                if SystemTime::now() < *allowed_until {
                    println!("    App '{}' is temporarily allowed (snoozed)", proc_name);
                    is_blocked = false;
                } else {
                    // Allowance expired
                    self.temporary_allowances.remove(&proc_name.to_lowercase());
                }
            }
        }

        self.update_app_focus_duration(&proc_name)?;
        self.log_app_event(&proc_name, is_blocked)?;
        self.handle_distraction(&proc_name, is_blocked)?;

        if any_work_app_running && is_work_app {
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
        let now_secs = now.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64;
        if let Some(last_app) = self.last_app.take() {
            if last_app != proc_name {
                if let Some(start_time) = self.last_app_start.take() {
                    let start_time_secs =
                        start_time.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64;
                    let end_time = now_secs;
                    let duration = end_time - start_time_secs;
                    // Only record if a focus session is active
                    if let Some(ref session) = self.current_session {
                        let mut is_blocked = self.apprules.is_blocked(&last_app);
                        // Check allowance for historical record too?
                        // If it was allowed when it started, it should probably be recorded as allowed.
                        // But strictly, we record status based on rules.
                        // Ideally, we pass the status determined at detection time.
                        // But `update_app_focus_duration` recalculates `is_blocked`.
                        // Let's check allowance here too for consistency.
                        if is_blocked {
                            if let Some(allowed_until) =
                                self.temporary_allowances.get(&last_app.to_lowercase())
                            {
                                // If allowed *now*, we count it as allowed. Ideally strictly checking ranges,
                                // but this is good enough approximation.
                                if SystemTime::now() < *allowed_until {
                                    is_blocked = false;
                                }
                            }
                        }

                        let status = if is_blocked { "blocked" } else { "allowed" };
                        let session_id = Some(session.id);
                        let event_id = self.db_handle.insert_app_usage_event(
                            &last_app,
                            status,
                            session_id,
                            start_time_secs,
                            end_time,
                            duration,
                        )?;
                        self.last_app_event_id = Some(event_id);
                        // Immediately send to Supabase
                        if let Some(sync) = &self.supabase_sync {
                            let event = crate::types::AppUsageEvent {
                                id: Uuid::new_v4(),
                                process_name: last_app.clone(),
                                status: status.to_string(),
                                session_id,
                                start_time: start_time_secs,
                                end_time,
                                duration_secs: duration,
                            };
                            let sync = sync.clone();
                            tokio::spawn(async move {
                                let _ = sync.push_app_usage_events(&[event]).await;
                            });
                        }
                    }
                }
            } else {
                // Same app, just update tracking fields
                self.last_app = Some(last_app);
                return Ok(());
            }
        }
        // Start tracking the new app in focus
        self.last_app = Some(proc_name.to_string());
        self.last_app_start = Some(now);
        Ok(())
    }

    fn log_app_event(&mut self, proc_name: &str, is_blocked: bool) -> Result<(), SynapseError> {
        let now = SystemTime::now();
        let now_secs = now.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64;
        log_event(
            Some(&self.db_handle),
            proc_name,
            is_blocked,
            Some(is_blocked),
            self.current_session.as_ref().map(|s| s.id),
            Some(now_secs),
            Some(now_secs),
            Some(0),
        )?;
        self.last_checked_process = Some(proc_name.to_string());
        self.last_blocked = is_blocked;
        Ok(())
    }

    fn handle_distraction(
        &mut self,
        proc_name: &str,
        is_blocked: bool,
    ) -> Result<(), SynapseError> {
        if is_blocked {
            // Only count distraction and notify if it's a new distraction event
            // (i.e., different app than last time, or re-opening the same app after switching away)
            if self.last_distraction_app.as_deref() != Some(proc_name) {
                println!("    Blocked app in focus: {}", proc_name);
                if let Some(session) = self.current_session.as_mut() {
                    session.distraction_attempts += 1;
                    // Persist distraction count immediately
                    if let Some(session_id) = self.session_id.clone() {
                        if let Err(e) = self.db_handle.update_session_distractions(
                            session_id.into(),
                            session.distraction_attempts as i32,
                        ) {
                            eprintln!("Failed to update distraction count in DB: {}", e);
                        }
                    }
                }

                if self.current_session.is_some() {
                    if let Some(callback) = &self.on_distraction {
                        callback(proc_name);
                    } else {
                        // Fallback to native popup if no callback provided
                        show_distraction_popup(proc_name).map_err(|e| {
                            SynapseError::Platform(format!(
                                "Failed to show distraction popup: {}",
                                e
                            ))
                        })?;
                    }
                    self.last_distraction_app = Some(proc_name.to_string());
                }
            }
        } else {
            self.last_distraction_app = None;
        }
        Ok(())
    }

    fn start_new_session_if_needed(
        &mut self,
        running_processes: &[String],
    ) -> Result<(), SynapseError> {
        if self.current_session.is_none() {
            println!("\n--- Focus session started ---");
            let work_apps: Vec<String> = running_processes
                .iter()
                .filter(|name| self.apprules.is_work_app(name))
                .cloned()
                .collect();
            let session = FocusSession {
                id: Uuid::new_v4(),
                start_time: SystemTime::now(),
                end_time: None,
                work_apps: work_apps.clone(),
                distraction_attempts: 0,
            };
            self.db_handle.execute_sql(
                "INSERT INTO focus_sessions (id, start_time, end_time, work_apps, distraction_attempts) VALUES (?1, ?2, NULL, ?3, ?4)",
                &[
                    &session.id.to_string(),
                    &session.start_time.duration_since(SystemTime::UNIX_EPOCH)?.as_secs().to_string(),
                    &work_apps.join(","),
                    &session.distraction_attempts.to_string(),
                ],
            )?;
            // Supabase: insert session at start
            if let Some(sync) = &self.supabase_sync {
                let session_clone = session.clone();
                let sync = sync.clone();
                tokio::spawn(async move {
                    let _ = sync.insert_focus_session(&session_clone).await;
                });
            }
            self.session_id = Some(SessionId::from(session.id));
            self.current_session = Some(session);
        }
        Ok(())
    }

    fn update_work_apps_in_current_session(&mut self, running_processes: &[String]) {
        if let Some(session) = self.current_session.as_mut() {
            for name in running_processes
                .iter()
                .filter(|name| self.apprules.is_work_app(name))
            {
                if !session.work_apps.contains(name) {
                    session.work_apps.push(name.clone());
                }
            }
        }
    }

    fn check_and_end_session(
        &mut self,
        any_work_app_running: bool,
    ) -> Result<Option<FocusSession>, SynapseError> {
        if self.current_session.is_some() && !any_work_app_running {
            // Finalize last app usage event if any
            self.finalize_last_app_usage_event()?;
            if let Some(mut session) = self.current_session.take() {
                println!("\n--- Focus session ended ---");
                println!("Apps used: {:?}", session.work_apps());
                let now = SystemTime::now();
                session.end_time = Some(now);
                if let Some(session_id) = self.session_id.take() {
                    let end_time = now.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64;
                    let work_apps_str = session.work_apps().join(",");
                    let distraction_attempts = session.distraction_attempts() as i32;
                    self.db_handle
                        .update_session(
                            session_id.into(),
                            end_time,
                            &work_apps_str,
                            distraction_attempts,
                        )
                        .map_err(|e| {
                            SynapseError::Db(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
                        })?;
                }
                return Ok(Some(session));
            }
        }
        Ok(None)
    }

    /// Finalizes the last app usage event by updating its end_time and duration_secs if needed.
    fn finalize_last_app_usage_event(&mut self) -> Result<(), SynapseError> {
        if let (Some(event_id), Some(start_time), Some(app)) = (
            self.last_app_event_id.take(),
            self.last_app_start.take(),
            self.last_app.take(),
        ) {
            let now = SystemTime::now();
            let end_time = now.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64;
            let start_time_secs =
                start_time.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64;
            let duration = end_time - start_time_secs;
            self.db_handle
                .update_app_usage_event(event_id, end_time, duration)?;
        }
        Ok(())
    }
}

// Serde helpers for SystemTime serialization
pub mod serde_system_time {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time
            .duration_since(UNIX_EPOCH)
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::from_secs(secs))
    }
}

pub mod serde_option_system_time {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &Option<SystemTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match time {
            Some(t) => {
                let duration = t
                    .duration_since(UNIX_EPOCH)
                    .map_err(serde::ser::Error::custom)?;
                serializer.serialize_some(&duration.as_secs())
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<SystemTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<u64>::deserialize(deserializer)?;
        Ok(opt.map(|secs| UNIX_EPOCH + Duration::from_secs(secs)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apprules::AppRules;
    use crate::db::DbHandle;
    use std::time::{Duration, SystemTime};

    fn setup_manager() -> SessionManager {
        let rules = AppRules::test_with_rules(
            vec!["notepad.exe".to_string(), "word.exe".to_string()],
            vec!["chrome.exe".to_string(), "game.exe".to_string()],
        );
        let mut db = DbHandle::test_in_memory();
        db.test_conn()
            .execute(
                "CREATE TABLE IF NOT EXISTS focus_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                work_apps TEXT,
                distraction_attempts INTEGER
            )",
                [],
            )
            .unwrap();
        db.test_conn()
            .execute(
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
            )
            .unwrap();
        SessionManager::new(rules, db, None, None)
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
        assert!(mgr.last_app_event_id.is_none());
    }

    #[test]
    fn test_start_and_end_session() {
        let mut mgr = setup_manager();
        // Simulate starting a session
        let now = SystemTime::now();
        mgr.current_session = Some(FocusSession {
            id: Uuid::new_v4(), // Generate a new uuid for the new session
            start_time: now,
            end_time: None,
            work_apps: vec!["notepad.exe".to_string()],
            distraction_attempts: 0,
        });
        mgr.session_id = Some(SessionId::from(mgr.current_session.as_ref().unwrap().id));
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
            id: Uuid::new_v4(), // Generate a new uuid for the new session
            start_time: now,
            end_time: None,
            work_apps: vec!["notepad.exe".to_string()],
            distraction_attempts: 0,
        });
        if let Some(session) = mgr.current_session.as_mut() {
            session.distraction_attempts += 1;
        }
        assert_eq!(
            mgr.current_session.as_ref().unwrap().distraction_attempts,
            1
        );
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
            id: Uuid::new_v4(), // Generate a new uuid for the new session
            start_time: now,
            end_time: Some(now + Duration::from_secs(3600)),
            work_apps: vec!["notepad.exe".to_string(), "word.exe".to_string()],
            distraction_attempts: 2,
        };
        let session2 = session.clone();
        assert_eq!(session.work_apps, session2.work_apps);
        let debug_str = format!("{:?}", session2);
        assert!(debug_str.contains("notepad.exe"));
    }
}
