//! API functions exposed to the frontend (Tauri). Only these should be visible to the Tauri app.

use crate::db::DbHandle;
use crate::error::SynapseError;
use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the total focus time (in seconds) for today.
pub fn total_focus_time_today(db: &DbHandle) -> Result<i64, SynapseError> {
    let (start_of_day, end_of_day) = today_bounds();
    let mut stmt = db.conn().prepare(
        "SELECT SUM(COALESCE(end_time, strftime('%s','now')) - start_time) FROM focus_sessions WHERE start_time >= ?1 AND start_time < ?2"
    )?;
    let total: Option<i64> = stmt.query_row([start_of_day, end_of_day], |row| row.get(0)).ok();
    Ok(total.unwrap_or(0))
}

/// Returns the total number of distractions today.
pub fn total_distractions_today(db: &DbHandle) -> Result<i64, SynapseError> {
    let (start_of_day, end_of_day) = today_bounds();
    let mut stmt = db.conn().prepare(
        "SELECT SUM(distraction_attempts) FROM focus_sessions WHERE start_time >= ?1 AND start_time < ?2"
    )?;
    let total: Option<i64> = stmt.query_row([start_of_day, end_of_day], |row| row.get(0)).ok();
    Ok(total.unwrap_or(0))
}

/// Returns the total number of focus sessions started today.
pub fn total_focus_sessions_today(db: &DbHandle) -> Result<i64, SynapseError> {
    let (start_of_day, end_of_day) = today_bounds();
    let mut stmt = db.conn().prepare(
        "SELECT COUNT(*) FROM focus_sessions WHERE start_time >= ?1 AND start_time < ?2"
    )?;
    let count: Option<i64> = stmt.query_row([start_of_day, end_of_day], |row| row.get(0)).ok();
    Ok(count.unwrap_or(0))
}

/// Helper: Returns (start_of_day, end_of_day) as UNIX timestamps for today in UTC.
/// NOTE: This uses UTC, not local time, to avoid external crates. If you need local time, use a crate or OS-specific API.
fn today_bounds() -> (i64, i64) {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
    // Calculate UTC midnight for today
    let days_since_epoch = now / 86400;
    let start = days_since_epoch * 86400;
    let end = start + 86400;
    (start, end)
}

// Extension trait to access the private conn field safely
trait DbConn {
    fn conn(&self) -> &rusqlite::Connection;
}

impl DbConn for DbHandle {
    fn conn(&self) -> &rusqlite::Connection {
        // SAFETY: We are only exposing for read-only queries
        &self.conn
    }
} 