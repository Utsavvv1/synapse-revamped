use std::fs::{OpenOptions};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn log_event(process: &str, blocked: bool) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let status = if blocked { "BLOCKED" } else { "ALLOWED" };
    let entry = format!("[{}] {} -> {}\n", timestamp, status, process);

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("synapse.log")
    {
        let _ = file.write_all(entry.as_bytes());
    }
}
