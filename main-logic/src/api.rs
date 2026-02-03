//! API functions exposed to the frontend (Tauri). Only these should be visible to the Tauri app.

use crate::db::DbHandle;
use crate::error::SynapseError;
#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

/// Returns the total focus time (in seconds) for today.
pub fn total_focus_time_today(db: &DbHandle) -> Result<i64, SynapseError> {
    let (start_of_day, end_of_day) = today_bounds();
    let mut stmt = db.conn().prepare(
        "SELECT SUM(COALESCE(end_time, strftime('%s','now')) - start_time) FROM focus_sessions WHERE start_time >= ?1 AND start_time < ?2"
    )?;
    let total: Option<i64> = stmt
        .query_row([start_of_day, end_of_day], |row| row.get(0))
        .ok();
    Ok(total.unwrap_or(0))
}

/// Returns the total number of distractions today.
pub fn total_distractions_today(db: &DbHandle) -> Result<i64, SynapseError> {
    let (start_of_day, end_of_day) = today_bounds();
    let mut stmt = db.conn().prepare(
        "SELECT SUM(distraction_attempts) FROM focus_sessions WHERE start_time >= ?1 AND start_time < ?2"
    )?;
    let total: Option<i64> = stmt
        .query_row([start_of_day, end_of_day], |row| row.get(0))
        .ok();
    Ok(total.unwrap_or(0))
}

/// Returns the total number of focus sessions started today.
pub fn total_focus_sessions_today(db: &DbHandle) -> Result<i64, SynapseError> {
    let (start_of_day, end_of_day) = today_bounds();
    let mut stmt = db.conn().prepare(
        "SELECT COUNT(*) FROM focus_sessions WHERE start_time >= ?1 AND start_time < ?2",
    )?;
    let count: Option<i64> = stmt
        .query_row([start_of_day, end_of_day], |row| row.get(0))
        .ok();
    Ok(count.unwrap_or(0))
}

#[cfg(target_os = "windows")]
/// Returns a list of installed (app_name, exe_name) tuples from the Windows registry.
pub fn get_installed_apps_api() -> Vec<(String, String)> {
    use std::path::Path;
    use winreg::enums::*;
    use winreg::RegKey;

    fn extract_exe_name(path: &str) -> Option<String> {
        // Strip quotes/arguments/comma and extract only filename ending in .exe
        path.split(|c| c == ',' || c == ' ' || c == '\"')
            .find(|s| s.to_lowercase().ends_with(".exe"))
            .and_then(|p| {
                Path::new(p)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
            })
    }

    let mut apps = Vec::new();

    let uninstall_paths = [
        (
            RegKey::predef(HKEY_LOCAL_MACHINE),
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            RegKey::predef(HKEY_LOCAL_MACHINE),
            r"SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            RegKey::predef(HKEY_CURRENT_USER),
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
    ];

    for (hive, path) in uninstall_paths.iter() {
        if let Ok(uninstall) = hive.open_subkey(path) {
            for item in uninstall.enum_keys().flatten() {
                if let Ok(subkey) = uninstall.open_subkey(&item) {
                    let display_name: Result<String, _> = subkey.get_value("DisplayName");
                    let is_system_component =
                        subkey.get_value::<u32, _>("SystemComponent").unwrap_or(0) == 1;

                    if let Ok(name) = display_name {
                        if is_system_component || name.trim().is_empty() {
                            continue;
                        }

                        // Try DisplayIcon first, fallback to UninstallString
                        let exe_source = subkey
                            .get_value::<String, _>("DisplayIcon")
                            .ok()
                            .or_else(|| subkey.get_value("UninstallString").ok());

                        if let Some(source) = exe_source {
                            if let Some(exe_name) = extract_exe_name(&source) {
                                apps.push((name.trim().to_string(), exe_name));
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort and deduplicate by app name
    apps.sort_by(|a, b| a.0.cmp(&b.0));
    apps.dedup_by(|a, b| a.0 == b.0);
    apps
}

use chrono::{Local, TimeZone};

/// Helper: Returns (start_of_day, end_of_day) as UNIX timestamps for today in Local Time.
fn today_bounds() -> (i64, i64) {
    let now = Local::now();
    // Get start of today in local time
    let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
    // Convert to UTC timestamp for DB comparison
    // We need to assume the DB stores UTC timestamps (SystemTime::now())
    // but we want to filter for records that fall within "Today" in Local time.
    // So distinct from UTC midnight.
    // Example: If Local is IST (UTC+5:30), Today 00:00 is Yesterday 18:30 UTC.
    // any timestamp >= Yesterday 18:30 UTC belongs to Today in IST.

    // Convert naive datetime (local) back to timestamp using the offset
    let start_timestamp = Local.from_local_datetime(&start).unwrap().timestamp();
    let end_timestamp = start_timestamp + 86400;
    (start_timestamp, end_timestamp)
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
