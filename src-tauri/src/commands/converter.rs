use crate::{
    integrations::converter::{Availability, Inspection, Snapshot},
    state::AppState,
};
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn get_converter_status(
    state: State<'_, AppState>,
    refresh: bool,
) -> Result<Availability, String> {
    let manager = state.converter_manager.clone();
    let config = state.settings_manager.get_settings().converter;
    tauri::async_runtime::spawn_blocking(move || manager.status(&config, refresh))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn inspect_conversion_files(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<Inspection, String> {
    let manager = state.converter_manager.clone();
    let config = state.settings_manager.get_settings().converter;
    tauri::async_runtime::spawn_blocking(move || manager.inspect(&config, paths))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn start_conversion(
    app: AppHandle,
    state: State<'_, AppState>,
    paths: Vec<String>,
    target: String,
    output_folder: Option<String>,
) -> Result<Snapshot, String> {
    let manager = state.converter_manager.clone();
    let config = state.settings_manager.get_settings().converter;
    tauri::async_runtime::spawn_blocking(move || {
        manager.convert(&app, &config, paths, target, output_folder)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn cancel_conversion(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    state.converter_manager.cancel(&job_id)
}

#[tauri::command]
pub fn get_conversion_state(state: State<'_, AppState>) -> Snapshot {
    state.converter_manager.snapshot()
}

#[tauri::command]
pub async fn open_conversion_result(
    state: State<'_, AppState>,
    path: String,
    folder: bool,
) -> Result<(), String> {
    let manager = state.converter_manager.clone();
    tauri::async_runtime::spawn_blocking(move || manager.open_result(&path, folder))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn pick_conversion_path(kind: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::integrations::converter::picker::pick(&kind)
    })
    .await
    .map_err(|e| e.to_string())?
}
