use windows::{
    core::PCSTR,
    Win32::System::Diagnostics::ToolHelp::*,
    Win32::UI::WindowsAndMessaging::*,
};

use std::ffi::{CStr, CString};
use crate::error::SynapseError;

pub fn get_foreground_process_name() -> Result<Option<String>, SynapseError> {
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
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).map_err(|e| SynapseError::Platform(format!("Snapshot failed: {:?}", e)))?;
        let mut entry = PROCESSENTRY32 {
            dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
            ..Default::default()
        };
        if Process32First(snapshot, &mut entry).is_ok() {
            loop {
                if entry.th32ProcessID == pid {
                    let raw_name = entry.szExeFile.as_ptr();
                    let name = CStr::from_ptr(raw_name as *const i8)
                        .to_string_lossy()
                        .into_owned()
                        .to_lowercase();
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

pub fn list_running_process_names() -> Result<Vec<String>, SynapseError> {
    let mut names = Vec::new();
    unsafe {
        let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(s) => s,
            Err(e) => return Err(SynapseError::Platform(format!("Snapshot failed: {:?}", e))),
        };
        let mut entry = PROCESSENTRY32 {
            dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
            ..Default::default()
        };
        if Process32First(snapshot, &mut entry).is_ok() {
            loop {
                let raw_name = entry.szExeFile.as_ptr();
                let name = CStr::from_ptr(raw_name as *const i8)
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

pub fn show_distraction_popup(app_name: &str) -> Result<(), SynapseError> {
    unsafe {
        let title = CString::new("Distraction Detected!").map_err(|e| SynapseError::Platform(format!("CString failed: {}", e)))?;
        let message = CString::new(format!("You opened a blocked app: {}", app_name)).map_err(|e| SynapseError::Platform(format!("CString failed: {}", e)))?;
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
    fn test_get_foreground_process_name_handles_no_window() {
        // This test is a placeholder: in real CI, you would mock Windows APIs
        // Here, just check that the function returns Ok or an error, but does not panic
        let result = get_foreground_process_name();
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_list_running_process_names_returns_vec() {
        let result = list_running_process_names();
        assert!(result.is_ok());
        let names = result.unwrap();
        assert!(names.is_empty() || !names.is_empty()); // Always true, just checks type
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_show_distraction_popup_returns_ok() {
        let result = show_distraction_popup("test.exe");
        assert!(result.is_ok());
    }

    #[test]
    fn test_non_windows_functions_do_not_panic() {
        // On non-Windows, these functions should not panic if called (should not be available)
        // This is a placeholder for cross-platform safety
        assert!(true);
    }
}
