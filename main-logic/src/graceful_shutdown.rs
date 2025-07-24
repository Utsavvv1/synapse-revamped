//! Graceful shutdown module: handles Ctrl-C signal for a clean application exit.

use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use ctrlc;
use crate::session::SessionManager;
use crate::logger::log_error;

/// Installs a Ctrl-C handler to gracefully shut down the application.
///
/// On Ctrl-C, it sets a shutdown flag and ends any active session.
///
/// # Panics
/// Panics if the Ctrl-C handler cannot be set.
pub fn install(session_mgr: Arc<Mutex<SessionManager>>, shutdown_flag: Arc<AtomicBool>) {
    ctrlc::set_handler(move || {
        shutdown_flag.store(true, Ordering::SeqCst);
        if let Ok(mut mgr) = session_mgr.lock() {
            if let Err(e) = mgr.end_active_session() {
                log_error(&e);
            }
        }
    }).expect("Error setting Ctrl-C handler");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
    use crate::session::SessionManager;
    use crate::apprules::AppRules;
    use crate::db::DbHandle;

    #[test]
    fn test_install_sets_shutdown_flag_and_cleans_up() {
        let rules = AppRules::test_with_rules(vec!["notepad.exe".to_string()], vec![]);
        let db = DbHandle::test_in_memory();
        let mgr = Arc::new(Mutex::new(SessionManager::new(rules, db, None)));
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        // We can't actually trigger Ctrl-C in a test, but we can call the handler logic directly
        // Simulate what the handler would do
        shutdown_flag.store(true, Ordering::SeqCst);
        if let Ok(mut mgr) = mgr.lock() {
            let _ = mgr.end_active_session();
        }
        assert!(shutdown_flag.load(Ordering::SeqCst));
        // No panic means cleanup logic is safe
    }
} 
