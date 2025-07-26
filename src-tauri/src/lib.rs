use main_logic::{DbHandle, api, apprules}; // Added apprules
use dotenvy;

#[tauri::command]
fn total_focus_time_today_cmd() -> Result<i64, String> {
    let db = DbHandle::new().map_err(|e| format!("{:?}", e))?;
    let result = api::total_focus_time_today(&db);
    println!("total_focus_time_today_cmd result: {:?}", result);
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::from_filename(".env").ok();
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            // Example usage of main-logic crate
            // main_logic::some_function();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            total_focus_time_today_cmd,
            total_distractions_today_cmd,
            total_focus_sessions_today_cmd,
            #[cfg(target_os = "windows")] get_installed_apps_cmd,
            update_app_rules_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}