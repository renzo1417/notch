use crate::integrations::media::MediaState;
use crate::state::AppState;
use tauri::State;

/// Retrieves the current active media state from the GSMTC integration.
#[tauri::command]
pub fn get_media_state(state: State<'_, AppState>) -> Result<MediaState, String> {
    Ok(state.media_manager.get_state())
}

/// Toggles playback (play/pause) on the current Windows media session.
#[tauri::command]
pub fn media_toggle_play_pause(state: State<'_, AppState>) -> Result<(), String> {
    state.media_manager.toggle_play_pause()
}

/// Skips to the previous track on the current Windows media session.
#[tauri::command]
pub fn media_previous(state: State<'_, AppState>) -> Result<(), String> {
    state.media_manager.skip_previous()
}

/// Skips to the next track on the current Windows media session.
#[tauri::command]
pub fn media_next(state: State<'_, AppState>) -> Result<(), String> {
    state.media_manager.skip_next()
}
