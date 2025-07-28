use main_logic::{DbHandle, api, apprules}; // Added apprules
use dotenvy;
use std::thread;
use tauri::{Manager, Emitter};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, window::WindowBuilder, WebviewWindowBuilder, WebviewUrl};

static TAURI_APP: once_cell::sync::OnceCell<Arc<Mutex<tauri::AppHandle>>> = once_cell::sync::OnceCell::new();

#[tauri::command]
fn total_focus_time_today_cmd() -> Result<i64, String> {
    let db = DbHandle::new().map_err(|e| format!("{:?}", e))?;
    let result = api::total_focus_time_today(&db);
    // println!("total_focus_time_today_cmd result: {:?}", result);
    result.map_err(|e| format!("{:?}", e))
}

#[tauri::command]
fn total_distractions_today_cmd() -> Result<i64, String> {
    let db = DbHandle::new().map_err(|e| format!("{:?}", e))?;
    api::total_distractions_today(&db).map_err(|e| format!("{:?}", e))
}

#[tauri::command]
fn total_focus_sessions_today_cmd() -> Result<i64, String> {
    let db = DbHandle::new().map_err(|e| format!("{:?}", e))?;
    api::total_focus_sessions_today(&db).map_err(|e| format!("{:?}", e))
}

#[cfg(target_os = "windows")]
#[tauri::command]
fn get_installed_apps_cmd() -> Vec<(String, String)> {
    main_logic::api::get_installed_apps_api()
}

#[tauri::command]
fn update_app_rules_cmd(whitelist: Vec<String>, blacklist: Vec<String>) -> Result<(), String> {
  println!("update_app_rules_cmd called with whitelist: {:?}, blacklist: {:?}", whitelist, blacklist);
  let whitelist_clone = whitelist.clone();
  let blacklist_clone = blacklist.clone();
  let result = apprules::update_app_rules(whitelist, blacklist);
  println!("update_app_rules_cmd called with whitelist: {:?}, blacklist: {:?}", whitelist_clone, blacklist_clone);
  println!("update_app_rules_cmd result: {:?}", result);
  result.map_err(|e| format!("{:?}", e))
}

#[tauri::command]
fn start_focus_mode_cmd() -> Result<String, String> {
    // For now, just return success - in a real implementation this would trigger the session manager
    // to start a focus session immediately
    Ok("Focus mode started".to_string())
}


#[tauri::command]
fn handle_distraction_modal_action(action: String, app_name: String) -> Result<String, String> {
    println!("Distraction modal action: {} for app: {}", action, app_name);
    
    match action.as_str() {
        "close_app" => {
            // In a real implementation, you could try to close the app
            // For now, just return success
            Ok(format!("Closed app: {}", app_name))
        }
        "use_5_mins" => {
            // In a real implementation, you could start a 5-minute timer
            Ok(format!("Allowing {} for 5 minutes", app_name))
        }
        "show_again" => {
            Ok(format!("Will show modal again for {}", app_name))
        }
        _ => Err("Unknown action".to_string())
    }
}


// Add this helper function to create a separate modal window


// Add this helper function to create a separate modal window
pub fn emit_distraction_event(app_name: &str) -> Result<(), String> {
    if let Some(app_handle) = TAURI_APP.get() {
        if let Ok(handle) = app_handle.lock() {
            // Create a new window for the modal
            let window_label = format!("distraction-modal-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
            
            
            let window = WebviewWindowBuilder::new(
                &*handle,
                &window_label,
                WebviewUrl::App("modal.html".into())  // ✅ use modal.html, not /#modal
            )
            .title("Distraction Alert")
            .inner_size(400.0, 300.0)
            .center()
            .resizable(false)
            .always_on_top(true)
            .decorations(false)
            .transparent(true)
            .build()
            .map_err(|e| format!("Failed to create modal window: {}", e))?;

            // Send the app name to the new window
            window.emit("show-distraction-modal", serde_json::json!({
                "app_name": app_name
            })).map_err(|e| format!("Failed to emit to modal window: {}", e))?;
            
            println!("Created distraction modal window for: {}", app_name);
            return Ok(());
        }
    }
    Err("Tauri app handle not available".to_string())
}



// Modify your existing run() function to store the app handle and add the new command
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::from_filename(".env").ok();
    tauri::Builder::default()
        .setup(|app| {
            // Store the app handle for later use
            TAURI_APP.set(Arc::new(Mutex::new(app.handle().clone())))
                .map_err(|_| "Failed to set app handle")?;
            
            // Start backend main logic in a background thread
            thread::spawn(|| {
                main_logic::run_backend_with_emit(emit_distraction_event);
            });
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            total_focus_time_today_cmd,
            total_distractions_today_cmd,
            total_focus_sessions_today_cmd,
            start_focus_mode_cmd,
            handle_distraction_modal_action,  // Add this new command
        #[cfg(target_os = "windows")] get_installed_apps_cmd,
            update_app_rules_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}