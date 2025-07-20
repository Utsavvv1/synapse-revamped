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
            println!("    apprules.json not found - using empty rules.");
            let whitelist: Vec<String> = Vec::new();
            let blacklist: Vec<String> = Vec::new();
            Ok(AppRules {
                whitelist: Self::expand_names(whitelist),
                blacklist: Self::expand_names(blacklist),
            })
        }
    }

    /// Test-only: construct AppRules directly from whitelist and blacklist.
    #[cfg(test)]
    pub fn test_with_rules(whitelist: Vec<String>, blacklist: Vec<String>) -> Self {
        AppRules {
            whitelist: Self::expand_names(whitelist),
            blacklist: Self::expand_names(blacklist),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn loads_valid_json() {
        let json = r#"{"whitelist": ["notepad.exe"], "blacklist": ["chrome.exe"]}"#;
        fs::write("test_apprules.json", json).unwrap();
        let path = Path::new("test_apprules.json");
        let contents = fs::read_to_string(path).unwrap();
        let parsed: AppRulesFile = serde_json::from_str(&contents).unwrap();
        assert_eq!(parsed.whitelist, vec!["notepad.exe"]);
        assert_eq!(parsed.blacklist, vec!["chrome.exe"]);
        fs::remove_file(path).unwrap();
    }


    

    #[test]
    fn checks_whitelist_case_insensitive() {
        let rules = AppRules::new().unwrap();
        assert!(rules.is_work_app("Notepad.exe"));
        assert!(rules.is_work_app("NOTEPAD.EXE"));
    }

    #[test]
    fn missing_file_leaves_whitelist_empty() {
        let path = Path::new("apprules.json");
        let backup = Path::new("apprules.json.bak_test");
        // If apprules.json exists, rename it
        let had_file = if path.exists() {
            fs::rename(path, backup).is_ok()
        } else {
            false
        };
        let rules = AppRules::new().unwrap();
        assert!(rules.whitelist.is_empty());
        // Restore apprules.json if it was present
        if had_file {
            let _ = fs::rename(backup, path);
        }
    }
}
