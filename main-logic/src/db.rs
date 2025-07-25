//! Database module: handles SQLite connection, schema, and event/session storage.

use rusqlite::{params, Connection};
use crate::error::SynapseError;
use crate::types::AppUsageEvent;
use uuid::Uuid;

/// Handle for interacting with the SQLite database.
pub struct DbHandle {
    /// The underlying SQLite connection.
    pub(crate) conn: Connection,
}

impl DbHandle {
    /// Opens or creates the SQLite database and ensures required tables exist.
    ///
    /// # Errors
    /// Returns `SynapseError` if the database cannot be opened or tables cannot be created.
    pub fn new() -> Result<Self, SynapseError> {
        let conn = Connection::open("synapse_metrics.db")
            .map_err(|e| SynapseError::Db(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
        // Enable foreign key support
        conn.execute("PRAGMA foreign_keys = ON", []).ok();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS focus_sessions (
                id TEXT PRIMARY KEY,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                work_apps TEXT,
                distraction_attempts INTEGER
            )",
            [],
        ).map_err(|e| SynapseError::Db(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_usage_events (
                id TEXT PRIMARY KEY,
                process_name TEXT NOT NULL,
                status TEXT NOT NULL,
                session_id TEXT,
                start_time INTEGER,
                end_time INTEGER,
                duration_secs INTEGER,
                FOREIGN KEY(session_id) REFERENCES focus_sessions(id)
            )",
            [],
        ).map_err(|e| SynapseError::Db(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
        Ok(DbHandle { conn })
    }

    /// Construct DbHandle with an in-memory SQLite database (for tests and integration).
    pub fn test_in_memory() -> Self {
        DbHandle { conn: Connection::open_in_memory().unwrap() }
    }

    /// Logs an app usage event to the database.
    ///
    /// # Arguments
    /// * `process_name` - Name of the process
    /// * `status` - Status of the app usage (e.g., "blocked", "active", "distraction")
    /// * `session_id` - Associated session ID
    /// * `start_time`, `end_time`, `duration_secs` - Timing info
    ///
    /// # Errors
    /// Returns `SynapseError` if the insert fails.
    pub fn log_event(&self, process_name: &str, status: &str, session_id: Option<Uuid>, start_time: Option<i64>, end_time: Option<i64>, duration_secs: Option<i64>) -> Result<(), SynapseError> {
        self.conn.execute(
            "INSERT INTO app_usage_events (process_name, status, session_id, start_time, end_time, duration_secs) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![process_name, status, session_id.map(|u| u.to_string()), start_time, end_time, duration_secs],
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
    pub fn insert_session(&self, start_time: i64) -> Result<Uuid, SynapseError> {
        let session_id = Uuid::new_v4();
        self.conn.execute(
            "INSERT INTO focus_sessions (id, start_time, distraction_attempts) VALUES (?1, ?2, 0)",
            params![session_id.to_string(), start_time],
        ).map_err(|e| SynapseError::Db(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
        Ok(session_id)
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
    pub fn update_session(&self, session_id: Uuid, end_time: i64, work_apps: &str, distraction_attempts: i32) -> Result<(), SynapseError> {
        self.conn.execute(
            "UPDATE focus_sessions SET end_time = ?1, work_apps = ?2, distraction_attempts = ?3 WHERE id = ?4",
            params![end_time, work_apps, distraction_attempts, session_id.to_string()],
        ).map_err(|e| SynapseError::Db(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
        Ok(())
    }

    /// Updates an app usage event with end_time and duration_secs.
    ///
    /// # Arguments
    /// * `event_id` - The id of the app_usage_event row
    /// * `end_time` - The end time (seconds since epoch)
    /// * `duration_secs` - The duration in seconds
    ///
    /// # Errors
    /// Returns `SynapseError` if the update fails.
    pub fn update_app_usage_event(&self, event_id: Uuid, end_time: i64, duration_secs: i64) -> Result<(), SynapseError> {
        self.conn.execute(
            "UPDATE app_usage_events SET end_time = ?1, duration_secs = ?2 WHERE id = ?3",
            params![end_time, duration_secs, event_id.to_string()],
        ).map_err(|e| SynapseError::Db(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
        Ok(())
    }

    /// Inserts a new app usage event and returns its row ID.
    ///
    /// # Arguments
    /// * `process_name` - Name of the process
    /// * `status` - Status of the app usage (e.g., "blocked", "active", "distraction")
    /// * `session_id` - Associated session ID
    /// * `start_time` - When the app came into focus
    ///
    /// # Errors
    /// Returns `SynapseError` if the insert fails.
    pub fn insert_app_usage_event(&self, process_name: &str, status: &str, session_id: Option<Uuid>, start_time: i64, end_time: i64, duration_secs: i64) -> Result<Uuid, SynapseError> {
        let event_id = Uuid::new_v4();
        self.conn.execute(
            "INSERT INTO app_usage_events (id, process_name, status, session_id, start_time, end_time, duration_secs) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![event_id.to_string(), process_name, status, session_id.map(|u| u.to_string()), start_time, end_time, duration_secs],
        ).map_err(|e| SynapseError::Db(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
        Ok(event_id)
    }

    pub fn get_app_usage_events_for_session(&self, session_id: Uuid) -> Result<Vec<AppUsageEvent>, SynapseError> {
        let mut stmt = self.conn.prepare(
            "SELECT process_name, status, session_id, start_time, end_time, duration_secs FROM app_usage_events WHERE session_id = ?1"
        )?;
        let rows = stmt.query_map([session_id.to_string()], |row| {
            Ok(AppUsageEvent {
                id: Uuid::new_v4(), // Dummy value for test, replace with actual if available
                process_name: row.get(0)?,
                status: row.get(1)?,
                session_id: row.get(2).ok().and_then(|s: String| Uuid::parse_str(&s).ok()),
                start_time: row.get(3)?,
                end_time: row.get(4)?,
                duration_secs: row.get(5)?,
            })
        })?;
        let mut events = Vec::new();
        for event in rows {
            events.push(event?);
        }
        Ok(events)
    }

    pub fn execute_sql(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> rusqlite::Result<usize> {
        self.conn.execute(sql, params)
    }

    pub fn test_conn(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

pub trait DbConn {
    fn conn(&self) -> &rusqlite::Connection;
}
impl DbConn for DbHandle {
    fn conn(&self) -> &rusqlite::Connection {
        &self.conn
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
                id TEXT PRIMARY KEY,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                work_apps TEXT,
                distraction_attempts INTEGER
            )",
            [],
        ).unwrap();
        let id = db.insert_session(12345).unwrap();
        assert_ne!(id, Uuid::nil());
    }

    #[test]
    fn logs_event_and_queries() {
        let db = db_in_memory();
        db.conn.execute(
            "CREATE TABLE IF NOT EXISTS app_usage_events (
                id TEXT PRIMARY KEY,
                process_name TEXT NOT NULL,
                status TEXT NOT NULL,
                session_id TEXT,
                start_time INTEGER,
                end_time INTEGER,
                duration_secs INTEGER
            )",
            [],
        ).unwrap();
        let uuid = Uuid::new_v4();
        db.log_event("test.exe", "active", Some(uuid), Some(123), Some(124), Some(1)).unwrap();
        let mut stmt = db.conn.prepare("SELECT process_name FROM app_usage_events WHERE session_id = ?").unwrap();
        let mut rows = stmt.query([uuid.to_string()]).unwrap();
        let row = rows.next().unwrap().unwrap();
        let name: String = row.get(0).unwrap();
        assert_eq!(name, "test.exe");
    }

    #[test]
    fn update_session_and_query() {
        let db = db_in_memory();
        db.conn.execute(
            "CREATE TABLE IF NOT EXISTS focus_sessions (
                id TEXT PRIMARY KEY,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                work_apps TEXT,
                distraction_attempts INTEGER
            )",
            [],
        ).unwrap();
        let id = db.insert_session(12345).unwrap();
        db.update_session(id, 54321, "notepad.exe,word.exe", 2).unwrap();
        let mut stmt = db.conn.prepare("SELECT end_time, work_apps, distraction_attempts FROM focus_sessions WHERE id = ?").unwrap();
        let mut rows = stmt.query([id.to_string()]).unwrap();
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
        let result = db.log_event("test.exe", "active", Some(Uuid::new_v4()), Some(123), Some(124), Some(1));
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
        let result = db.update_session(Uuid::new_v4(), 54321, "notepad.exe,word.exe", 2);
        assert!(result.is_err());
    }
} 
