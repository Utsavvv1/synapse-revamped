use main_logic::apprules::AppRules;
use main_logic::db::DbHandle;
use main_logic::session::{SessionManager, FocusSession};
use main_logic::metrics::Metrics;
use main_logic::logger::{log_event, log_error};
use main_logic::error::SynapseError;
use std::time::SystemTime;
use main_logic::sync::{SupabaseSync, SharedSyncStatus, SyncStatus};
use std::sync::Arc;
use std::sync::Mutex;
use main_logic::session::AppStatus;
use main_logic::sync::merge_sessions;

#[test]
fn test_full_session_lifecycle_and_metrics() {
    // Setup test rules and in-memory DB
    let rules = AppRules::test_with_rules(
        vec!["notepad.exe".to_string(), "word.exe".to_string()],
        vec!["chrome.exe".to_string(), "game.exe".to_string()],
    );
    let mut db = DbHandle::test_in_memory();
    db.test_conn().execute(
        "CREATE TABLE IF NOT EXISTS focus_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            start_time INTEGER NOT NULL,
            end_time INTEGER,
            work_apps TEXT,
            distraction_attempts INTEGER
        )",
        [],
    ).unwrap();
    db.test_conn().execute(
        "CREATE TABLE IF NOT EXISTS app_usage_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL,
            process_name TEXT NOT NULL,
            is_blocked BOOLEAN NOT NULL,
            distraction BOOLEAN,
            session_id INTEGER,
            start_time INTEGER,
            end_time INTEGER,
            duration_secs INTEGER
        )",
        [],
    ).unwrap();
    let mut mgr = SessionManager::new(rules.clone(), db);
    let mut metrics = Metrics::new();

    // Simulate session start
    let now = SystemTime::now();
    mgr.set_current_session(FocusSession::new(now, vec!["notepad.exe".to_string()]));
    mgr.set_session_id(1.into());
    assert!(mgr.current_session().is_some());

    // Simulate app usage and logging
    let process = "notepad.exe";
    let log_result = log_event(Some(mgr.db_handle()), process, false, Some(false), mgr.session_id().map(Into::into), Some(100), Some(200), Some(100));
    assert!(log_result.is_ok());
    metrics.update(process, false);
    metrics.update("chrome.exe", true);
    assert_eq!(metrics.total_checks, 2);
    assert_eq!(metrics.blocked_count, 1);

    // Simulate distraction
    if let Some(session) = mgr.current_session_mut() {
        session.increment_distraction_attempts();
    }
    assert_eq!(mgr.current_session().unwrap().distraction_attempts(), 1);

    // End session
    mgr.end_active_session().unwrap();
    assert!(mgr.current_session().is_none());
    assert!(mgr.session_id().is_none());

    // Log summary
    metrics.last_summary = std::time::Instant::now() - std::time::Duration::from_secs(61);
    assert!(metrics.log_summary().is_ok());
}

#[test]
fn test_error_propagation_and_logging() {
    // Simulate an error and ensure it is logged
    let err = SynapseError::Other("integration test error".to_string());
    log_error(&err);
    let contents = std::fs::read_to_string("synapse.log").unwrap();
    assert!(contents.contains("integration test error"));
}

#[tokio::test]
async fn test_supabase_sync_push_focus_session() {
    // Try to load Supabase config from .env
    let sync = match SupabaseSync::from_env() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[Supabase sync test] Skipping: {}", e);
            return;
        }
    };
    // Create a dummy FocusSession
    let session = FocusSession::new(
        std::time::SystemTime::now(),
        vec!["test_app.exe".to_string()]
    );
    // Attempt to push to Supabase
    let result = sync.push_focus_session(&session).await;
    match result {
        Ok(_) => println!("[Supabase sync test] Session synced!"),
        Err(e) => eprintln!("[Supabase sync test] Sync failed: {}", e),
    }
}

#[tokio::test]
async fn test_supabase_sync_status_tracking() {
    // Try to load Supabase config from .env
    let sync = match SupabaseSync::from_env() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[Supabase sync status test] Skipping: {}", e);
            return;
        }
    };
    // Create a dummy FocusSession
    let session = FocusSession::new(
        std::time::SystemTime::now(),
        vec!["test_app.exe".to_string()]
    );
    // Create shared sync status
    let status = Arc::new(Mutex::new(SyncStatus::new()));
    // Attempt to push to Supabase and update status
    let result = sync.push_focus_session_with_status(&session, Some(&status)).await;
    match result {
        Ok(_) => println!("[Supabase sync status test] Session synced!"),
        Err(e) => eprintln!("[Supabase sync status test] Sync failed: {}", e),
    }
    // Print sync status
    let status = status.lock().unwrap();
    println!("[Supabase sync status test] Last sync time: {:?}", status.last_sync_time);
    println!("[Supabase sync status test] Last result: {:?}", status.last_result);
    println!("[Supabase sync status test] Last error: {:?}", status.last_error);
}

#[tokio::test]
async fn test_supabase_pull_focus_sessions() {
    // Try to load Supabase config from .env
    let sync = match SupabaseSync::from_env() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[Supabase pull test] Skipping: {}", e);
            return;
        }
    };
    // Attempt to pull all focus sessions
    match sync.pull_focus_sessions().await {
        Ok(sessions) => {
            println!("[Supabase pull test] Pulled {} sessions:", sessions.len());
            for (i, session) in sessions.iter().enumerate() {
                println!("Session {}: {:?}", i + 1, session);
            }
        }
        Err(e) => eprintln!("[Supabase pull test] Pull failed: {}", e),
    }
}

#[tokio::test]
async fn test_supabase_merge_sessions() {
    // Try to load Supabase config from .env
    let sync = match SupabaseSync::from_env() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[Supabase merge test] Skipping: {}", e);
            return;
        }
    };
    // Create dummy local sessions
    let now = std::time::SystemTime::now();
    let local_sessions = vec![
        FocusSession::new(now, vec!["local_app.exe".to_string()]),
    ];
    // Pull remote sessions
    let remote_sessions = match sync.pull_focus_sessions().await {
        Ok(sessions) => sessions,
        Err(e) => {
            eprintln!("[Supabase merge test] Pull failed: {}", e);
            return;
        }
    };
    // Merge sessions
    let merged = merge_sessions(local_sessions, remote_sessions);
    println!("[Supabase merge test] Merged sessions:");
    for (i, session) in merged.iter().enumerate() {
        println!("Session {}: {:?}", i + 1, session);
    }
} 
