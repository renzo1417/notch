use crate::integrations::launcher::{self, InspectedApp, LaunchableApp};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_installed_apps(
    state: State<'_, AppState>,
    refresh: Option<bool>,
) -> Result<launcher::InstalledAppCatalog, String> {
    let manager = state.launcher_manager.clone();
    tauri::async_runtime::spawn_blocking(move || {
        manager.list_installed_apps(refresh.unwrap_or(false))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn get_installed_app_icon(
    state: State<'_, AppState>,
    catalog_id: String,
) -> Result<Option<String>, String> {
    let manager = state.launcher_manager.clone();
    tauri::async_runtime::spawn_blocking(move || manager.installed_app_icon(&catalog_id))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn pin_installed_app(
    state: State<'_, AppState>,
    catalog_id: String,
    replace_id: Option<String>,
) -> Result<LaunchableApp, String> {
    let manager = state.launcher_manager.clone();
    tauri::async_runtime::spawn_blocking(move || {
        manager.pin_installed_app(&catalog_id, replace_id.as_deref())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn get_launcher_apps(state: State<'_, AppState>) -> Result<Vec<LaunchableApp>, String> {
    let manager = state.launcher_manager.clone();
    tauri::async_runtime::spawn_blocking(move || manager.get_apps())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn pick_app_file() -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(launcher::pick_executable_or_shortcut)
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub fn inspect_app_file(state: State<'_, AppState>, path: String) -> Result<InspectedApp, String> {
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
pub async fn launch_app(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let manager = state.launcher_manager.clone();
    tauri::async_runtime::spawn_blocking(move || manager.launch_app(&id))
        .await
        .map_err(|error| error.to_string())?
}
