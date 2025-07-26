//! API functions exposed to the frontend (Tauri). Only these should be visible to the Tauri app.

use crate::db::DbHandle;
use crate::error::SynapseError;
use std::time::{SystemTime, UNIX_EPOCH};
use winreg::enums::*;
use winreg::RegKey;

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

#[cfg(target_os = "windows")]
/// Returns a list of installed application display names from the Windows registry.
pub fn get_installed_apps_api() -> Vec<String> {
    let mut apps = Vec::new();
    let uninstall_paths = [
        (RegKey::predef(HKEY_LOCAL_MACHINE), r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
        (RegKey::predef(HKEY_LOCAL_MACHINE), r"SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall"),
        (RegKey::predef(HKEY_CURRENT_USER), r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
    ];    
    for (hive, path) in uninstall_paths.iter() {
        if let Ok(uninstall) = hive.open_subkey(path) {
            for item in uninstall.enum_keys().flatten() {
                if let Ok(subkey) = uninstall.open_subkey(&item) {
                    let display_name: Result<String, _> = subkey.get_value("DisplayName");
                    let is_system_component: bool = subkey
                        .get_value::<u32, _>("SystemComponent")
                        .unwrap_or(0) == 1;
                    if let Ok(name) = display_name {
                        if !is_system_component && !name.trim().is_empty() {
                            apps.push(name);
                        }
                    }
                }
            }
        }
    }
    apps.sort();
    apps.dedup();
    apps
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