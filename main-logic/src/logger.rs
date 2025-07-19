use std::io::Write;
use crate::db::DbHandle;
use crate::error::SynapseError;

pub fn log_event(db_handle: Option<&DbHandle>, process: &str, blocked: bool, distraction: Option<bool>, session_id: Option<i64>, start_time: Option<i64>, end_time: Option<i64>, duration_secs: Option<i64>) -> Result<(), SynapseError> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    // Log to SQLite if available
    if let Some(db) = db_handle {
        db.log_event(
            timestamp,
            process,
            blocked,
            distraction,
            session_id,
            start_time,
            end_time,
            duration_secs,
        )?;
    }

    // Fallback: also log to file as before
    let status = if blocked { "BLOCKED" } else { "ALLOWED" };
    let entry = format!("[{}] {} -> {}\n", timestamp, status, process);
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("synapse.log")?;
    file.write_all(entry.as_bytes())?;
    Ok(())
}

pub fn log_error(err: &SynapseError) {
    let entry = format!("[ERROR] {}\n", err);
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("synapse.log")
    {
        let _ = file.write_all(entry.as_bytes());
    }
    eprintln!("{}", entry);
}
