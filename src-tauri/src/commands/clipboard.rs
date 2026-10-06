//! Tauri commands for Clipboard history and native listener.
//! Defined in PRODUCT_SPEC.md section 17 and Phase 7 requirements.

use crate::integrations::clipboard::{self, ClipboardItem};
use crate::state::AppState;
use tauri::State;

/// Retrieves the current in-memory clipboard history.
#[tauri::command]
pub fn get_clipboard_history(state: State<'_, AppState>) -> Result<Vec<ClipboardItem>, String> {
    Ok(state.clipboard_manager.get_history())
}

/// Copies the given text back onto the native Windows clipboard.
#[tauri::command]
pub fn copy_to_clipboard(text: String) -> Result<(), String> {
    clipboard::write_clipboard_text(&text)
}

/// Clears in-memory clipboard history.
#[tauri::command]
pub fn clear_clipboard_history(state: State<'_, AppState>) -> Result<(), String> {
    state.clipboard_manager.clear_history();
    Ok(())
}

/// Toggles whether clipboard capture is paused.
#[tauri::command]
pub fn set_clipboard_paused(paused: bool, state: State<'_, AppState>) -> Result<(), String> {
    state.clipboard_manager.set_paused(paused);
    Ok(())
}

/// Returns whether clipboard capture is currently paused.
#[tauri::command]
pub fn is_clipboard_paused(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.clipboard_manager.is_paused())
}

/// Configures the maximum number of items in history (1..=100).
#[tauri::command]
pub fn set_clipboard_max_items(max_items: usize, state: State<'_, AppState>) -> Result<(), String> {
    state.clipboard_manager.set_max_items(max_items);
    Ok(())
}
