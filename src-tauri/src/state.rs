use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use crate::integrations::media::MediaManager;

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
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            is_ready: Mutex::new(true),
            shell_config: Mutex::new(ShellConfig::default()),
            media_manager: Arc::new(MediaManager::new()),
        }
    }
}

