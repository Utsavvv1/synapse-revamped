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
