use serde::{Deserialize, Serialize};
use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition};

pub const COMPACT_WINDOW_WIDTH: f64 = 332.0;
pub const COMPACT_WINDOW_HEIGHT: f64 = 50.0;

pub const EXPANDED_WINDOW_WIDTH: f64 = 552.0;
pub const EXPANDED_WINDOW_HEIGHT: f64 = 88.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub name: Option<String>,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
    pub is_primary: bool,
}

/// Returns the logical (width, height) for a given notch shell state.
pub fn get_dimensions_for_state(state: &str) -> (f64, f64) {
    match state {
        "expanded" => (EXPANDED_WINDOW_WIDTH, EXPANDED_WINDOW_HEIGHT),
        _ => (COMPACT_WINDOW_WIDTH, COMPACT_WINDOW_HEIGHT),
    }
}

/// Locates the requested monitor by name, or falls back to primary / current monitor.
pub fn find_target_monitor(
    window: &tauri::Window,
    target_monitor_name: Option<&str>,
) -> Result<tauri::Monitor, Box<dyn std::error::Error>> {
    let monitors = window.available_monitors()?;

    if let Some(name) = target_monitor_name {
        if let Some(m) = monitors.iter().find(|m| m.name().map_or(false, |n| n == name)) {
            return Ok(m.clone());
        }
    }

    if let Some(current) = window.current_monitor()? {
        return Ok(current);
    }

    if let Some(primary) = window.primary_monitor()? {
        return Ok(primary);
    }

    if let Some(first) = monitors.into_iter().next() {
        return Ok(first);
    }

    Err("No active monitor detected on system".into())
}

/// Repositions and resizes the notch window at the top-center edge of the designated monitor,
/// respecting Windows DPI scaling and multi-monitor offsets.
pub fn reposition_notch_window(
    window: &tauri::Window,
    logical_width: f64,
    logical_height: f64,
    target_monitor_name: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let monitor = find_target_monitor(window, target_monitor_name)?;
    let monitor_pos = monitor.position();
    let monitor_size = monitor.size();
    let scale_factor = monitor.scale_factor();

    let physical_width = (logical_width * scale_factor).round() as i32;
    let center_x = monitor_pos.x + ((monitor_size.width as i32 - physical_width) / 2);
    let top_y = monitor_pos.y;

    window.set_size(LogicalSize::new(logical_width, logical_height))?;
    window.set_position(PhysicalPosition::new(center_x, top_y))?;

    Ok(())
}

/// Applies a shell state to the window: resizes logical bounds and recenters at top edge.
pub fn apply_notch_state(
    window: &tauri::Window,
    state: &str,
    target_monitor_name: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = get_dimensions_for_state(state);
    reposition_notch_window(window, width, height, target_monitor_name)
}

/// Sets up the initial compact notch window on application startup.
pub fn setup_notch_window(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window("main") {
        let (width, height) = get_dimensions_for_state("compact");
        let win = window.as_ref().window();
        reposition_notch_window(&win, width, height, None)?;
    }
    Ok(())
}
