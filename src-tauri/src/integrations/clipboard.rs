//! Clipboard Integration placeholder module.
//!
//! Reserved for AddClipboardFormatListener / WM_CLIPBOARDUPDATE integration in Phase 7.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: String,
    pub text: String,
    pub timestamp_ms: u64,
}
