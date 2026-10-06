pub mod commands;
pub mod integrations;
pub mod state;
pub mod tray;
pub mod window;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![commands::ping::ping])
        .setup(|app| {
            let handle = app.handle();
            let _ = window::setup_notch_window(handle);
            let _ = tray::setup_tray(handle);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Project Notch desktop application");
}
