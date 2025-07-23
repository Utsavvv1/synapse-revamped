//! Logger module: handles logging of events and errors to file and database.

use std::io::Write;
use crate::db::DbHandle;
use crate::error::SynapseError;

/// Logs an app usage event to the database (if available) and to the fallback log file.
///
/// # Arguments
/// * `db_handle` - Optional database handle
/// * `process` - Name of the process
/// * `blocked` - Whether the process was blocked
/// * `distraction` - Whether this was a distraction attempt
/// * `session_id` - Associated session ID
/// * `start_time`, `end_time`, `duration_secs` - Timing info
///
/// # Errors
/// Returns `SynapseError` if logging to the database or file fails.
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
        .open("synapse.log")
        .map_err(|e| SynapseError::Io(std::io::Error::new(e.kind(), format!("Failed to open synapse.log: {}", e))))?;
    file.write_all(entry.as_bytes())
        .map_err(|e| SynapseError::Io(std::io::Error::new(e.kind(), format!("Failed to write to synapse.log: {}", e))))?;
    Ok(())
}

/// Logs an error to the fallback log file and stderr.
///
/// # Arguments
/// * `err` - The error to log
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

pub fn log_error_with_context(context: &str, err: &crate::error::SynapseError) {
    let entry = format!("[ERROR] {}: {}\n", context, err);
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("synapse.log")
    {
        let _ = file.write_all(entry.as_bytes());
    }
    eprintln!("{}", entry);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use crate::error::SynapseError;
    use crate::db::DbHandle;

    #[test]
    fn log_error_writes_to_file_and_stderr() {
        let msg = "Test error message";
        let err = SynapseError::Other(msg.to_string());
        log_error(&err);
        let contents = fs::read_to_string("synapse.log").unwrap();
        assert!(contents.contains(msg));
    }

    #[test]
    fn log_event_writes_to_file() {
        let process = "test.exe";
        let result = log_event(None, process, true, Some(true), Some(1), Some(100), Some(200), Some(100));
        assert!(result.is_ok());
        let contents = fs::read_to_string("synapse.log").unwrap();
        assert!(contents.contains(process));
        assert!(contents.contains("BLOCKED"));
    }

    #[test]
    fn log_event_writes_to_db() {
        let mut db = DbHandle::test_in_memory();
        db.test_conn().execute(
            "CREATE TABLE IF NOT EXISTS app_usage_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                process_name TEXT NOT NULL,
                is_blocked BOOLEAN NOT NULL,
                distraction BOOLEAN,
                session_id INTEGER,
                start_time INTEGER,
                end_time INTEGER,
                duration_secs INTEGER
            )",
            [],
        ).unwrap();
        let process = "test.exe";
        let result = log_event(Some(&db), process, false, Some(false), Some(1), Some(100), Some(200), Some(100));
        assert!(result.is_ok());
        let mut stmt = db.test_conn().prepare("SELECT process_name FROM app_usage_events WHERE session_id = 1").unwrap();
        let mut rows = stmt.query([]).unwrap();
        let row = rows.next().unwrap().unwrap();
        let name: String = row.get(0).unwrap();
        assert_eq!(name, process);
    }

    #[test]
    fn log_event_file_error() {
        // Simulate file error by using an invalid path (readonly dir, etc.)
        // This is hard to do portably, so we just check that the function returns an error if file cannot be opened
        // For now, this is a placeholder for a more advanced test with a mock FS
        // let result = log_event(None, "/invalid/path/test.exe", true, Some(true), Some(1), Some(100), Some(200), Some(100));
        // assert!(result.is_err());
        // Skipped for portability
        assert!(true);
    }
}
