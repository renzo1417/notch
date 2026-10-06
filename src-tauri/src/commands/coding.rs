//! Tauri commands for Coding context and foreground application detection.
//! Defined in PRODUCT_SPEC.md section 14.

use crate::integrations::foreground::CodingSession;
use crate::state::AppState;
use tauri::State;

/// Retrieves the current foreground coding session.
#[tauri::command]
pub fn get_coding_session(state: State<'_, AppState>) -> Result<CodingSession, String> {
    Ok(state.foreground_manager.get_session())
}

/// Allows the user to configure an optional custom project/session label.
#[tauri::command]
pub fn set_coding_project_label(
    label: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.foreground_manager.set_user_project_label(label);
    Ok(())
}

/// Resets the active coding session elapsed timer.
#[tauri::command]
pub fn reset_coding_session(state: State<'_, AppState>) -> Result<(), String> {
    state.foreground_manager.reset_session();
    Ok(())
}
