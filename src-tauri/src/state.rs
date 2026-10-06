use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use crate::integrations::clipboard::ClipboardManager;
use crate::integrations::foreground::ForegroundManager;
use crate::integrations::launcher::LauncherManager;
use crate::integrations::media::MediaManager;
use crate::integrations::power::PowerManager;

/// Runtime configuration for the notch shell window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    pub always_on_top: bool,
    pub current_state: String,
    pub selected_monitor: Option<String>,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            always_on_top: true,
            current_state: "compact".to_string(),
            selected_monitor: None,
        }
    }
}

/// Central backend state for Project Notch.
pub struct AppState {
    pub is_ready: Mutex<bool>,
    pub shell_config: Mutex<ShellConfig>,
    pub media_manager: Arc<MediaManager>,
    pub foreground_manager: Arc<ForegroundManager>,
    pub launcher_manager: Arc<LauncherManager>,
    pub clipboard_manager: Arc<ClipboardManager>,
    pub power_manager: Arc<PowerManager>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            is_ready: Mutex::new(true),
            shell_config: Mutex::new(ShellConfig::default()),
            media_manager: Arc::new(MediaManager::new()),
            foreground_manager: Arc::new(ForegroundManager::new()),
            launcher_manager: Arc::new(LauncherManager::new()),
            clipboard_manager: Arc::new(ClipboardManager::new()),
            power_manager: Arc::new(PowerManager::new()),
        }
    }
}
