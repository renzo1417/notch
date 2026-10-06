//! Media Integration placeholder module.
//!
//! Reserved for Windows Media Session (GSMTC) integration in Phase 3.
//! Will use Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MediaState {
    pub is_playing: bool,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub position_ms: Option<u64>,
    pub duration_ms: Option<u64>,
}
