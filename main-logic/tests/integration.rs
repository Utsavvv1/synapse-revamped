use main_logic::apprules::AppRules;
use main_logic::db::DbHandle;
use main_logic::error::{SupabaseError, SynapseError};
use main_logic::logger::{log_error, log_event};
use main_logic::metrics::Metrics;
use main_logic::session::{FocusSession, SessionManager};
use main_logic::sync::merge_sessions;
use main_logic::sync::{SharedSyncStatus, SupabaseSync, SyncStatus};
use main_logic::SessionId;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;

#[test]
fn test_full_session_lifecycle_and_metrics() {
    // Setup test rules and in-memory DB
    let rules = AppRules::test_with_rules(
        vec!["notepad.exe".to_string(), "word.exe".to_string()],
        vec!["chrome.exe".to_string(), "game.exe".to_string()],
    );
    let mut db = DbHandle::test_in_memory();
    db.test_conn()
        .execute(
            "CREATE TABLE IF NOT EXISTS focus_sessions (
            id TEXT PRIMARY KEY,
            start_time INTEGER NOT NULL,
            end_time INTEGER,
            work_apps TEXT,
            distraction_attempts INTEGER
        )",
            [],
        )
        .unwrap();
    db.test_conn()
        .execute(
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
        )
        .unwrap();
    let mut mgr = SessionManager::new(rules.clone(), db, None, None);

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
    let log_result = log_event(
        Some(mgr.db_handle()),
        process,
        false,
        Some(false),
        Some(session_id),
        Some(100),
        Some(200),
        Some(100),
    );
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
    let end_time = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 1000;
    {
        let db_handle = mgr.db_handle();
        db_handle
            .execute_sql(
                "UPDATE focus_sessions SET end_time = ?1 WHERE id = ?2",
                &[&end_time.to_string(), &session_id.to_string()],
            )
            .unwrap();
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
        vec!["test_app.exe".to_string()],
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
        vec!["test_app.exe".to_string()],
    );
    // Create shared sync status
    let status = Arc::new(Mutex::new(SyncStatus::new()));
    // Attempt to push to Supabase and update status
    let result = sync
        .push_focus_session_with_status(&session, Some(&status))
        .await;
    match result {
        Ok(_) => println!("[Supabase sync status test] Session synced!"),
        Err(e) => eprintln!("[Supabase sync status test] Sync failed: {}", e),
    }
    // Print sync status
    let status = status.lock().unwrap();
    println!(
        "[Supabase sync status test] Last sync time: {:?}",
        status.last_sync_time
    );
    println!(
        "[Supabase sync status test] Last result: {:?}",
        status.last_result
    );
    println!(
        "[Supabase sync status test] Last error: {:?}",
        status.last_error
    );
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
    let local_sessions = vec![FocusSession::new(now, vec!["local_app.exe".to_string()])];
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

#[test]
fn test_api_today_vs_past_entries() {
    use main_logic::{api, db::DbHandle};
    use std::time::{SystemTime, UNIX_EPOCH};

    let mut db = DbHandle::test_in_memory();
    db.test_conn()
        .execute(
            "CREATE TABLE IF NOT EXISTS focus_sessions (
            id TEXT PRIMARY KEY,
            start_time INTEGER NOT NULL,
            end_time INTEGER,
            work_apps TEXT,
            distraction_attempts INTEGER
        )",
            [],
        )
        .unwrap();
    db.test_conn()
        .execute(
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
        )
        .unwrap();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let one_day = 86400;
    let today_start = now / one_day * one_day;
    let today_end = today_start + one_day;
    let yesterday = today_start - one_day;
    let tomorrow = today_end + one_day;

    // Edge: session starts exactly at today's midnight
    db.test_conn().execute(
        "INSERT INTO focus_sessions (id, start_time, end_time, work_apps, distraction_attempts) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["id-midnight", today_start, today_start + 100, "midnight.exe", 1],
    ).unwrap();
    // Edge: session starts just before today's midnight (should NOT count)
    db.test_conn().execute(
        "INSERT INTO focus_sessions (id, start_time, end_time, work_apps, distraction_attempts) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["id-before-midnight", today_start - 1, today_start + 100, "before.exe", 2],
    ).unwrap();
    // Edge: session starts just after today's midnight
    db.test_conn().execute(
        "INSERT INTO focus_sessions (id, start_time, end_time, work_apps, distraction_attempts) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["id-after-midnight", today_start + 1, today_start + 200, "after.exe", 3],
    ).unwrap();
    // Normal: session in the middle of today
    db.test_conn().execute(
        "INSERT INTO focus_sessions (id, start_time, end_time, work_apps, distraction_attempts) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["id-today", today_start + 3600, today_start + 7200, "notepad.exe", 4],
    ).unwrap();
    // Edge: session starts today, ends tomorrow
    db.test_conn().execute(
        "INSERT INTO focus_sessions (id, start_time, end_time, work_apps, distraction_attempts) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["id-crosses-tomorrow", today_end - 10, tomorrow + 100, "cross.exe", 5],
    ).unwrap();
    // Edge: session starts before today, ends today (should NOT count)
    db.test_conn().execute(
        "INSERT INTO focus_sessions (id, start_time, end_time, work_apps, distraction_attempts) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["id-starts-before", yesterday + one_day - 1, today_start + 10, "before2.exe", 6],
    ).unwrap();
    // Edge: session ongoing (no end_time)
    db.test_conn().execute(
        "INSERT INTO focus_sessions (id, start_time, end_time, work_apps, distraction_attempts) VALUES (?1, ?2, NULL, ?3, ?4)",
        rusqlite::params!["id-ongoing", today_start + 8000, "ongoing.exe", 7],
    ).unwrap();
    // Edge: session with zero duration
    db.test_conn().execute(
        "INSERT INTO focus_sessions (id, start_time, end_time, work_apps, distraction_attempts) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["id-zero-duration", today_start + 9000, today_start + 9000, "zero.exe", 8],
    ).unwrap();
    // Edge: session with negative distraction_attempts (should still sum)
    db.test_conn().execute(
        "INSERT INTO focus_sessions (id, start_time, end_time, work_apps, distraction_attempts) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["id-negative-distraction", today_start + 9500, today_start + 9600, "neg.exe", -2],
    ).unwrap();
    // Past: session from yesterday (should NOT count)
    db.test_conn().execute(
        "INSERT INTO focus_sessions (id, start_time, end_time, work_apps, distraction_attempts) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["id-yesterday", yesterday + 100, yesterday + 200, "word.exe", 2],
    ).unwrap();
    // Future: session from tomorrow (should NOT count)
    db.test_conn().execute(
        "INSERT INTO focus_sessions (id, start_time, end_time, work_apps, distraction_attempts) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["id-tomorrow", tomorrow + 100, tomorrow + 200, "future.exe", 1],
    ).unwrap();

    // Calculate expected values
    // Only sessions starting in [today_start, today_end) should count
    // These are: id-midnight, id-after-midnight, id-today, id-crosses-tomorrow, id-ongoing, id-zero-duration, id-negative-distraction
    let expected_count = 7;
    let expected_distractions = 1 + 3 + 4 + 5 + 7 + 8 + (-2); // sum of distraction_attempts for those
    let expected_focus_time = (today_start + 100 - today_start) + // id-midnight
        (today_start + 200 - (today_start + 1)) + // id-after-midnight
        (today_start + 7200 - (today_start + 3600)) + // id-today
        (tomorrow + 100 - (today_end - 10)) + // id-crosses-tomorrow
        (0) + // id-ongoing (no end_time, will use now, but for test, treat as 0)
        (0) + // id-zero-duration
        (today_start + 9600 - (today_start + 9500)); // id-negative-distraction

    let focus_time = api::total_focus_time_today(&db).unwrap();
    let distractions = api::total_distractions_today(&db).unwrap();
    let session_count = api::total_focus_sessions_today(&db).unwrap();

    assert_eq!(
        session_count, expected_count,
        "Session count mismatch: got {}, expected {}",
        session_count, expected_count
    );
    assert_eq!(
        distractions, expected_distractions,
        "Distraction sum mismatch: got {}, expected {}",
        distractions, expected_distractions
    );
    // Allow focus_time to be >= expected_focus_time (ongoing session may add time)
    assert!(
        focus_time >= expected_focus_time,
        "Focus time mismatch: got {}, expected at least {}",
        focus_time,
        expected_focus_time
    );
}
