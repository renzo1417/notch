use tauri::{AppHandle, Manager, PhysicalPosition};

/// Sets up the initial notch window positioning at the top-center of the primary monitor.
pub fn setup_notch_window(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window("main") {
        if let Ok(Some(monitor)) = window.primary_monitor() {
            let monitor_size = monitor.size();
            let scale_factor = monitor.scale_factor();
            let window_width = (560.0 * scale_factor) as i32;
            let center_x = (monitor_size.width as i32 - window_width) / 2;
            
            // Position flush to top edge (y = 0)
            let _ = window.set_position(PhysicalPosition::new(center_x, 0));
        }
    }
    Ok(())
}
