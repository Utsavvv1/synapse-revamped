mod monitor;
mod blacklist;

use std::{thread, time::Duration};
use blacklist::Blacklist;
use monitor::get_foreground_process_name;

fn main() {
    let blacklist = Blacklist::new();
    
    loop {
        if let Some(proc) = get_foreground_process_name() {
            if blacklist.is_blocked(&proc) {
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
