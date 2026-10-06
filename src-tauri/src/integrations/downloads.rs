//! Downloads Integration placeholder module.
//!
//! Reserved for filesystem watched folder download detection in Phase 8.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadState {
    pub file_name: String,
    pub progress_percent: Option<u8>,
    pub is_completed: bool,
}
