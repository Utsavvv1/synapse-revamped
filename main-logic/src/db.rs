//! Database module: handles SQLite connection, schema, and event/session storage.

use rusqlite::{params, Connection};
use crate::error::SynapseError;

/// Handle for interacting with the SQLite database.
pub struct DbHandle {
    /// The underlying SQLite connection.
    pub conn: Connection,
}

impl DbHandle {
    /// Opens or creates the SQLite database and ensures required tables exist.
    ///
    /// # Errors
    /// Returns `SynapseError` if the database cannot be opened or tables cannot be created.
    pub fn new() -> Result<Self, SynapseError> {
        let conn = Connection::open("synapse_metrics.db")
            .map_err(|e| SynapseError::Db(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
        conn.execute(
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
        ).map_err(|e| SynapseError::Db(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS focus_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                work_apps TEXT,
                distraction_attempts INTEGER
            )",
            [],
        ).map_err(|e| SynapseError::Db(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
        Ok(DbHandle { conn })
    }

    /// Construct DbHandle with an in-memory SQLite database (for tests and integration).
    pub fn test_in_memory() -> Self {
        DbHandle { conn: Connection::open_in_memory().unwrap() }
    }

    /// Get a mutable reference to the underlying connection (for tests and integration).
    pub fn test_conn(&mut self) -> &mut Connection {
        &mut self.conn
    }

    /// Logs an app usage event to the database.
    ///
    /// # Arguments
    /// * `timestamp` - Event timestamp (seconds since epoch)
    /// * `process_name` - Name of the process
    /// * `is_blocked` - Whether the process was blocked
    /// * `distraction` - Whether this was a distraction attempt
    /// * `session_id` - Associated session ID
    /// * `start_time`, `end_time`, `duration_secs` - Timing info
    ///
    /// # Errors
    /// Returns `SynapseError` if the insert fails.
    pub fn log_event(&self, timestamp: i64, process_name: &str, is_blocked: bool, distraction: Option<bool>, session_id: Option<i64>, start_time: Option<i64>, end_time: Option<i64>, duration_secs: Option<i64>) -> Result<(), SynapseError> {
        self.conn.execute(
            "INSERT INTO app_usage_events (timestamp, process_name, is_blocked, distraction, session_id, start_time, end_time, duration_secs) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![timestamp, process_name, is_blocked, distraction, session_id, start_time, end_time, duration_secs],
        ).map_err(|e| SynapseError::Db(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
        Ok(())
    }

    /// Inserts a new focus session into the database.
    ///
    /// # Arguments
    /// * `start_time` - Session start time (seconds since epoch)
    ///
    /// # Errors
    /// Returns `SynapseError` if the insert fails.
    pub fn insert_session(&self, start_time: i64) -> Result<i64, SynapseError> {
        self.conn.execute(
            "INSERT INTO focus_sessions (start_time, distraction_attempts) VALUES (?1, 0)",
            params![start_time],
        ).map_err(|e| SynapseError::Db(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Updates a focus session with end time, apps used, and distraction attempts.
    ///
    /// # Arguments
    /// * `session_id` - Session ID
    /// * `end_time` - Session end time (seconds since epoch)
    /// * `work_apps` - Comma-separated list of apps used
    /// * `distraction_attempts` - Number of distractions
    ///
    /// # Errors
    /// Returns `SynapseError` if the update fails.
    pub fn update_session(&self, session_id: i64, end_time: i64, work_apps: &str, distraction_attempts: i32) -> Result<(), SynapseError> {
        self.conn.execute(
            "UPDATE focus_sessions SET end_time = ?1, work_apps = ?2, distraction_attempts = ?3 WHERE id = ?4",
            params![end_time, work_apps, distraction_attempts, session_id],
        ).map_err(|e| SynapseError::Db(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Error as RusqliteError;

    fn db_in_memory() -> DbHandle {
        DbHandle { conn: Connection::open_in_memory().unwrap() }
    }

    #[test]
    fn creates_tables_and_inserts_session() {
        let db = db_in_memory();
        db.conn.execute(
            "CREATE TABLE IF NOT EXISTS focus_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                work_apps TEXT,
                distraction_attempts INTEGER
            )",
            [],
        ).unwrap();
        let id = db.insert_session(12345).unwrap();
        assert!(id > 0);
    }

    #[test]
    fn logs_event_and_queries() {
        let db = db_in_memory();
        db.conn.execute(
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
        db.log_event(123, "test.exe", false, Some(false), Some(1), Some(123), Some(124), Some(1)).unwrap();
        let mut stmt = db.conn.prepare("SELECT process_name FROM app_usage_events WHERE session_id = 1").unwrap();
        let mut rows = stmt.query([]).unwrap();
        let row = rows.next().unwrap().unwrap();
        let name: String = row.get(0).unwrap();
        assert_eq!(name, "test.exe");
    }

    #[test]
    fn update_session_and_query() {
        let db = db_in_memory();
        db.conn.execute(
            "CREATE TABLE IF NOT EXISTS focus_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                work_apps TEXT,
                distraction_attempts INTEGER
            )",
            [],
        ).unwrap();
        let id = db.insert_session(12345).unwrap();
        db.update_session(id, 54321, "notepad.exe,word.exe", 2).unwrap();
        let mut stmt = db.conn.prepare("SELECT end_time, work_apps, distraction_attempts FROM focus_sessions WHERE id = ?1").unwrap();
        let mut rows = stmt.query([id]).unwrap();
        let row = rows.next().unwrap().unwrap();
        let end_time: i64 = row.get(0).unwrap();
        let work_apps: String = row.get(1).unwrap();
        let distraction_attempts: i32 = row.get(2).unwrap();
        assert_eq!(end_time, 54321);
        assert_eq!(work_apps, "notepad.exe,word.exe");
        assert_eq!(distraction_attempts, 2);
    }

    #[test]
    fn log_event_invalid_table() {
        let db = db_in_memory();
        // Do not create the table, should error
        let result = db.log_event(123, "test.exe", false, Some(false), Some(1), Some(123), Some(124), Some(1));
        assert!(result.is_err());
    }

    #[test]
    fn insert_session_invalid_table() {
        let db = db_in_memory();
        // Do not create the table, should error
        let result = db.insert_session(12345);
        assert!(result.is_err());
    }

    #[test]
    fn update_session_invalid_table() {
        let db = db_in_memory();
        // Do not create the table, should error
        let result = db.update_session(1, 54321, "notepad.exe,word.exe", 2);
        assert!(result.is_err());
    }
} 
