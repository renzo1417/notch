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
            commands::coding::get_coding_session,
            commands::coding::set_coding_project_label,
            commands::coding::reset_coding_session,
            commands::launcher::get_launcher_apps,
            commands::launcher::pick_app_file,
            commands::launcher::inspect_app_file,
            commands::launcher::add_launcher_app,
            commands::launcher::update_launcher_app,
            commands::launcher::remove_launcher_app,
            commands::launcher::reorder_launcher_apps,
            commands::launcher::locate_app,
            commands::launcher::launch_app,
            commands::clipboard::get_clipboard_history,
            commands::clipboard::copy_to_clipboard,
            commands::clipboard::clear_clipboard_history,
            commands::clipboard::set_clipboard_paused,
            commands::clipboard::is_clipboard_paused,
            commands::clipboard::set_clipboard_max_items,
            commands::battery::get_battery_state,
            commands::battery::refresh_battery_state,
            commands::downloads::get_watched_folders,
            commands::downloads::add_watched_folder,
            commands::downloads::remove_watched_folder,
            commands::downloads::pick_download_folder,
            commands::downloads::get_download_state,
            commands::downloads::open_download_folder,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::reset_settings,
            commands::settings::get_autostart_status,
            commands::settings::set_autostart_status,
        ])
        .setup(|app| {
            let handle = app.handle();
            let _ = window::setup_notch_window(handle);
            let _ = tray::setup_tray(handle);
            let media_mgr = app.state::<AppState>().media_manager.clone();
            media_mgr.init(handle.clone());
            let fg_mgr = app.state::<AppState>().foreground_manager.clone();
            fg_mgr.init(handle.clone());
            let launcher_mgr = app.state::<AppState>().launcher_manager.clone();
            if let Ok(app_data_dir) = handle.path().app_data_dir() {
                launcher_mgr.init(app_data_dir);
            }
            let clipboard_mgr = app.state::<AppState>().clipboard_manager.clone();
            clipboard_mgr.init(handle.clone());
            let power_mgr = app.state::<AppState>().power_manager.clone();
            power_mgr.init(handle.clone());
            let downloads_mgr = app.state::<AppState>().downloads_manager.clone();
            if let Ok(app_data_dir) = handle.path().app_data_dir() {
                downloads_mgr.init(app_data_dir.clone(), handle.clone());
            }
            let settings_mgr = app.state::<AppState>().settings_manager.clone();
            if let Ok(app_data_dir) = handle.path().app_data_dir() {
                settings_mgr.init(app_data_dir, handle.clone());
                let config = settings_mgr.get_settings();
                if let Ok(mut shell) = app.state::<AppState>().shell_config.lock() {
                    shell.selected_monitor = config.general.selected_monitor.clone();
                    shell.always_on_top = config.general.always_on_top;
                }
                if let Some(webview) = app.get_webview_window("main") {
                    let win = webview.as_ref().window();
                    let _ = win.set_always_on_top(config.general.always_on_top);
                    let _ = window::apply_notch_state(&win, "compact", config.general.selected_monitor.as_deref());
                }
            }
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
