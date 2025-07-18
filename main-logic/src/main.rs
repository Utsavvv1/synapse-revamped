mod session;
mod metrics;
mod apprules;
mod platform;
mod logger;
// mod db; // Uncomment when db.rs is added

use session::SessionManager;
use metrics::Metrics;
use apprules::AppRules;
// use db::DbHandle; // Uncomment when db.rs is added

fn main() {
    let apprules = AppRules::new();
    let mut metrics = Metrics::new();
    let mut session_mgr = SessionManager::new(apprules.clone());
    // let mut db_handle = DbHandle::new(); // Uncomment when db.rs is added

    loop {
        session_mgr.poll();
        metrics.update_from_session(&session_mgr);
        if metrics.should_log_summary() {
            metrics.log_summary();
        }
        // db_handle.sync_if_needed(); // Uncomment when db.rs is added
        // Add other feature calls here
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
}
