use crate::apprules::AppRules;
use crate::platform::{get_foreground_process_name, list_running_process_names, show_distraction_popup};
use crate::logger::{log_event, log_session_event, log_session_json};
use std::time::Duration;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FocusSession {
    pub work_apps: Vec<String>,
    pub is_active: bool,
    // Add other fields as needed
}

pub struct SessionManager {
    apprules: AppRules,
    pub current_session: Option<FocusSession>,
    last_distraction_app: Option<String>,
}

impl SessionManager {
    pub fn new(apprules: AppRules) -> Self {
        Self {
            apprules,
            current_session: None,
            last_distraction_app: None,
        }
    }

    pub fn poll(&mut self) {
        let running_processes = list_running_process_names();
        let any_work_app_running = running_processes.iter().any(|name| self.apprules.is_work_app(name));

        if let Some(proc) = get_foreground_process_name() {
            let is_work = self.apprules.is_work_app(&proc);
            let is_blocked = self.apprules.is_blocked(&proc);
            log_event(&proc, is_blocked);

            if is_blocked {
                println!("    Blocked app in focus: {}", proc);
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

            // Start a session if not already in one and any work app is running
            if any_work_app_running && self.current_session.is_none() {
                println!("\n--- Focus session started ---");
                let work_apps: Vec<String> = running_processes.iter().filter(|name| self.apprules.is_work_app(name)).cloned().collect();
                let session = FocusSession {
                    work_apps,
                    is_active: true,
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
            self.last_distraction_app = None;
        }

        // End session if no whitelisted app is running
        if let Some(session) = self.current_session.as_mut() {
            if !any_work_app_running {
                session.is_active = false;
                println!("\n--- Focus session ended ---");
                println!("Apps used: {:?}", session.work_apps);
                log_session_event(session, false);
                log_session_json(session);
                self.current_session = None;
            }
        }
    }
}
