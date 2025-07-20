use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use ctrlc;
use crate::session::SessionManager;
use crate::logger::log_error;

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
    #[test]
    fn dummy_test() {
        assert_eq!(2 + 2, 4);
    }
} 
