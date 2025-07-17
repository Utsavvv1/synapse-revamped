mod logger;
mod monitor;
mod blacklist;

use std::{thread, time::Duration};
use blacklist::Blacklist;
use monitor::get_foreground_process_name;
use logger::log_event;

fn main() {
    let blacklist = Blacklist::new();
    
    loop {
        if let Some(proc) = get_foreground_process_name() {
            let blocked = blacklist.is_blocked(&proc);
            log_event(&proc, blocked);
            
            if blocked {
                println!("    Blocked app in focus: {}", proc);
            } else {
                println!("    Allowed app in focus: {}", proc);
            }
        } else {
            println!("Could not detect foreground app.");
        }
        
        thread::sleep(Duration::from_secs(3));
    }
}
