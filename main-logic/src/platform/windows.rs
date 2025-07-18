use windows::{
    core::PCSTR,
    Win32::Foundation::*,
    Win32::System::Diagnostics::ToolHelp::*,
    Win32::UI::WindowsAndMessaging::*,
};

use std::ffi::{CStr, CString};

pub fn get_foreground_process_name() -> Option<String> {
    unsafe {
        // get foreground window handle
        let hwnd = GetForegroundWindow();
        if hwnd.0 == 0 {
            return None;
        }

        // get PID from window handle
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }

        // create snapshot of all processes
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
        let mut entry = PROCESSENTRY32 {
            dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
            ..Default::default()
        };

        // iterate all processes
        if Process32First(snapshot, &mut entry).is_ok() {
            loop {
                if entry.th32ProcessID == pid {
                    // szExeFile is a null-terminated C string
                    let raw_name = entry.szExeFile.as_ptr();
                    let name = CStr::from_ptr(raw_name as *const i8)
                        .to_string_lossy()
                        .into_owned();
                    return Some(name);
                }

                if Process32Next(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
    }
    None
}

pub fn list_running_process_names() -> Vec<String> {
    let mut names = Vec::new();
    unsafe {
        let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(s) => s,
            Err(_) => return names,
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
                    .into_owned();
                names.push(name);
                if Process32Next(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
    }
    names
}

pub fn show_distraction_popup(app_name: &str) {
    unsafe {
        let title = CString::new("Distraction Detected!").unwrap();
        let message = CString::new(format!("You Opened a Blocked App: {}", app_name)).unwrap();
        MessageBoxA(
            None,
            PCSTR(message.as_ptr() as *const u8),
            PCSTR(title.as_ptr() as *const u8),
            MB_OK | MB_ICONWARNING | MB_TOPMOST,
        );
    }
}
