use serde::{Deserialize, Serialize};
use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition};

pub const COMPACT_WINDOW_WIDTH: f64 = 336.0;
pub const COMPACT_WINDOW_HEIGHT: f64 = 60.0;

pub const EXPANDED_WINDOW_WIDTH: f64 = 512.0;
pub const EXPANDED_WINDOW_HEIGHT: f64 = 176.0;

pub const APPS_WINDOW_HEIGHT: f64 = 344.0;
pub const FOCUS_WINDOW_HEIGHT: f64 = 344.0;
pub const CLIPBOARD_WINDOW_HEIGHT: f64 = 344.0;
pub const DOWNLOADS_WINDOW_HEIGHT: f64 = 344.0;

pub const LAUNCHER_WINDOW_WIDTH: f64 = 512.0;
pub const LAUNCHER_WINDOW_HEIGHT: f64 = 344.0;

pub const SETTINGS_WINDOW_WIDTH: f64 = 672.0;
pub const SETTINGS_WINDOW_HEIGHT: f64 = 472.0;

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
        "dormant" => (192.0, 48.0),
        "expanded" => (EXPANDED_WINDOW_WIDTH, EXPANDED_WINDOW_HEIGHT),
        "apps" | "launcher" => (EXPANDED_WINDOW_WIDTH, APPS_WINDOW_HEIGHT),
        "focus" | "timer" => (EXPANDED_WINDOW_WIDTH, FOCUS_WINDOW_HEIGHT),
        "clipboard" => (EXPANDED_WINDOW_WIDTH, CLIPBOARD_WINDOW_HEIGHT),
        "downloads" | "download" => (EXPANDED_WINDOW_WIDTH, DOWNLOADS_WINDOW_HEIGHT),
        "utilities" | "battery" | "panel" => (EXPANDED_WINDOW_WIDTH, APPS_WINDOW_HEIGHT),
        "settings" => (SETTINGS_WINDOW_WIDTH, SETTINGS_WINDOW_HEIGHT),
        _ => (COMPACT_WINDOW_WIDTH, COMPACT_WINDOW_HEIGHT),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drawer_aliases_and_shadow_bounds_match() {
        assert_eq!(get_dimensions_for_state("compact"), (336.0, 60.0));
        assert_eq!(get_dimensions_for_state("expanded"), (512.0, 176.0));
        assert_eq!(get_dimensions_for_state("settings"), (672.0, 472.0));
        for state in ["apps", "launcher", "timer", "focus", "download", "downloads", "clipboard", "utilities", "battery", "panel"] {
            assert_eq!(get_dimensions_for_state(state), (512.0, 344.0));
        }
    }

    #[test]
    fn bounds_fit_monitor_at_all_supported_scales() {
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let (width, height) = clamp_window_dimensions(672.0, 472.0, 1024, 768, scale);
            assert!(width * scale <= 1024.0);
            assert!(height * scale <= 768.0);
        }
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

/// Clamp in logical units so the shell and its shadow fit scaled displays.
pub fn clamp_window_dimensions(width: f64, height: f64, monitor_width: u32, monitor_height: u32, scale: f64) -> (f64, f64) {
    (width.min(monitor_width as f64 / scale), height.min(monitor_height as f64 / scale))
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

    let (logical_width, logical_height) = clamp_window_dimensions(
        logical_width, logical_height, monitor_size.width, monitor_size.height, scale_factor,
    );
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
