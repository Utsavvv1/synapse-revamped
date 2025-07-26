//! Application rules module: handles loading, parsing, and checking whitelist/blacklist rules for process names.

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_json;
use crate::error::SynapseError;

/// Structure for deserializing the application rules JSON file.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppRulesFile {
    whitelist: Vec<String>,
    blacklist: Vec<String>,
}

/// Application rules for process whitelisting and blacklisting.
#[derive(Clone)]
pub struct AppRules {
    whitelist: Vec<String>,
    blacklist: Vec<String>,
}

impl AppRules {
    /// Loads application rules from `apprules.json` if present, or uses empty rules otherwise.
    ///
    /// # Errors
    /// Returns `SynapseError` if the file cannot be read or parsed.
    pub fn new() -> Result<Self, SynapseError> {
        let path_str = std::env::var("APPRULES_PATH").unwrap_or_else(|_| "apprules.json".to_string());
        let path = Path::new(&path_str);
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
                if !name_lc.ends_with(".exe") && !name_lc.contains(".exe") {
                    expanded.push(format!("{}.exe", name_lc));
                }
            }
        }
        expanded
    }

    /// Updates the whitelist and blacklist, then saves to `apprules.json`.
    ///
    /// # Arguments
    /// * `whitelist` - New whitelist apps (expected as .exe names from frontend).
    /// * `blacklist` - New blacklist apps (expected as .exe names from frontend).
    ///
    /// # Errors
    /// Returns `SynapseError` if the file cannot be written or serialized.
    pub fn update_rules(&mut self, whitelist: Vec<String>, blacklist: Vec<String>) -> Result<(), SynapseError> {
        log::info!("[DEBUG] update_rules called");
        log::info!("[DEBUG] Incoming whitelist: {:?}", whitelist);
        log::info!("[DEBUG] Incoming blacklist: {:?}", blacklist);

        self.whitelist = Self::expand_names(whitelist); // Expand .exe names if needed
        self.blacklist = Self::expand_names(blacklist); // Expand .exe names if needed

        log::info!("[DEBUG] Expanded whitelist: {:?}", self.whitelist);
        log::info!("[DEBUG] Expanded blacklist: {:?}", self.blacklist);

        let rules = AppRulesFile {
            whitelist: self.whitelist.iter().map(|s| s.to_string()).collect(),
            blacklist: self.blacklist.iter().map(|s| s.to_string()).collect(),
        };

        let json = serde_json::to_string_pretty(&rules)
            .map_err(|e| {
                log::error!("[DEBUG] Failed to serialize app rules: {}", e);
                SynapseError::Config(format!("Failed to serialize app rules: {}", e))
            })?;
        let path_str = std::env::var("APPRULES_PATH").unwrap_or_else(|_| "apprules.json".to_string());
        let path = Path::new(&path_str);

        log::info!("[DEBUG] Writing rules to: {}", path.display());
        fs::write(path, json)
            .map_err(|e| {
                log::error!("[DEBUG] Failed to write apprules.json: {}", e);
                SynapseError::Config(format!("Failed to write apprules.json: {}", e))
            })?;

        log::info!("[DEBUG] App rules successfully updated and written to disk.");

        Ok(())
    }

    /// Checks if a process name is in the whitelist.
    pub fn is_work_app(&self, process_name: &str) -> bool {
        self.whitelist.iter().any(|name| name.eq_ignore_ascii_case(process_name))
    }

    /// Checks if a process name is in the blacklist.
    pub fn is_blocked(&self, process_name: &str) -> bool {
        self.blacklist.iter().any(|name| name.eq_ignore_ascii_case(process_name))
    }

    /// Returns a reference to the whitelist.
    pub fn whitelist(&self) -> &Vec<String> {
        &self.whitelist
    }

    /// Returns a reference to the blacklist.
    pub fn blacklist(&self) -> &Vec<String> {
        &self.blacklist
    }
}

/// Public function to update apprules.json without managing state in src-tauri.
pub fn update_app_rules(whitelist: Vec<String>, blacklist: Vec<String>) -> Result<(), SynapseError> {
    log::info!("Updating app rules:");
    log::info!("  New whitelist: {:?}", whitelist);
    log::info!("  New blacklist: {:?}", blacklist);

    // Load existing rules, update them, and save
    let mut rules = AppRules::new()?;
    rules.update_rules(whitelist, blacklist)?;
    log::info!("App rules updated and saved to apprules.json.");

    // Print the updated rules for debug
    log::info!("  Updated whitelist: {:?}", rules.whitelist());
    log::info!("  Updated blacklist: {:?}", rules.blacklist());

    Ok(())
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
        let had_file = if path.exists() {
            fs::rename(path, backup).is_ok()
        } else {
            false
        };
        let rules = AppRules::new().unwrap();
        assert!(rules.whitelist().is_empty());
        assert!(rules.blacklist().is_empty());
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

    #[test]
    fn updates_and_saves_rules_with_exe() {
        let mut rules = AppRules::test_with_rules(vec!["notepad".to_string()], vec!["chrome".to_string()]);
        let new_whitelist = vec!["emacs.exe".to_string()];
        let new_blacklist = vec!["discord.exe".to_string()];
        rules.update_rules(new_whitelist, new_blacklist).unwrap();

        let path = Path::new("test_apprules.json");
        let contents = fs::read_to_string(path).unwrap();
        let parsed: AppRulesFile = serde_json::from_str(&contents).unwrap();
        assert_eq!(parsed.whitelist, vec!["emacs.exe"]);
        assert_eq!(parsed.blacklist, vec!["discord.exe"]);
        fs::remove_file(path).unwrap();
    }
}