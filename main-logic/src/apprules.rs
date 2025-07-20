//! Application rules module: handles loading, parsing, and checking whitelist/blacklist rules for process names.

use std::fs;
use std::path::Path;
use serde::Deserialize;
use serde_json;
use crate::error::SynapseError;

/// Structure for deserializing the application rules JSON file.
#[derive(Debug, Deserialize, Clone)]
pub struct AppRulesFile {
    /// List of whitelisted process names.
    pub whitelist: Vec<String>,
    /// List of blacklisted process names.
    pub blacklist: Vec<String>,
}

/// Application rules for process whitelisting and blacklisting.
#[derive(Clone)]
pub struct AppRules {
    /// Whitelisted process names (expanded for platform).
    pub whitelist: Vec<String>,
    /// Blacklisted process names (expanded for platform).
    pub blacklist: Vec<String>,
}

impl AppRules {
    /// Loads application rules from `apprules.json` if present, or uses empty rules otherwise.
    ///
    /// # Errors
    /// Returns `SynapseError` if the file cannot be read or parsed.
    pub fn new() -> Result<Self, SynapseError> {
        let path = Path::new("apprules.json");
        if path.exists() {
            let contents = fs::read_to_string(path)
                .map_err(|e| SynapseError::Config(format!("Failed to read apprules.json: {}", e)))?;
            let parsed: AppRulesFile = serde_json::from_str(&contents)
                .map_err(|e| SynapseError::Config(format!("Failed to parse apprules.json: {}", e)))?;
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

    /// Construct AppRules directly from whitelist and blacklist (for tests and integration).
    pub fn test_with_rules(whitelist: Vec<String>, blacklist: Vec<String>) -> Self {
        AppRules {
            whitelist: Self::expand_names(whitelist),
            blacklist: Self::expand_names(blacklist),
        }
    }

    /// Expands process names for platform-specific matching (e.g., adds `.exe` on Windows).
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

    /// Checks if a process name is in the whitelist.
    pub fn is_work_app(&self, process_name: &str) -> bool {
        self.whitelist.iter().any(|name| name.eq_ignore_ascii_case(process_name))
    }

    /// Checks if a process name is in the blacklist.
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
        let rules = AppRules::test_with_rules(vec!["notepad.exe".to_string()], vec!["chrome.exe".to_string()]);
        assert!(rules.is_work_app("Notepad.exe"));
        assert!(rules.is_work_app("NOTEPAD.EXE"));
        assert!(!rules.is_work_app("chrome.exe"));
    }

    #[test]
    fn checks_blacklist_case_insensitive() {
        let rules = AppRules::test_with_rules(vec!["notepad.exe".to_string()], vec!["chrome.exe".to_string()]);
        assert!(rules.is_blocked("chrome.exe"));
        assert!(rules.is_blocked("CHROME.EXE"));
        assert!(!rules.is_blocked("notepad.exe"));
    }

    #[test]
    fn missing_file_leaves_whitelist_and_blacklist_empty() {
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
        assert!(rules.blacklist.is_empty());
        // Restore apprules.json if it was present
        if had_file {
            let _ = fs::rename(backup, path);
        }
    }

    #[test]
    fn expand_names_adds_exe_on_windows() {
        let names = vec!["notepad".to_string()];
        let expanded = AppRules::test_with_rules(names.clone(), vec![]).whitelist;
        #[cfg(target_os = "windows")]
        assert!(expanded.contains(&"notepad.exe".to_string()));
        assert!(expanded.contains(&"notepad".to_string()));
        #[cfg(not(target_os = "windows"))]
        assert!(expanded.contains(&"notepad".to_string()));
    }

    #[test]
    fn handles_empty_lists() {
        let rules = AppRules::test_with_rules(vec![], vec![]);
        assert!(!rules.is_work_app("anything.exe"));
        assert!(!rules.is_blocked("anything.exe"));
    }

    #[test]
    fn handles_malformed_json() {
        let path = Path::new("test_apprules_bad.json");
        fs::write(path, "not a json").unwrap();
        let result = fs::read_to_string(path).and_then(|contents| serde_json::from_str::<AppRulesFile>(&contents).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)));
        assert!(result.is_err());
        fs::remove_file(path).unwrap();
    }
}
