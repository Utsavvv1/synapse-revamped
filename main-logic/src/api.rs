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
        (RegKey::predef(HKEY_LOCAL_MACHINE), r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
        (RegKey::predef(HKEY_LOCAL_MACHINE), r"SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall"),
        (RegKey::predef(HKEY_CURRENT_USER), r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
    ];

    for (hive, path) in uninstall_paths.iter() {
        if let Ok(uninstall) = hive.open_subkey(path) {
            for item in uninstall.enum_keys().flatten() {
                if let Ok(subkey) = uninstall.open_subkey(&item) {
                    let display_name: Result<String, _> = subkey.get_value("DisplayName");
                    let is_system_component = subkey
                        .get_value::<u32, _>("SystemComponent")
                        .unwrap_or(0) == 1;

                    if let Ok(name) = display_name {
                        if is_system_component || name.trim().is_empty() {
                            continue;
                        }

                        // Try DisplayIcon first, fallback to UninstallString
                        let exe_source = subkey.get_value::<String, _>("DisplayIcon")
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

#[cfg(target_os = "linux")]
pub fn get_installed_apps_api() -> Vec<(String, String)> {
    use std::collections::HashSet;
    use std::fs;
    use std::path::{Path, PathBuf};
    log::debug!("[TAURI] Linux get_installed_apps_api called");
    fn extract_exe_name(exec_line: &str) -> Option<String> {
        // Take the first token (before any whitespace or placeholder %)
        let cmd = exec_line
            .split_whitespace()
            .find(|&s| !s.starts_with('%'))?;
        // If it's an absolute path, extract file name, otherwise use as-is
        let file_name = Path::new(cmd)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(cmd);
        Some(file_name.to_string())
    }

    fn parse_desktop_file(path: &Path) -> Option<(String, String)> {
        let content = fs::read_to_string(path).ok()?;
        let mut name = None;
        let mut exec = None;

        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("Name=") {
                name = Some(rest.trim().to_string());
            } else if let Some(rest) = line.strip_prefix("Exec=") {
                exec = Some(rest.trim().to_string());
            }
            if name.is_some() && exec.is_some() {
                break;
            }
        }

        let name = name?;
        let exec = exec?;
        let exe_name = extract_exe_name(&exec)?;
        Some((name, exe_name))
    }

    let mut apps = Vec::new();
    let mut seen = HashSet::new();
    let desktop_dirs = [
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
        dirs::home_dir()
            .map(|h| h.join(".local/share/applications"))
            .unwrap_or_else(|| PathBuf::from("/nonexistent")),
    ];

    for dir in &desktop_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("desktop") {
                    if let Some((app_name, exe_name)) = parse_desktop_file(&path) {
                        // dedupe by (app_name, exe_name)
                        if seen.insert((app_name.clone(), exe_name.clone())) {
                            apps.push((app_name, exe_name));
                        }
                    }
                }
            }
        }
    }

    // Sort by display name
    apps.sort_by(|a, b| a.0.cmp(&b.0));
    log::debug!("Found {} installed apps", apps.len());
    apps
}

