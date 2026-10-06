//! Tauri commands for Battery and power status inspection.
//! Defined in PRODUCT_SPEC.md section 18 and Phase 7 requirements.

use crate::integrations::power::BatteryState;
use crate::state::AppState;
use tauri::State;

/// Retrieves the current system power status from Windows kernel.
#[tauri::command]
pub fn get_battery_state(state: State<'_, AppState>) -> Result<BatteryState, String> {
    Ok(state.power_manager.get_state())
}

/// Force-refreshes power status and notifies listeners if state changed.
#[tauri::command]
pub fn refresh_battery_state(state: State<'_, AppState>) -> Result<BatteryState, String> {
    Ok(state.power_manager.refresh_and_notify())
}
