//! Windows platform module: provides process and popup utilities for Windows OS.

use windows::{
    core::PCSTR,
    Win32::System::Diagnostics::ToolHelp::*,
    Win32::UI::WindowsAndMessaging::*,
};
use std::ffi::{CStr, CString};
use std::collections::HashMap;
use crate::{error::SynapseError, api};

/// Raw probe of the foreground executable name (e.g. "code.exe" → "code")
fn raw_foreground_exe_name() -> Result<Option<String>, SynapseError> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == 0 {
            return Ok(None);
        }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return Ok(None);
        }
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
            .map_err(|e| SynapseError::Platform(format!("Snapshot failed: {:?}", e)))?;
        let mut entry = PROCESSENTRY32 {
            dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
            ..Default::default()
        };
        if Process32First(snapshot, &mut entry).is_ok() {
            loop {
                if entry.th32ProcessID == pid {
                    // szExeFile is a null-terminated CStr
                    let raw = entry.szExeFile.as_ptr() as *const i8;
                    let name = CStr::from_ptr(raw)
                        .to_string_lossy()
                        .into_owned()
                        .to_lowercase();
                    // strip any ".exe" suffix if you prefer
                    let name = name.strip_suffix(".exe").unwrap_or(&name).into();
                    return Ok(Some(name));
                }
                if Process32Next(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
    }
    Ok(None)
}

/// Returns the *display* name of the currently‐focused app by matching
/// the raw exe against your installed‐apps list.
pub fn get_foreground_process_name() -> Result<Option<String>, SynapseError> {
    // 1️⃣ Build a map exe_name → display_name
    let mut map: HashMap<String, String> = HashMap::new();
    for (display, exe) in api::get_installed_apps_api() {
        map.insert(exe.to_lowercase(), display);
    }

    // 2️⃣ Probe the raw exe
    if let Some(raw) = raw_foreground_exe_name()? {
        // 3️⃣ Try exact match
        if let Some(display) = map.get(&raw) {
            return Ok(Some(display.clone()));
        }
        // 4️⃣ Fallback: substring
        for (exe, display) in &map {
            if exe.contains(&raw) || raw.contains(exe) {
                return Ok(Some(display.clone()));
            }
        }
    }
    Ok(None)
}

/// Lists all running process names on Windows.
///
/// # Errors
/// Returns `SynapseError` if the process list cannot be retrieved.
pub fn list_running_process_names() -> Result<Vec<String>, SynapseError> {
    let mut names = Vec::new();
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
            .map_err(|e| SynapseError::Platform(format!("Snapshot failed: {:?}", e)))?;
        let mut entry = PROCESSENTRY32 {
            dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
            ..Default::default()
        };
        if Process32First(snapshot, &mut entry).is_ok() {
            loop {
                let raw = entry.szExeFile.as_ptr() as *const i8;
                let name = CStr::from_ptr(raw)
                    .to_string_lossy()
                    .into_owned()
                    .to_lowercase();
                names.push(name);
                if Process32Next(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
    }
    Ok(names)
}

/// Shows a popup warning for a distraction app on Windows.
///
/// # Arguments
/// * `app_name` - Name of the blocked app
pub fn show_distraction_popup(app_name: &str) -> Result<(), SynapseError> {
    unsafe {
        let title = CString::new("Distraction Detected!")
            .map_err(|e| SynapseError::Platform(format!("CString failed: {}", e)))?;
        let message = CString::new(format!("You opened a blocked app: {}", app_name))
            .map_err(|e| SynapseError::Platform(format!("CString failed: {}", e)))?;
        MessageBoxA(
            None,
            PCSTR(message.as_ptr() as *const u8),
            PCSTR(title.as_ptr() as *const u8),
            MB_OK | MB_ICONWARNING | MB_TOPMOST,
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_get_foreground_process_name() {
        // We just ensure it doesn't panic; real CI should mock Win32.
        let _ = get_foreground_process_name();
    }
}
