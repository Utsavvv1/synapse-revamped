mod monitor;

use std::thread::sleep;
use std::time::Duration;

fn main() {
    loop {
        if let Some(app) = monitor::get_foreground_process_name() {
            println!("Foreground process: {app}");
        } else {
            println!("Could not detect foreground process.");
        }
        sleep(Duration::from_secs(2));
    }
}
