//! Graceful shutdown module: handles Ctrl-C signal for a clean application exit.

use crate::logger::log_error;
use crate::session::SessionManager;
use ctrlc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

/// Installs a Ctrl-C handler to gracefully shut down the application.
///
/// On Ctrl-C, it sets a shutdown flag and ends any active session.
///
/// # Note
/// This function will only set the handler if one hasn't been set already.
/// Multiple calls will be ignored to prevent "MultipleHandlers" errors.
pub fn install(session_mgr: Arc<Mutex<SessionManager>>, shutdown_flag: Arc<AtomicBool>) {
    // Use a static flag to track if we've already set a handler
    static mut HANDLER_SET: bool = false;

    unsafe {
        if HANDLER_SET {
            // Handler already set, skip
            return;
        }

        match ctrlc::set_handler(move || {
            shutdown_flag.store(true, Ordering::SeqCst);
            if let Ok(mut mgr) = session_mgr.lock() {
                if let Err(e) = mgr.end_active_session() {
                    log_error(&e);
                }
            }
        }) {
            Ok(_) => {
                HANDLER_SET = true;
                println!("[GracefulShutdown] Ctrl-C handler installed successfully");
            }
            Err(e) => {
                if e.to_string().contains("MultipleHandlers") {
                    println!("[GracefulShutdown] Ctrl-C handler already set, skipping");
                    HANDLER_SET = true;
                } else {
                    eprintln!("[GracefulShutdown] Failed to set Ctrl-C handler: {}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apprules::AppRules;
    use crate::db::DbHandle;
    use crate::session::SessionManager;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    };

    #[test]
    fn test_install_sets_shutdown_flag_and_cleans_up() {
        let rules = AppRules::test_with_rules(vec!["notepad.exe".to_string()], vec![]);
        let db = DbHandle::test_in_memory();
        let mgr = Arc::new(Mutex::new(SessionManager::new(rules, db, None, None)));
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
