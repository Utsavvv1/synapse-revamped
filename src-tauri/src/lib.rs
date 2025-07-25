use main_logic::{DbHandle, api};

#[tauri::command]
fn total_focus_time_today_cmd() -> Result<i64, String> {
    let db = DbHandle::new().map_err(|e| format!("{:?}", e))?;
    api::total_focus_time_today(&db).map_err(|e| format!("{:?}", e))
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
        total_focus_sessions_today_cmd
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
