use std::fs::{OpenOptions};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::session::FocusSession;
use serde_json;

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

pub fn log_session_event(session: &FocusSession, is_start: bool) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let event = if is_start { "SESSION_START" } else { "SESSION_END" };
    let apps = session.work_apps.join(", ");
    let entry = format!("[{}] {}: apps=[{}]\n", timestamp, event, apps);
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("synapse.log")
    {
        let _ = file.write_all(entry.as_bytes());
    }
}

pub fn log_session_json(session: &FocusSession) {
    if let Ok(json) = serde_json::to_string(session) {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("synapse.log")
        {
            let _ = writeln!(file, "SESSION_JSON: {}", json);
        }
    }
}
