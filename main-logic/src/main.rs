mod logger;
mod blacklist;
mod platform;
mod metrics;
mod session;

use std::{thread, time::Duration};
use blacklist::Blacklist;
use platform::get_foreground_process_name;
use logger::log_event;
use metrics::Metrics;
use session::FocusSession;

fn main() {
    let blacklist = Blacklist::new();
    let mut metrics = Metrics::new();
    let mut current_session: Option<FocusSession> = None;
    
    loop {
        if let Some(proc) = get_foreground_process_name() {
            let blocked = blacklist.is_blocked(&proc);
            log_event(&proc, blocked);
            
            if blocked {
                println!("    Blocked app in focus: {}", proc);
            } else {
                println!("    Allowed app in focus: {}", proc);
            }

            metrics.update(&proc, blocked);

            // start a session if not already in one and this is a work apps
            if !blocked && current_session.is_none() {
                println!("\n--- Focus Session Started ---");
                current_session = Some(FocusSession::new(vec![proc.clone()]));
            }
            // if already in a session, update work_apps if new app
            if let Some(session) = current_session.as_mut() {
                if !session.work_apps.contains(&proc) && !blocked {
                    session.work_apps.push(proc.clone());
                }
            }
        } else {
            println!("Could not detect foreground app.");
        }

        // end session if no work app is in focus
        if let Some(session) = current_session.as_mut() {
            if let Some(proc) = get_foreground_process_name() {
                if blacklist.is_blocked(&proc) {
                    // still in focus session, do nothing
                } else {
                    // still in focus session, do nothing
                }
            } else {
                // no app detected, end session
                session.end();
                println!("\n--- Focus Session ended ----");
                if let Some(duration) = session.duration() {
                    println!("Session Duration: {:.2?}", duration);
                }
                println!("Apps Used: {:?}", session.work_apps);
                current_session = None;
            }
        }

        if metrics.should_log_summary() {
            metrics.log_summary();
        }
        
        thread::sleep(Duration::from_secs(3));
    }
}
