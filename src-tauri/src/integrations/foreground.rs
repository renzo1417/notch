//! Foreground Process Integration placeholder module.
//!
//! Reserved for active IDE/coding process detection in Phase 4.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForegroundApp {
    pub process_name: String,
    pub window_title: String,
    pub is_editor: bool,
}
