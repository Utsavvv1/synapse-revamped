mod logger;
mod apprules;
mod platform;
mod metrics;
mod session;

use std::{thread, time::Duration};
use apprules::AppRules;
use platform::{get_foreground_process_name, list_running_process_names, show_distraction_popup};
use logger::log_event;
use metrics::Metrics;
use session::FocusSession;

fn main() {
    let apprules = AppRules::new();
    let mut metrics = Metrics::new();
    let mut current_session: Option<FocusSession> = None;
    let mut last_distraction_app: Option<String> = None;
    
    loop {
        let running_processes = list_running_process_names();
        let any_work_app_running = running_processes.iter().any(|name| apprules.is_work_app(name));

        if let Some(proc) = get_foreground_process_name() {
            let proc_lc = proc.to_lowercase();
            let is_work = apprules.is_work_app(&proc_lc);
            let is_blocked = apprules.is_blocked(&proc_lc);
            log_event(&proc, is_blocked);
            
            if is_blocked {
                println!("    Blocked app in focus: {}", proc);
                if current_session.is_some() {
                    if last_distraction_app.as_deref() != Some(&proc_lc) {
                        show_distraction_popup(&proc);
                        last_distraction_app = Some(proc_lc.clone());
                    }
                }
            } else if is_work {
                println!("    Work app in focus: {}", proc);
                last_distraction_app = None;
            } else {
                println!("    Neutral app in focus: {}", proc);
                last_distraction_app = None;
            }

            metrics.update(&proc, is_blocked);

            // start a session if not already in one and this is a work apps
            if any_work_app_running && current_session.is_none() {
                println!("\n--- Focus Session Started ---");
                // collect all running work apps at session start
                let work_apps: Vec<String> = running_processes.iter().filter(|name| apprules.is_work_app(name)).cloned().collect();
                current_session = Some(FocusSession::new(work_apps));
            }
            // if already in a session, update work_apps if new work app appears
            if let Some(session) = current_session.as_mut() {
                for name in running_processes.iter().filter(|name| apprules.is_work_app(name)) {
                    if !session.work_apps.contains(&proc) {
                        session.work_apps.push(name.clone());
                    }
                }
            }
        } else {
            println!("Could not detect foreground app.");
            last_distraction_app = None;
        }

        // end session if no whitelisted app is in the foregound
        if let Some(session) = current_session.as_mut() {
            if !any_work_app_running {
                session.end();
                println!("\n--- Focus Session Ended ---");
                if let Some(duration) = session.duration() {
                    println!("Session Duration: {:.2?}", duration);
                }
                println!("Apps Used: {:?}", session.work_apps);
                session.log_to_file();
                current_session = None;
            }
        }

        if metrics.should_log_summary() {
            metrics.log_summary();
        }
        
        thread::sleep(Duration::from_secs(3));
    }
}
