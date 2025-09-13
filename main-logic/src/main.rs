//! Main application entry point and logic loop.
mod error;
mod api;
mod session;
mod metrics;
mod apprules;
mod platform;
mod logger;
mod db;
mod graceful_shutdown;
mod types;
mod constants;
mod sync;
use notify::{RecommendedWatcher, RecursiveMode, Event, EventKind, Watcher};
use std::sync::mpsc::channel;
use std::path::Path;

use session::SessionManager;
use metrics::Metrics;
use apprules::AppRules;
use db::DbHandle;
use logger::{log_error, log_error_with_context};
use constants::MAIN_LOOP_SLEEP_MS;
use sync::{SupabaseSync, SyncStatus};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Check Supabase connection at startup
    match SupabaseSync::from_env(false) {
        Ok(_) => println!("Supabase connection established!"),
        Err(e) => println!("Supabase connection failed: {}", e),
    }
    let apprules = match AppRules::new() {
        Ok(rules) => rules,
        Err(e) => {
            log_error_with_context("Initializing AppRules", &e);
            return;
        }
    };
    let mut metrics = Metrics::new();
    let db_handle = match DbHandle::new() {
        Ok(db) => db,
        Err(e) => {
            log_error_with_context("Initializing DbHandle", &e);
            return;
        }
    };
    let supabase_sync = SupabaseSync::from_env(false).ok();
    let sync_status = Arc::new(Mutex::new(SyncStatus::new()));
    // Set up a Tokio runtime for async tasks
    // let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"); // This line is removed as per edit hint

    println!("Constructing SessionManager with supabase_sync: {}", supabase_sync.is_some());
    let session_mgr = Arc::new(Mutex::new(SessionManager::new(apprules.clone(), db_handle, supabase_sync.clone())));
    let shutdown_flag = Arc::new(AtomicBool::new(false));

    // --- File watcher for apprules.json ---
    {
        let session_mgr = session_mgr.clone();
        let shutdown_flag = shutdown_flag.clone();
        thread::spawn(move || {
            let (tx, rx) = channel();
            let path_str = std::env::var("APPRULES_PATH").unwrap_or_else(|_| "apprules.json".to_string());
            let path = Path::new(&path_str);
            let mut watcher = RecommendedWatcher::new(tx, notify::Config::default()).expect("Failed to create watcher");
            watcher.watch(path, RecursiveMode::NonRecursive).expect("Failed to watch apprules.json");
            while !shutdown_flag.load(Ordering::SeqCst) {
                if let Ok(event) = rx.recv_timeout(Duration::from_secs(1)) {
                    match event {
                        Ok(Event { kind: EventKind::Modify(_), .. }) => {
                            log::info!("[Watcher] Detected apprules.json change, reloading...");
                            match AppRules::new() {
                                Ok(new_rules) => {
                                    let mut mgr = session_mgr.lock().unwrap();
                                    mgr.set_apprules(new_rules);
                                    log::info!("[Watcher] AppRules reloaded successfully.");
                                },
                                Err(e) => {
                                    log::error!("[Watcher] Failed to reload AppRules: {}", e);
                                }
                            }
                        },
                        _ => {}
                    }
                }
            }
        });
    }

    graceful_shutdown::install(session_mgr.clone(), shutdown_flag.clone());

    // Set up Supabase sync (optional, can be disabled if env not set)
    // let supabase_sync = SupabaseSync::from_env(false).ok(); // This line is removed as per edit hint
    // let sync_status = Arc::new(Mutex::new(SyncStatus::new())); // This line is removed as per edit hint
    // Set up a Tokio runtime for async tasks // This line is removed as per edit hint
    // let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"); // This line is removed as per edit hint

    while !shutdown_flag.load(Ordering::SeqCst) {
        let mut mgr = session_mgr.lock().unwrap();
        let poll_result = match mgr.poll() {
            Ok(ended_session) => ended_session,
            Err(e) => {
                log_error_with_context("Polling session manager", &e);
                None
            }
        };
        metrics.update_from_session(&mgr);
        if metrics.should_log_summary() {
            if let Err(e) = metrics.log_summary() {
                log_error_with_context("Logging metrics summary", &e);
            }
        }
        // If a session just ended, push it to Supabase
        if let (Some(sync), Some(session)) = (&supabase_sync, poll_result) {
            match serde_json::to_string_pretty(&session) {
                Ok(json) => println!("[DEBUG] Pushing session to Supabase: {}", json),
                Err(e) => eprintln!("[DEBUG] Failed to serialize session: {}", e),
            }
            let status = sync_status.clone();
            let sync = sync.clone();
            // Await the async push
            // REMOVE: push_focus_session_with_status at session end
            // Only update app usage events here
            // Push app usage events for this 
            let db_handle = mgr.db_handle();
            if let Some(sid) = mgr.session_id().map(|id| id.0) {
                match db_handle.get_app_usage_events_for_session(sid) {
                    Ok(events) => {
                        if !events.is_empty() {
                            match sync.push_app_usage_events(&events).await {
                                Ok(_) => println!("[Supabase] App usage events pushed successfully!"),
                                Err(e) => eprintln!("[Supabase] App usage events sync failed: {}", e),
                            }
                        }
                    }
                    Err(e) => eprintln!("[Supabase] Failed to fetch app usage events: {}", e),
                }
            }
            // --- NEW: Always update session in Supabase when it ends ---
            let sync = sync.clone();
            let session_clone = session.clone();
            tokio::spawn(async move {
                let _ = sync.update_focus_session(&session_clone).await;
            });
        }
        thread::sleep(Duration::from_millis(MAIN_LOOP_SLEEP_MS));
    }
    // After loop: ensure session is ended and logged
    let mut mgr = session_mgr.lock().unwrap();
    println!("[Main] Calling end_active_session");
    match mgr.end_active_session() {
        Ok(Some(session)) => {
            match serde_json::to_string_pretty(&session) {
                Ok(json) => println!("[DEBUG] Pushing session to Supabase: {}", json),
                Err(e) => eprintln!("[DEBUG] Failed to serialize session: {}", e),
            }
            if let Some(sync) = &supabase_sync {
                // REMOVE: push_focus_session_with_status at session end
                // Only update app usage events here
                let status = sync_status.clone();
                // Push app usage events for this session
                let db_handle = mgr.db_handle();
                if let Some(sid) = mgr.session_id().map(|id| id.0) {
                    match db_handle.get_app_usage_events_for_session(sid) {
                        Ok(events) => {
                            if !events.is_empty() {
                                match sync.push_app_usage_events(&events).await {
                                    Ok(_) => println!("[Supabase] App usage events pushed successfully!"),
                                    Err(e) => eprintln!("[Supabase] App usage events sync failed: {}", e),
                                }
                            }
                        }
                        Err(e) => eprintln!("[Supabase] Failed to fetch app usage events: {}", e),
                    }
                }
            }
        }
        Ok(None) => {},
        Err(e) => log_error_with_context("Ending active session", &e),
    }
}
