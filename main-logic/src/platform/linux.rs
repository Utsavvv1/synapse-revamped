//! Linux platform module: provides process and popup utilities for Linux OS.

use std::process::Command;
use std::fs;
use std::collections::HashMap;
use crate::{error::SynapseError, api};

/// Raw probe of the foreground executable name (e.g. "code")
fn raw_foreground_exe_name() -> Result<Option<String>, SynapseError> {
    // 1️⃣ Get the active X11 window ID
    let out = Command::new("xprop")
        .arg("-root")
        .arg("_NET_ACTIVE_WINDOW")
        .output()
        .map_err(|e| SynapseError::Platform(format!("xprop failed: {}", e)))?
        .stdout;
    let s = String::from_utf8_lossy(&out);
    let id_opt = s.split_whitespace().last().map(str::to_string);
    let win = match id_opt {
        Some(ref x) if x != "0x0" => x.clone(),
        _ => return Ok(None),
    };

    // 2️⃣ Get its PID
    let pid_out = Command::new("xprop")
        .arg("-id").arg(&win)
        .arg("_NET_WM_PID")
        .output()
        .map_err(|e| SynapseError::Platform(format!("xprop failed: {}", e)))?
        .stdout;
    let s = String::from_utf8_lossy(&pid_out);
    let pid = s.split_whitespace().last()
                .and_then(|w| w.parse::<u32>().ok())
                .ok_or_else(|| SynapseError::Platform("No PID".into()))?;

    // 3️⃣ Read /proc/<pid>/comm
    let comm = fs::read_to_string(format!("/proc/{}/comm", pid))
        .map_err(|e| SynapseError::Platform(format!("Failed to read comm: {}", e)))?
        .trim()
        .to_lowercase();

    Ok(Some(comm))
}

/// Returns the *display* name of the currently‐focused app by matching
/// the raw exe against your installed‐apps list.
pub fn get_foreground_process_name() -> Result<Option<String>, SynapseError> {
    // Build exe → display map
    let mut map: HashMap<String, String> = HashMap::new();
    for (display, exe) in api::get_installed_apps_api() {
        map.insert(exe.to_lowercase(), display);
    }

    // Probe raw exe
    if let Some(raw) = raw_foreground_exe_name()? {
        // Exact match
        if let Some(display) = map.get(&raw) {
            return Ok(Some(display.clone()));
        }
        // Substring fallback
        for (exe, display) in &map {
            if exe.contains(&raw) || raw.contains(exe) {
                return Ok(Some(display.clone()));
            }
        }
    }
    Ok(None)
}

/// Lists all running process names on Linux.
///
/// # Errors
/// Returns `SynapseError` if the process list cannot be retrieved.
pub fn list_running_process_names() -> Result<Vec<String>, SynapseError> {
    let mut names = Vec::new();
    for entry in fs::read_dir("/proc")
        .map_err(|e| SynapseError::Platform(format!("read_dir failed: {}", e)))?
    {
        let entry = entry.map_err(|e| SynapseError::Platform(format!("entry failed: {}", e)))?;
        if let Some(pid) = entry.file_name().into_string().ok().and_then(|n| n.parse::<u32>().ok()) {
            if let Ok(c) = fs::read_to_string(format!("/proc/{}/comm", pid)) {
                names.push(c.trim().to_lowercase());
            }
        }
    }
    Ok(names)
}

/// Shows a popup warning for a distraction app on Linux.
///
/// # Arguments
/// * `app_name` - Name of the blocked app
pub fn show_distraction_popup(app_name: &str) -> Result<(), SynapseError> {
    let _ = Command::new("notify-send")
        .arg("Distraction Detected!")
        .arg(format!("You opened a blocked app: {}", app_name))
        .output();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn test_get_foreground_process_name() {
        // Ensure it doesn't panic; in real CI you'd mock xprop.
        let _ = get_foreground_process_name();
    }
}
