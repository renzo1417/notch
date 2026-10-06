pub mod commands;
pub mod integrations;
pub mod state;
pub mod tray;
pub mod window;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::ping::ping,
            commands::shell::set_notch_state,
            commands::shell::get_shell_config,
            commands::shell::set_always_on_top,
            commands::shell::reposition_notch,
            commands::shell::list_monitors,
            commands::media::get_media_state,
            commands::media::media_toggle_play_pause,
            commands::media::media_previous,
            commands::media::media_next,
        ])
        .setup(|app| {
            let handle = app.handle();
            let _ = window::setup_notch_window(handle);
            let _ = tray::setup_tray(handle);
            let media_mgr = app.state::<AppState>().media_manager.clone();
            media_mgr.init(handle.clone());
            Ok(())
        })

        .on_window_event(|window, event| match event {
            // Hiding on close prevents accidental app termination; only explicit Quit terminates
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.hide();
            }
            // Re-evaluate placement and DPI scaling after display/scale changes
            tauri::WindowEvent::ScaleFactorChanged { .. } => {
                let app = window.app_handle();
                let state = app.state::<AppState>();
                let current_state = {
                    let config = state.shell_config.lock();
                    config.map(|c| c.current_state.clone()).unwrap_or_else(|_| "compact".to_string())
                };
                let selected_monitor = {
                    let config = state.shell_config.lock();
                    config.map(|c| c.selected_monitor.clone()).unwrap_or(None)
                };
                let _ = window::apply_notch_state(
                    window,
                    &current_state,
                    selected_monitor.as_deref(),
                );
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running Project Notch desktop application");
}
