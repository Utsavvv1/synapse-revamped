mod session;
mod metrics;
mod apprules;
mod platform;
mod logger;
mod db;

use session::SessionManager;
use metrics::Metrics;
use apprules::AppRules;
use db::DbHandle;

fn main() {
    let apprules = AppRules::new();
    let mut metrics = Metrics::new();
    let db_handle = DbHandle::new().expect("Failed to initialize database");
    let mut session_mgr = SessionManager::new(apprules.clone(), db_handle);

    loop {
        session_mgr.poll();
        metrics.update_from_session(&session_mgr);
        if metrics.should_log_summary() {
            metrics.log_summary();
        }
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
}
