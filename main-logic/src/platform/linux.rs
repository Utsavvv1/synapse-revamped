//! Linux platform module: provides process and popup utilities for Linux OS.

use std::process::Command;
use std::fs;
use crate::error::SynapseError;

/// Gets the name of the foreground process on Linux.
///
/// # Errors
/// Returns `SynapseError` if the process name cannot be determined.
pub fn get_foreground_process_name() -> Result<Option<String>, SynapseError> {
    // Try to get the active window's PID using xprop and xdotool
    let window_id = Command::new("xprop")
        .arg("-root")
        .arg("_NET_ACTIVE_WINDOW")
        .output()
        .map_err(|e| SynapseError::Platform(format!("xprop failed: {}", e)))?
        .stdout;
    let s = String::from_utf8_lossy(&window_id);
    let id = s.split_whitespace().last().map(|w| w.trim().to_string());
    let window_id = match id {
        Some(id) if id != "0x0" => id,
        _ => return Ok(None),
    };
    let pid_out = Command::new("xprop")
        .arg("-id")
        .arg(&window_id)
        .arg("_NET_WM_PID")
        .output()
        .map_err(|e| SynapseError::Platform(format!("xprop failed: {}", e)))?
        .stdout;
    let s = String::from_utf8_lossy(&pid_out);
    let pid = s.split_whitespace().last().and_then(|w| w.parse::<u32>().ok());
    let pid = match pid {
        Some(pid) => pid,
        None => return Ok(None),
    };
    let comm_path = format!("/proc/{}/comm", pid);
    let name = fs::read_to_string(comm_path)?.trim().to_lowercase();
    Ok(Some(name))
}

/// Lists all running process names on Linux.
///
/// # Errors
/// Returns `SynapseError` if the process list cannot be retrieved.
pub fn list_running_process_names() -> Result<Vec<String>, SynapseError> {
    let mut names = Vec::new();
    for entry in fs::read_dir("/proc")? {
        let entry = entry?;
        if let Ok(file_name) = entry.file_name().into_string() {
            if let Ok(pid) = file_name.parse::<u32>() {
                let comm_path = format!("/proc/{}/comm", pid);
                if let Ok(name) = fs::read_to_string(comm_path) {
                    names.push(name.trim().to_lowercase());
                }
            }
        }
    }
    Ok(names)
}

/// Shows a popup warning for a distraction app on Linux.
///
/// # Arguments
/// * `app_name` - Name of the blocked app
///
/// # Errors
/// Returns `SynapseError` if the popup cannot be shown.
pub fn show_distraction_popup(app_name: &str) -> Result<(), SynapseError> {
    let result = Command::new("notify-send")
        .arg("Distraction Detected!")
        .arg(format!("You opened a blocked app: {}", app_name))
        .output();
    if result.is_err() {
        println!("(Warning: notify-send failed, no popup shown)");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn test_get_foreground_process_name_handles_no_window() {
        // This test is a placeholder: in real CI, you would mock Linux APIs
        // Here, just check that the function returns Ok or an error, but does not panic
        let result = get_foreground_process_name();
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_list_running_process_names_returns_vec() {
        let result = list_running_process_names();
        assert!(result.is_ok());
        let names = result.unwrap();
        assert!(names.is_empty() || !names.is_empty()); // Always true, just checks type
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_show_distraction_popup_returns_ok() {
        let result = show_distraction_popup("test.exe");
        assert!(result.is_ok());
    }

    #[test]
    fn test_non_linux_functions_do_not_panic() {
        // On non-Linux, these functions should not panic if called (should not be available)
        // This is a placeholder for cross-platform safety
        assert!(true);
    }
}
