use crate::apprules::AppRules;
use crate::platform::{get_foreground_process_name, list_running_process_names, show_distraction_popup};
use crate::logger::log_event;
use crate::db::DbHandle;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct FocusSession {
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub work_apps: Vec<String>,
    pub is_active: bool,
    pub distraction_attempts: u32,
}

pub struct SessionManager {
    apprules: AppRules,
    pub current_session: Option<FocusSession>,
    last_distraction_app: Option<String>,
    pub last_checked_process: Option<String>,
    pub last_blocked: bool,
    db_handle: DbHandle,
    session_id: Option<i64>,
    last_app: Option<String>,
    last_app_start: Option<std::time::SystemTime>,
}

impl SessionManager {
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

    pub fn poll(&mut self) {
        let running_processes = list_running_process_names();
        let any_work_app_running = running_processes.iter().any(|name| self.apprules.is_work_app(name));

        if let Some(proc) = get_foreground_process_name() {
            let is_work = self.apprules.is_work_app(&proc);
            let is_blocked = self.apprules.is_blocked(&proc);

            // Track app focus duration
            let now = std::time::SystemTime::now();
            if let Some(last_app) = &self.last_app {
                if last_app != &proc {
                    if let Some(start_time) = self.last_app_start {
                        let duration = now.duration_since(start_time).unwrap_or_default().as_secs() as i64;
                        log_event(
                            Some(&self.db_handle),
                            last_app,
                            false, // is_blocked for previous app not tracked
                            None,
                            self.session_id,
                            Some(start_time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64),
                            Some(now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64),
                            Some(duration),
                        );
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
                Some(now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64),
                None,
                None,
            );
            self.last_checked_process = Some(proc.clone());
            self.last_blocked = is_blocked;

            if is_blocked {
                println!("    Blocked app in focus: {}", proc);
                if let Some(session) = self.current_session.as_mut() {
                    session.distraction_attempts += 1;
                }
                if self.current_session.is_some() {
                    if self.last_distraction_app.as_deref() != Some(&proc) {
                        show_distraction_popup(&proc);
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
                let session_id = self.db_handle.insert_session(now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64).ok();
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
                    let end_time = session.end_time.unwrap().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
                    let work_apps_str = session.work_apps.join(",");
                    let distraction_attempts = session.distraction_attempts as i32;
                    let _ = self.db_handle.update_session(session_id, end_time, &work_apps_str, distraction_attempts);
                }
                self.current_session = None;
                self.session_id = None;
            }
        }
    }

    pub fn end_active_session(&mut self) {
        if let Some(session) = self.current_session.as_mut() {
            session.is_active = false;
            session.end_time = Some(SystemTime::now());
            println!("\n--- Focus session ended (graceful shutdown) ---");
            println!("Apps used: {:?}", session.work_apps);
            if let Some(session_id) = self.session_id {
                let end_time = session.end_time.unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
                let work_apps_str = session.work_apps.join(",");
                let distraction_attempts = session.distraction_attempts as i32;
                let _ = self.db_handle.update_session(session_id, end_time, &work_apps_str, distraction_attempts);
            }
            self.current_session = None;
            self.session_id = None;
        }
    }
}
