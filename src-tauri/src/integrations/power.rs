//! Power & Battery Integration placeholder module.
//!
//! Reserved for GetSystemPowerStatus integration in Phase 7.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BatteryState {
    pub percentage: u8,
    pub is_charging: bool,
    pub is_ac_connected: bool,
    pub remaining_seconds: Option<u32>,
}
