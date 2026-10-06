use crate::integrations::launcher::{self, InspectedApp, LaunchableApp};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn get_launcher_apps(state: State<'_, AppState>) -> Result<Vec<LaunchableApp>, String> {
    Ok(state.launcher_manager.get_apps())
}

#[tauri::command]
pub fn pick_app_file() -> Result<Option<String>, String> {
    launcher::pick_executable_or_shortcut()
}

#[tauri::command]
pub fn inspect_app_file(
    state: State<'_, AppState>,
    path: String,
) -> Result<InspectedApp, String> {
    state.launcher_manager.inspect_file(&path)
}

#[tauri::command]
pub fn add_launcher_app(
    state: State<'_, AppState>,
    app: LaunchableApp,
) -> Result<LaunchableApp, String> {
    state.launcher_manager.add_app(app)
}

#[tauri::command]
pub fn update_launcher_app(
    state: State<'_, AppState>,
    app: LaunchableApp,
) -> Result<LaunchableApp, String> {
    state.launcher_manager.update_app(app)
}

#[tauri::command]
pub fn remove_launcher_app(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.launcher_manager.remove_app(&id)
}

#[tauri::command]
pub fn reorder_launcher_apps(
    state: State<'_, AppState>,
    ids: Vec<String>,
) -> Result<Vec<LaunchableApp>, String> {
    state.launcher_manager.reorder_apps(&ids)
}

#[tauri::command]
pub fn locate_app(
    state: State<'_, AppState>,
    id: String,
    new_path: String,
) -> Result<LaunchableApp, String> {
    state.launcher_manager.locate_app(&id, &new_path)
}

#[tauri::command]
pub fn launch_app(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.launcher_manager.launch_app(&id)
}
