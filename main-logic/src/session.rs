use crate::apprules::AppRules;
use crate::platform::{get_foreground_process_name, list_running_process_names, show_distraction_popup};
use crate::logger::{log_event, log_session_event, log_session_json};
use std::time::{Duration, SystemTime};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
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
}

impl SessionManager {
    pub fn new(apprules: AppRules) -> Self {
        Self {
            apprules,
            current_session: None,
            last_distraction_app: None,
            last_checked_process: None,
            last_blocked: false,
        }
    }

    pub fn poll(&mut self) {
        let running_processes = list_running_process_names();
        let any_work_app_running = running_processes.iter().any(|name| self.apprules.is_work_app(name));

        if let Some(proc) = get_foreground_process_name() {
            let is_work = self.apprules.is_work_app(&proc);
            let is_blocked = self.apprules.is_blocked(&proc);
            log_event(&proc, is_blocked);
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
                    start_time: SystemTime::now(),
                    end_time: None,
                    work_apps,
                    is_active: true,
                    distraction_attempts: 0,
                };
                log_session_event(&session, true);
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
            self.last_distraction_app = None;
        }

        // End session if no whitelisted app is running
        if let Some(session) = self.current_session.as_mut() {
            if !any_work_app_running {
                session.is_active = false;
                session.end_time = Some(SystemTime::now());
                println!("\n--- Focus session ended ---");
                println!("Apps used: {:?}", session.work_apps);
                log_session_event(session, false);
                log_session_json(session);
                self.current_session = None;
            }
        }
    }
}
