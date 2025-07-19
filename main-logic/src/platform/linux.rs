use std::process::Command;
use std::fs;
use crate::error::SynapseError;

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
