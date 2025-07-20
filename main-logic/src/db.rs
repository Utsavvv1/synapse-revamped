use rusqlite::{params, Connection};
use crate::error::SynapseError;

pub struct DbHandle {
    conn: Connection,
}

impl DbHandle {
    pub fn new() -> Result<Self, SynapseError> {
        let conn = Connection::open("synapse_metrics.db")?;
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
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS focus_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                work_apps TEXT,
                distraction_attempts INTEGER
            )",
            [],
        )?;
        Ok(DbHandle { conn })
    }

    /// Test-only: construct DbHandle with an in-memory SQLite database.
    #[cfg(test)]
    pub fn test_in_memory() -> Self {
        DbHandle { conn: Connection::open_in_memory().unwrap() }
    }

    /// Test-only: get a mutable reference to the underlying connection.
    #[cfg(test)]
    pub fn test_conn(&mut self) -> &mut Connection {
        &mut self.conn
    }

    pub fn log_event(&self, timestamp: i64, process_name: &str, is_blocked: bool, distraction: Option<bool>, session_id: Option<i64>, start_time: Option<i64>, end_time: Option<i64>, duration_secs: Option<i64>) -> Result<(), SynapseError> {
        self.conn.execute(
            "INSERT INTO app_usage_events (timestamp, process_name, is_blocked, distraction, session_id, start_time, end_time, duration_secs) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![timestamp, process_name, is_blocked, distraction, session_id, start_time, end_time, duration_secs],
        )?;
        Ok(())
    }

    pub fn insert_session(&self, start_time: i64) -> Result<i64, SynapseError> {
        self.conn.execute(
            "INSERT INTO focus_sessions (start_time, distraction_attempts) VALUES (?1, 0)",
            params![start_time],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_session(&self, session_id: i64, end_time: i64, work_apps: &str, distraction_attempts: i32) -> Result<(), SynapseError> {
        self.conn.execute(
            "UPDATE focus_sessions SET end_time = ?1, work_apps = ?2, distraction_attempts = ?3 WHERE id = ?4",
            params![end_time, work_apps, distraction_attempts, session_id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
} 
