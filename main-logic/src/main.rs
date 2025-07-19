mod session;
mod metrics;
mod apprules;
mod platform;
mod logger;
mod db;
mod graceful_shutdown;

use session::SessionManager;
use metrics::Metrics;
use apprules::AppRules;
use db::DbHandle;

use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};

fn main() {
    let apprules = AppRules::new();
    let mut metrics = Metrics::new();
    let db_handle = DbHandle::new().expect("Failed to initialize database");
    let session_mgr = Arc::new(Mutex::new(SessionManager::new(apprules.clone(), db_handle)));
    let shutdown_flag = Arc::new(AtomicBool::new(false));

    graceful_shutdown::install(session_mgr.clone(), shutdown_flag.clone());

    while !shutdown_flag.load(Ordering::SeqCst) {
        let mut mgr = session_mgr.lock().unwrap();
        mgr.poll();
        metrics.update_from_session(&mgr);
        if metrics.should_log_summary() {
            metrics.log_summary();
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    // After loop: ensure session is ended and logged
    let mut mgr = session_mgr.lock().unwrap();
    mgr.end_active_session();
}
