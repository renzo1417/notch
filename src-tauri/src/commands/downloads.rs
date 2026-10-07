use crate::integrations::downloads::{self, DownloadState, WatchedFolder};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn get_watched_folders(state: State<'_, AppState>) -> Result<Vec<WatchedFolder>, String> {
    Ok(state.downloads_manager.get_watched_folders())
}

#[tauri::command]
pub fn add_watched_folder(
    state: State<'_, AppState>,
    path: String,
) -> Result<WatchedFolder, String> {
    state.downloads_manager.add_watched_folder(&path)
}

#[tauri::command]
pub fn remove_watched_folder(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.downloads_manager.remove_watched_folder(&id)
}

#[tauri::command]
pub fn pick_download_folder() -> Result<Option<String>, String> {
    downloads::pick_folder()
}

#[tauri::command]
pub fn get_download_state(state: State<'_, AppState>) -> Result<Option<DownloadState>, String> {
    Ok(state.downloads_manager.get_current_state())
}

#[tauri::command]
pub fn open_download_folder(path: Option<String>) -> Result<(), String> {
    downloads::open_folder_or_select_file(path.as_deref())
}
