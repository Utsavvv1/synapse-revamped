use std::fs;
use std::path::Path;
use serde::Deserialize;
use serde_json;
use crate::error::SynapseError;

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
    pub fn new() -> Result<Self, SynapseError> {
        let path = Path::new("apprules.json");
        if path.exists() {
            let contents = fs::read_to_string(path)?;
            let parsed: AppRulesFile = serde_json::from_str(&contents)?;
            Ok(AppRules {
                whitelist: Self::expand_names(parsed.whitelist),
                blacklist: Self::expand_names(parsed.blacklist),
            })
        } else {
            println!("    apprules.json not found - using default rules.");
            let whitelist = vec!["code", "notepad", "cursor", "windowsterminal"]
                .into_iter().map(|s| s.to_string()).collect();
            let blacklist = vec!["chrome", "discord", "vlc", "spotify"]
                .into_iter().map(|s| s.to_string()).collect();
            Ok(AppRules {
                whitelist: Self::expand_names(whitelist),
                blacklist: Self::expand_names(blacklist),
            })
        }
    }

    fn expand_names(names: Vec<String>) -> Vec<String> {
        let mut expanded = Vec::new();
        for name in names {
            let name_lc = name.to_lowercase();
            expanded.push(name_lc.clone());
            #[cfg(target_os = "windows")]
            {
                if !name_lc.ends_with(".exe") {
                    expanded.push(format!("{}.exe", name_lc));
                }
            }
        }
        expanded
    }

    pub fn is_work_app(&self, process_name: &str) -> bool {
        self.whitelist.iter().any(|name| name.eq_ignore_ascii_case(process_name))
    }

    pub fn is_blocked(&self, process_name: &str) -> bool {
        self.blacklist.iter().any(|name| name.eq_ignore_ascii_case(process_name))
    }

    // pub fn list_whitelist(&self) -> &[String] {
    //     &self.whitelist
    // }
    // pub fn list_blacklist(&self) -> &[String] {
    //     &self.blacklist
    // }
}
