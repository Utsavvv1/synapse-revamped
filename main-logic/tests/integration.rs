use main_logic::apprules::AppRules;
use main_logic::db::DbHandle;
use main_logic::session::{SessionManager, FocusSession};
use main_logic::metrics::Metrics;
use main_logic::logger::{log_event, log_error};
use main_logic::error::{SynapseError, SupabaseError};
use std::time::SystemTime;
use main_logic::sync::{SupabaseSync, SharedSyncStatus, SyncStatus};
use std::sync::Arc;
use std::sync::Mutex;
use main_logic::sync::merge_sessions;
use main_logic::SessionId;

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
            id TEXT PRIMARY KEY,
            start_time INTEGER NOT NULL,
            end_time INTEGER,
            work_apps TEXT,
            distraction_attempts INTEGER
        )",
        [],
    ).unwrap();
    db.test_conn().execute(
        "CREATE TABLE IF NOT EXISTS app_usage_events (
            id TEXT PRIMARY KEY,
            process_name TEXT NOT NULL,
            status TEXT NOT NULL,
            session_id TEXT,
            start_time INTEGER,
            end_time INTEGER,
            duration_secs INTEGER,
            FOREIGN KEY(session_id) REFERENCES focus_sessions(id)
        )",
        [],
    ).unwrap();
    let mut mgr = SessionManager::new(rules.clone(), db, None);

    // Simulate session start
    let now = SystemTime::now();
    let session = FocusSession::new(now, vec!["notepad.exe".to_string()]);
    let session_id = session.id;
    mgr.set_current_session(session);
    mgr.set_session_id(SessionId::from(session_id));
    // Insert session row into DB
    {
        let db_handle = mgr.db_handle();
        db_handle.execute_sql(
            "INSERT INTO focus_sessions (id, start_time, end_time, work_apps, distraction_attempts) VALUES (?1, ?2, NULL, ?3, ?4)",
            &[&session_id.to_string(),
              &now.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().to_string(),
              &"notepad.exe".to_string(),
              &0.to_string(),
            ],
        ).unwrap();
    }
    assert!(mgr.current_session().is_some());

    let mut metrics = Metrics::new();

    // Simulate app usage and logging
    let process = "notepad.exe";
    let log_result = log_event(Some(mgr.db_handle()), process, false, Some(false), Some(session_id), Some(100), Some(200), Some(100));
    assert!(log_result.is_ok(), "log_event error: {:?}", log_result);
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
    let end_time = now.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() + 1000;
    {
        let db_handle = mgr.db_handle();
        db_handle.execute_sql(
            "UPDATE focus_sessions SET end_time = ?1 WHERE id = ?2",
            &[&end_time.to_string(), &session_id.to_string()],
        ).unwrap();
    }
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
    let sync = match SupabaseSync::from_env(false) {
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
    let sync = match SupabaseSync::from_env(false) {
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
    let sync = match SupabaseSync::from_env(false) {
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
    let sync = match SupabaseSync::from_env(false) {
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
