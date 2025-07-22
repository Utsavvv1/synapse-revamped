//! Main application entry point and logic loop.
mod error;
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

use session::SessionManager;
use metrics::Metrics;
use apprules::AppRules;
use db::DbHandle;
use logger::log_error;
use constants::MAIN_LOOP_SLEEP_MS;
use sync::{SupabaseSync, SyncStatus};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

fn main() {
    // Check Supabase connection at startup
    match SupabaseSync::from_env() {
        Ok(_) => println!("Supabase connection established!"),
        Err(e) => println!("Supabase connection failed: {}", e),
    }
    let apprules = match AppRules::new() {
        Ok(rules) => rules,
        Err(e) => {
            log_error(&e);
            return;
        }
    };
    let mut metrics = Metrics::new();
    let db_handle = match DbHandle::new() {
        Ok(db) => db,
        Err(e) => {
            log_error(&e);
            return;
        }
    };
    let session_mgr = Arc::new(Mutex::new(SessionManager::new(apprules.clone(), db_handle)));
    let shutdown_flag = Arc::new(AtomicBool::new(false));

    graceful_shutdown::install(session_mgr.clone(), shutdown_flag.clone());

    // Set up Supabase sync (optional, can be disabled if env not set)
    let supabase_sync = SupabaseSync::from_env().ok();
    let sync_status = Arc::new(Mutex::new(SyncStatus::new()));
    // Set up a Tokio runtime for async tasks
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    while !shutdown_flag.load(Ordering::SeqCst) {
        let mut mgr = session_mgr.lock().unwrap();
        if let Err(e) = mgr.poll() {
            log_error(&e);
        }
        metrics.update_from_session(&mgr);
        if metrics.should_log_summary() {
            if let Err(e) = metrics.log_summary() {
                log_error(&e);
            }
        }
        // If a session just ended, push it to Supabase in the background
        if let Some(sync) = &supabase_sync {
            if let Some(session) = mgr.current_session() {
                let session = session.clone();
                let sync = sync.clone();
                let status = sync_status.clone();
                rt.spawn(async move {
                    match sync.push_focus_session_with_status(&session, Some(&status)).await {
                        Ok(_) => println!("[Supabase] Session pushed successfully!"),
                        Err(e) => eprintln!("[Supabase] Sync failed: {}", e),
                    }
                });
            }
        }
        thread::sleep(Duration::from_millis(MAIN_LOOP_SLEEP_MS));
    }
    // After loop: ensure session is ended and logged
    let mut mgr = session_mgr.lock().unwrap();
    if let Err(e) = mgr.end_active_session() {
        log_error(&e);
    }
}
