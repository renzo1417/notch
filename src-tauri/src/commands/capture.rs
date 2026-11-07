use crate::integrations::capture::{CaptureResult, CaptureTargets};
use crate::state::AppState;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn list_capture_targets() -> Result<CaptureTargets, String> {
    tauri::async_runtime::spawn_blocking(crate::integrations::capture::list_targets)
        .await
        .map_err(|_| "Could not enumerate screenshot targets.".to_string())?
}

#[tauri::command]
pub async fn start_capture(
    app: AppHandle,
    state: State<'_, AppState>,
    mode: String,
    target_id: String,
    request_id: String,
) -> Result<CaptureResult, String> {
    let manager = state.capture_manager.clone();
    tauri::async_runtime::spawn_blocking(move || {
        manager.capture(&app, &mode, &target_id, &request_id)
    })
    .await
    .map_err(|_| "The screenshot operation stopped unexpectedly.".to_string())?
}

#[tauri::command]
pub fn cancel_capture(state: State<'_, AppState>, request_id: String) -> Result<(), String> {
    state.capture_manager.cancel(&request_id)
}
