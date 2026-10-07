use tauri::State;

use crate::integrations::settings::SettingsConfig;
use crate::integrations::startup;
use crate::state::AppState;

/// Retrieves the current application settings.
#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<SettingsConfig, String> {
    Ok(state.settings_manager.get_settings())
}

/// Saves and applies new application settings without restarting where practical.
#[tauri::command]
pub fn save_settings(
    state: State<'_, AppState>,
    settings: SettingsConfig,
) -> Result<SettingsConfig, String> {
    state.settings_manager.save_settings(settings)
}

/// Resets application settings to their safe default values.
#[tauri::command]
pub fn reset_settings(state: State<'_, AppState>) -> Result<SettingsConfig, String> {
    state.settings_manager.reset_to_defaults()
}

/// Checks whether the application is registered in Windows startup.
#[tauri::command]
pub fn get_autostart_status() -> Result<bool, String> {
    Ok(startup::is_autostart_enabled())
}

/// Configures launch at Windows startup.
#[tauri::command]
pub fn set_autostart_status(enabled: bool) -> Result<bool, String> {
    startup::set_autostart_enabled(enabled)
}
