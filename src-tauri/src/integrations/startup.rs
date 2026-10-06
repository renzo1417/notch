//! Windows Startup Integration placeholder module.
//!
//! Reserved for launch-at-startup registry/task integration in Phase 9.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupConfig {
    pub launch_at_startup: bool,
}
