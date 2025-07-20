use main_logic::apprules::AppRules;
use main_logic::db::DbHandle;
use main_logic::session::{SessionManager, FocusSession};
use main_logic::metrics::Metrics;
use main_logic::logger::{log_event, log_error};
use main_logic::error::SynapseError;
use std::time::SystemTime;

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
    mgr.current_session = Some(FocusSession {
        start_time: now,
        end_time: None,
        work_apps: vec!["notepad.exe".to_string()],
        is_active: true,
        distraction_attempts: 0,
    });
    mgr.session_id = Some(1);
    assert!(mgr.current_session.is_some());

    // Simulate app usage and logging
    let process = "notepad.exe";
    let log_result = log_event(Some(&mgr.db_handle), process, false, Some(false), mgr.session_id, Some(100), Some(200), Some(100));
    assert!(log_result.is_ok());
    metrics.update(process, false);
    metrics.update("chrome.exe", true);
    assert_eq!(metrics.total_checks, 2);
    assert_eq!(metrics.blocked_count, 1);

    // Simulate distraction
    if let Some(session) = mgr.current_session.as_mut() {
        session.distraction_attempts += 1;
    }
    assert_eq!(mgr.current_session.as_ref().unwrap().distraction_attempts, 1);

    // End session
    mgr.end_active_session().unwrap();
    assert!(mgr.current_session.is_none());
    assert!(mgr.session_id.is_none());

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
