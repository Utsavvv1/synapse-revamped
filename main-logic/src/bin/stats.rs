use main_logic::{db::DbHandle, api};
use dotenvy;

fn main() {
    dotenvy::from_filename("../src-tauri/.env").ok();
    let db = match DbHandle::new() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to open DB: {:?}", e);
            std::process::exit(1);
        }
    };
    match api::total_focus_time_today(&db) {
        Ok(focus_time) => println!("Focus time today: {} seconds", focus_time),
        Err(e) => eprintln!("Error: {:?}", e),
    }
    match api::total_distractions_today(&db) {
        Ok(distractions) => println!("Distractions today: {}", distractions),
        Err(e) => eprintln!("Error: {:?}", e),
    }
    match api::total_focus_sessions_today(&db) {
        Ok(session_count) => println!("Focus sessions today: {}", session_count),
        Err(e) => eprintln!("Error: {:?}", e),
    }
} 