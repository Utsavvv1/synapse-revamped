use std::fs::{OpenOptions};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::db::DbHandle;

pub fn log_event(db_handle: Option<&DbHandle>, process: &str, blocked: bool, distraction: Option<bool>, session_id: Option<i64>, start_time: Option<i64>, end_time: Option<i64>, duration_secs: Option<i64>) {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Log to SQLite if available
    if let Some(db) = db_handle {
        let _ = db.log_event(
            timestamp,
            process,
            blocked,
            distraction,
            session_id,
            start_time,
            end_time,
            duration_secs,
        );
    }

    // Fallback: also log to file as before
    let status = if blocked { "BLOCKED" } else { "ALLOWED" };
    let entry = format!("[{}] {} -> {}\n", timestamp, status, process);
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("synapse.log")
    {
        let _ = file.write_all(entry.as_bytes());
    }
}
