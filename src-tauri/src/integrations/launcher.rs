//! App Launcher Integration placeholder module.
//!
//! Reserved for secure application launching in Phase 6.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchableApp {
    pub id: String,
    pub name: String,
    pub executable_path: String,
    pub args: Vec<String>,
    pub icon_path: Option<String>,
    pub enabled: bool,
    pub order: u32,
}
