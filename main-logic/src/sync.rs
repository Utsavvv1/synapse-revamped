use crate::session::FocusSession;
use reqwest::Client;
use serde_json;
use dotenvy::dotenv;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

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
    pub fn from_env() -> Result<Self, String> {
        dotenv().ok();
        let api_key = env::var("SUPABASE_API_KEY").map_err(|_| "SUPABASE_API_KEY not set".to_string())?;
        let base_url = env::var("SUPABASE_URL").map_err(|_| "SUPABASE_URL not set".to_string())?;
        Ok(Self::new(api_key, base_url))
    }

    /// Push a focus session to Supabase
    pub async fn push_focus_session(&self, session: &FocusSession) -> Result<(), String> {
        let url = format!("{}/focus_sessions", self.base_url.trim_end_matches('/'));
        let resp = self.client.post(&url)
            .header("apikey", &self.api_key)
            .header("Content-Type", "application/json")
            .json(session)
            .send()
            .await
            .map_err(|e| format!("Request error: {}", e))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(format!("Supabase sync failed: {} - {}", status, body))
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
    pub async fn push_focus_session_with_status(&self, session: &FocusSession, status: Option<&SharedSyncStatus>) -> Result<(), String> {
        let url = format!("{}/focus_sessions", self.base_url.trim_end_matches('/'));
        let resp = self.client.post(&url)
            .header("apikey", &self.api_key)
            .header("Content-Type", "application/json")
            .json(session)
            .send()
            .await
            .map_err(|e| format!("Request error: {}", e));
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
                    Err(err)
                }
            }
            Err(e) => {
                if let Some(shared) = status {
                    let mut s = shared.lock().unwrap();
                    s.update(false, Some(e.clone()));
                }
                Err(e)
            }
        }
    }

    /// Pull all focus sessions from Supabase
    pub async fn pull_focus_sessions(&self) -> Result<Vec<FocusSession>, String> {
        let url = format!("{}/focus_sessions", self.base_url.trim_end_matches('/'));
        let resp = self.client.get(&url)
            .header("apikey", &self.api_key)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("Request error: {}", e))?;
        if resp.status().is_success() {
            let sessions: Vec<FocusSession> = resp.json().await.map_err(|e| format!("Deserialization error: {}", e))?;
            Ok(sessions)
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(format!("Supabase pull failed: {} - {}", status, body))
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