use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use ctrlc;
use crate::session::SessionManager;

pub fn install(_session_mgr: Arc<Mutex<SessionManager>>, shutdown_flag: Arc<AtomicBool>) {
    ctrlc::set_handler(move || {
        shutdown_flag.store(true, Ordering::SeqCst);
        // Do NOT call end_active_session here. The main loop will handle it.
    }).expect("Error setting Ctrl-C handler");
} 
