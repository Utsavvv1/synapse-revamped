use std::fs;
use std::path::Path;
use serde::Deserialize;
use serde_json;

#[derive(Debug, Deserialize, Clone)]
pub struct AppRulesFile {
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
}

#[derive(Clone)]
pub struct AppRules {
    whitelist: Vec<String>,
    blacklist: Vec<String>,
}

impl AppRules {
    pub fn new() -> Self {
        let path = Path::new("apprules.json");
        if path.exists() {
            let contents = fs::read_to_string(path)
                .expect("Failed to read apprules.json");
            let parsed: AppRulesFile = serde_json::from_str(&contents)
                .expect("apprules.json has invalid format");
            AppRules {
                whitelist: parsed.whitelist.into_iter().map(|s| s.to_lowercase()).collect(),
                blacklist: parsed.blacklist.into_iter().map(|s| s.to_lowercase()).collect(),
            }
        } else {
            println!("    apprules.json not found - using default rules.");
            let whitelist = vec!["code.exe", "notepad.exe", "cursor.exe", "windowsterminal.exe"];
            let blacklist = vec!["chrome.exe", "discord.exe", "vlc.exe", "spotify.exe"];
            AppRules {
                whitelist: whitelist.into_iter().map(|s| s.to_lowercase()).collect(),
                blacklist: blacklist.into_iter().map(|s| s.to_lowercase()).collect(),
            }
        }
    }

    pub fn is_work_app(&self, process_name: &str) -> bool {
        self.whitelist.iter().any(|name| name.eq_ignore_ascii_case(process_name))
    }

    pub fn is_blocked(&self, process_name: &str) -> bool {
        self.blacklist.iter().any(|name| name.eq_ignore_ascii_case(process_name))
    }

    pub fn list_whitelist(&self) -> &[String] {
        &self.whitelist
    }
    pub fn list_blacklist(&self) -> &[String] {
        &self.blacklist
    }
}
