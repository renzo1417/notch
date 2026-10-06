use tauri::{AppHandle, Manager, State};

use crate::state::{AppState, ShellConfig};
use crate::window::{self, MonitorInfo};

/// Changes the shell state (e.g. "compact" vs "expanded") and resizes/re-centers the native window.
#[tauri::command]
pub fn set_notch_state(
    app: AppHandle,
    state: State<'_, AppState>,
    notch_state: String,
) -> Result<(), String> {
    let mut config = state
        .shell_config
        .lock()
        .map_err(|e| format!("Failed to lock shell config: {e}"))?;

    config.current_state = notch_state.clone();
    let selected_monitor = config.selected_monitor.clone();
    drop(config);

    if let Some(webview_window) = app.get_webview_window("main") {
        let window = webview_window.as_ref().window();
        window::apply_notch_state(&window, &notch_state, selected_monitor.as_deref())
            .map_err(|e| format!("Failed to apply notch window bounds: {e}"))?;
    }

    Ok(())
}

/// Retrieves the current shell configuration.
#[tauri::command]
pub fn get_shell_config(state: State<'_, AppState>) -> Result<ShellConfig, String> {
    let config = state
        .shell_config
        .lock()
        .map_err(|e| format!("Failed to lock shell config: {e}"))?;
    Ok(config.clone())
}

/// Configures whether the notch window stays always-on-top.
#[tauri::command]
pub fn set_always_on_top(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<bool, String> {
    let mut config = state
        .shell_config
        .lock()
        .map_err(|e| format!("Failed to lock shell config: {e}"))?;

    config.always_on_top = enabled;
    drop(config);

    if let Some(window) = app.get_webview_window("main") {
        window
            .set_always_on_top(enabled)
            .map_err(|e| format!("Failed to set always on top: {e}"))?;
    }

    Ok(enabled)
}

/// Re-evaluates top-center placement for the notch window (e.g. after display changes).
#[tauri::command]
pub fn reposition_notch(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let config = state
        .shell_config
        .lock()
        .map_err(|e| format!("Failed to lock shell config: {e}"))?;

    let current_state = config.current_state.clone();
    let selected_monitor = config.selected_monitor.clone();
    drop(config);

    if let Some(webview_window) = app.get_webview_window("main") {
        let window = webview_window.as_ref().window();
        window::apply_notch_state(&window, &current_state, selected_monitor.as_deref())
            .map_err(|e| format!("Failed to reposition notch window: {e}"))?;
    }

    Ok(())
}

/// Lists available monitors with display properties and DPI scale factors.
#[tauri::command]
pub fn list_monitors(app: AppHandle) -> Result<Vec<MonitorInfo>, String> {
    if let Some(webview_window) = app.get_webview_window("main") {
        let window = webview_window.as_ref().window();
        let monitors = window
            .available_monitors()
            .map_err(|e| format!("Failed to query monitors: {e}"))?;

        let primary_name = window
            .primary_monitor()
            .ok()
            .flatten()
            .and_then(|p| p.name().map(|s| s.to_string()));

        let list = monitors
            .into_iter()
            .map(|m| {
                let name = m.name().map(|s| s.to_string());
                let is_primary = match (&name, &primary_name) {
                    (Some(n), Some(p)) => n == p,
                    _ => false,
                };
                let size = m.size();
                MonitorInfo {
                    name,
                    width: size.width,
                    height: size.height,
                    scale_factor: m.scale_factor(),
                    is_primary,
                }
            })
            .collect();

        Ok(list)
    } else {
        Err("Main window not found".to_string())
    }
}
