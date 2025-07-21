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

use session::SessionManager;
use metrics::Metrics;
use apprules::AppRules;
use db::DbHandle;
use logger::log_error;
use constants::MAIN_LOOP_SLEEP_MS;

use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};

fn main() {
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
        std::thread::sleep(std::time::Duration::from_millis(MAIN_LOOP_SLEEP_MS));
    }
    // After loop: ensure session is ended and logged
    let mut mgr = session_mgr.lock().unwrap();
    if let Err(e) = mgr.end_active_session() {
        log_error(&e);
    }
}
