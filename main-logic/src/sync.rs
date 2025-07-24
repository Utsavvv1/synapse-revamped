use crate::session::FocusSession;
use reqwest::Client;
use serde_json;
use dotenvy::dotenv;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use crate::error::SupabaseError;
use crate::types::AppUsageEvent;

/// Supabase sync client module
#[derive(Clone)]
pub struct SupabaseSync {
    pub client: Client,
    pub api_key: String,
    pub base_url: String,
}

impl SupabaseSync {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
        }
    }

    /// Initialize SupabaseSync from environment variables (.env)
    pub fn from_env(skip_dotenv: bool) -> Result<Self, SupabaseError> {
        if !skip_dotenv {
            dotenv().ok();
        }
        let api_key = env::var("SUPABASE_API_KEY").map_err(|_| SupabaseError::Config("SUPABASE_API_KEY not set".to_string()))?;
        let base_url = env::var("SUPABASE_URL").map_err(|_| SupabaseError::Config("SUPABASE_URL not set".to_string()))?;
        Ok(Self::new(api_key, base_url))
    }

    /// Push a focus session to Supabase
    pub async fn push_focus_session(&self, session: &FocusSession) -> Result<(), SupabaseError> {
        let url = format!("{}/focus_sessions", self.base_url.trim_end_matches('/'));
        let resp = self.client.post(&url)
            .header("apikey", &self.api_key)
            .header("Content-Type", "application/json")
            .json(session)
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(SupabaseError::Api(format!("Supabase sync failed: {} - {}", status, body)))
        }
    }

    pub async fn push_app_usage_events(&self, events: &[AppUsageEvent]) -> Result<(), SupabaseError> {
        // Debug: print the events being sent
        println!("[DEBUG] Sending app_usage_events to Supabase: {}", serde_json::to_string_pretty(&events).unwrap_or_else(|_| "<serialization error>".to_string()));
        let url = format!("{}/app_usage_events", self.base_url.trim_end_matches('/'));
        let resp = self.client.post(&url)
            .header("apikey", &self.api_key)
            .header("Content-Type", "application/json")
            .json(events)
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(SupabaseError::Api(format!("Supabase sync failed: {} - {}", status, body)))
        }
    }
}

/// Tracks the status of the last sync attempt
#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub last_sync_time: Option<SystemTime>,
    pub last_result: Option<bool>, // true = success, false = failure
    pub last_error: Option<String>,
}

impl SyncStatus {
    pub fn new() -> Self {
        Self {
            last_sync_time: None,
            last_result: None,
            last_error: None,
        }
    }

    pub fn update(&mut self, success: bool, error: Option<String>) {
        self.last_sync_time = Some(SystemTime::now());
        self.last_result = Some(success);
        self.last_error = error;
    }
}

/// Example: Shared sync status for the app
pub type SharedSyncStatus = Arc<Mutex<SyncStatus>>;

impl SupabaseSync {
    /// Push a focus session to Supabase and update sync status if provided
    pub async fn push_focus_session_with_status(&self, session: &FocusSession, status: Option<&SharedSyncStatus>) -> Result<(), SupabaseError> {
        let url = format!("{}/focus_sessions", self.base_url.trim_end_matches('/'));
        let resp = self.client.post(&url)
            .header("apikey", &self.api_key)
            .header("Content-Type", "application/json")
            .json(session)
            .send()
            .await;
        match resp {
            Ok(resp) => {
                if resp.status().is_success() {
                    if let Some(shared) = status {
                        let mut s = shared.lock().unwrap();
                        s.update(true, None);
                    }
                    Ok(())
                } else {
                    let status_code = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    let err = format!("Supabase sync failed: {} - {}", status_code, body);
                    if let Some(shared) = status {
                        let mut s = shared.lock().unwrap();
                        s.update(false, Some(err.clone()));
                    }
                    Err(SupabaseError::Api(err))
                }
            }
            Err(e) => {
                if let Some(shared) = status {
                    let mut s = shared.lock().unwrap();
                    s.update(false, Some(e.to_string()));
                }
                Err(SupabaseError::Http(e))
            }
        }
    }

    /// Pull all focus sessions from Supabase
    pub async fn pull_focus_sessions(&self) -> Result<Vec<FocusSession>, SupabaseError> {
        let url = format!("{}/focus_sessions", self.base_url.trim_end_matches('/'));
        let resp = self.client.get(&url)
            .header("apikey", &self.api_key)
            .header("Accept", "application/json")
            .send()
            .await?;
        if resp.status().is_success() {
            let sessions: Vec<FocusSession> = resp.json().await?;
            Ok(sessions)
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(SupabaseError::Api(format!("Supabase pull failed: {} - {}", status, body)))
        }
    }
}

/// Merge local and remote sessions using last-write-wins on start_time.
pub fn merge_sessions(local: Vec<FocusSession>, remote: Vec<FocusSession>) -> Vec<FocusSession> {
    // Key: (start_time as u64, work_apps joined)
    fn session_key(s: &FocusSession) -> (u64, String) {
        let start = s.start_time().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let apps = s.work_apps().join(",");
        (start, apps)
    }
    let mut map: HashMap<(u64, String), FocusSession> = HashMap::new();
    for s in local.into_iter() {
        map.insert(session_key(&s), s);
    }
    for s in remote.into_iter() {
        let key = session_key(&s);
        // If remote is newer or not present, use remote
        match map.get(&key) {
            Some(existing) => {
                let remote_time = s.start_time().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
                let local_time = existing.start_time().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
                if remote_time >= local_time {
                    map.insert(key, s);
                }
            }
            None => {
                map.insert(key, s);
            }
        }
    }
    map.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_from_env_missing_api_key_and_url() {
        use std::env;
        // Save original values to restore later
        let orig_api_key = env::var("SUPABASE_API_KEY").ok();
        let orig_url = env::var("SUPABASE_URL").ok();

        // Test missing API key
        env::remove_var("SUPABASE_API_KEY");
        env::set_var("SUPABASE_URL", "http://example.com");
        let result = SupabaseSync::from_env(true);
        assert!(matches!(result, Err(SupabaseError::Config(_))));
        if let Err(SupabaseError::Config(msg)) = result {
            assert!(msg.contains("SUPABASE_API_KEY"));
        }

        // Test missing URL
        env::set_var("SUPABASE_API_KEY", "dummy");
        env::remove_var("SUPABASE_URL");
        let result = SupabaseSync::from_env(true);
        assert!(matches!(result, Err(SupabaseError::Config(_))));
        if let Err(SupabaseError::Config(msg)) = result {
            assert!(msg.contains("SUPABASE_URL"));
        }

        // Restore original values
        if let Some(val) = orig_api_key {
            env::set_var("SUPABASE_API_KEY", val);
        } else {
            env::remove_var("SUPABASE_API_KEY");
        }
        if let Some(val) = orig_url {
            env::set_var("SUPABASE_URL", val);
        } else {
            env::remove_var("SUPABASE_URL");
        }
    }

    #[test]
    fn test_supabase_error_variants() {
        let err = SupabaseError::Config("bad config".to_string());
        assert_eq!(format!("{}", err), "Configuration error: bad config");
        let err = SupabaseError::Api("api error".to_string());
        assert_eq!(format!("{}", err), "API error: api error");
        let err = SupabaseError::Other("other error".to_string());
        assert_eq!(format!("{}", err), "Other error: other error");
    }

    #[test]
    fn test_merge_sessions_basic() {
        use std::time::{SystemTime, Duration};
        let now = SystemTime::now();
        let s1 = FocusSession::new(now, vec!["a.exe".to_string()]);
        let s2 = FocusSession::new(now + Duration::from_secs(1), vec!["b.exe".to_string()]);
        let s3 = FocusSession::new(now, vec!["a.exe".to_string()]); // duplicate of s1
        let merged = merge_sessions(vec![s1.clone(), s2.clone()], vec![s3.clone()]);
        assert_eq!(merged.len(), 2);
        assert!(merged.iter().any(|s| s.work_apps() == &vec!["a.exe".to_string()]));
        assert!(merged.iter().any(|s| s.work_apps() == &vec!["b.exe".to_string()]));
    }

    #[test]
    fn test_merge_sessions_last_write_wins() {
        use std::time::{SystemTime, Duration};
        let now = SystemTime::now();
        let mut s1 = FocusSession::new(now, vec!["a.exe".to_string()]);
        let mut s2 = s1.clone();
        for _ in 0..5 {
            s2.increment_distraction_attempts();
        }
        let merged = merge_sessions(vec![s1.clone()], vec![s2.clone()]);
        // Should keep s2 (remote, same key, but last-write-wins)
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].distraction_attempts(), 5);
    }

    // Helper for tests to create a FocusSession with custom fields
    fn make_focus_session(
        start_time: std::time::SystemTime,
        end_time: Option<std::time::SystemTime>,
        work_apps: Vec<String>,
        distraction_attempts: u32,
    ) -> FocusSession {
        let mut session = FocusSession::new(start_time, work_apps);
        if let Some(et) = end_time {
            // Unsafe: for test only, use std::mem::transmute or use a public setter if available
            // Instead, use clone and set via public API if possible
            // But since there is no setter, use the default constructor and then set via struct update syntax if the field is pub(crate) in tests
            // If not possible, skip setting end_time/distraction_attempts in this test
        }
        // For now, only test with default values due to privacy
        session
    }

    #[test]
    fn test_focus_session_serialization_roundtrip() {
        use std::time::{SystemTime, Duration};
        let now = SystemTime::now();
        let session = FocusSession::new(now, vec!["notepad.exe".to_string(), "word.exe".to_string()]);
        let json = serde_json::to_string(&session).unwrap();
        let deserialized: FocusSession = serde_json::from_str(&json).unwrap();
        assert_eq!(session.work_apps(), deserialized.work_apps());
        assert_eq!(session.distraction_attempts(), deserialized.distraction_attempts());
        assert_eq!(session.end_time().is_some(), deserialized.end_time().is_some());
    }

    #[test]
    fn test_focus_session_deserialization_error() {
        // Missing required field (start_time)
        let json = r#"{"work_apps": ["a.exe"], "distraction_attempts": 1}"#;
        let result = serde_json::from_str::<FocusSession>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_supabase_error_propagation() {
        // Simulate an API error
        let err = SupabaseError::Api("api fail".to_string());
        let result: Result<(), SupabaseError> = Err(err);
        let synapse_result: Result<(), crate::error::SynapseError> = result.map_err(crate::error::SynapseError::from);
        assert!(matches!(synapse_result, Err(crate::error::SynapseError::Supabase(_))));
    }
} 