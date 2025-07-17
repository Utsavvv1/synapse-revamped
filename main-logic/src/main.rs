mod logger;
mod blacklist;
mod platform;
mod metrics;

use std::{thread, time::Duration};
use blacklist::Blacklist;
use platform::get_foreground_process_name;
use logger::log_event;
use metrics::Metrics;

fn main() {
    let blacklist = Blacklist::new();
    let mut metrics = Metrics::new();
    
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
        } else {
            println!("Could not detect foreground app.");
        }

        if metrics.should_log_summary() {
            metrics.log_summary();
        }
        
        thread::sleep(Duration::from_secs(3));
    }
}
