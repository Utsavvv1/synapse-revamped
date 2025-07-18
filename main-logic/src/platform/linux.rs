use std::process::Command;
use std::fs;

pub fn get_foreground_process_name() -> Option<String> {
    // Try to get the active window's PID using xprop and xdotool
    // This will only work if X11 tools are available
    let window_id = Command::new("xprop")
        .arg("-root")
        .arg("_NET_ACTIVE_WINDOW")
        .output()
        .ok()
        .and_then(|out| {
            let s = String::from_utf8_lossy(&out.stdout);
            s.split_whitespace().last().map(|w| w.trim().to_string())
        })?;
    if window_id == "0x0" {
        return None;
    }
    let pid = Command::new("xprop")
        .arg("-id")
        .arg(&window_id)
        .arg("_NET_WM_PID")
        .output()
        .ok()
        .and_then(|out| {
            let s = String::from_utf8_lossy(&out.stdout);
            s.split_whitespace().last()?.parse::<u32>().ok()
        })?;
    // Read /proc/<pid>/comm for the process name
    let comm_path = format!("/proc/{}/comm", pid);
    fs::read_to_string(comm_path).ok().map(|s| s.trim().to_lowercase())
}

pub fn list_running_process_names() -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if let Ok(pid) = file_name.parse::<u32>() {
                    let comm_path = format!("/proc/{}/comm", pid);
                    if let Ok(name) = fs::read_to_string(comm_path) {
                        names.push(name.trim().to_lowercase());
                    }
                }
            }
        }
    }
    names
}

pub fn show_distraction_popup(app_name: &str) {
    let result = Command::new("notify-send")
        .arg("Distraction Detected!")
        .arg(format!("You opened a blocked app: {}", app_name))
        .output();
    if result.is_err() {
        println!("(Warning: notify-send failed, no popup shown)");
    }
}
