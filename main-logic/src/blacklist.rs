use std::collections::HashSet;
use std::fs;
use std::path::Path;
use serde::Deserialize;

#[derive(Debug)]
pub struct Blacklist {
    blocked: Vec<String>,
}

#[derive(Deserialize)]
struct BlacklistFile {
    blocked: Vec<String>,
}

impl Blacklist {
    pub fn new() -> Self {
        let path = Path::new("blacklist.json");

        let apps = if path.exists() {
            let contents = fs::read_to_string(path)
                .expect("Failed to read blacklist.json");

            let parsed: BlacklistFile = serde_json::from_str(&contents)
                .expect("blacklist.json has invalid format");

            parsed.blocked.into_iter().map(|s| s.to_lowercase()).collect()
        } else {
            println!("    blacklist.json not found - using default hardcoded blacklist.");
            let default = vec!["chrome.exe", "discord.exe", "vlc.exe"];
            default.into_iter().map(|s| s.to_string()).collect()
        };

        Blacklist { blocked: apps }
    }

    pub fn is_blocked(&self, process_name: &str) -> bool {
        self.blocked
            .iter()
            .any(|blocked_name| blocked_name.eq_ignore_ascii_case(process_name))
    }

    pub fn list(&self) -> &[String] {
        &self.blocked
    }
}
