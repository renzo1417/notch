//! Power & Battery Integration module.
//!
//! Native Windows power state queries via GetSystemPowerStatus.
//! Defined in PRODUCT_SPEC.md section 18 and Phase 7 requirements.

use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

/// System battery and power status model.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BatteryState {
    pub has_battery: bool,
    pub percentage: u8,
    pub is_charging: bool,
    pub is_ac_connected: bool,
    pub remaining_seconds: Option<u32>,
}

/// Reads the current system power status from Windows kernel.
/// Never fabricates battery remaining time.
pub fn read_system_power_status() -> BatteryState {
    let mut status = SYSTEM_POWER_STATUS::default();
    let success = unsafe { GetSystemPowerStatus(&mut status) };

    if success.is_err() {
        return BatteryState {
            has_battery: false,
            percentage: 100,
            is_charging: false,
            is_ac_connected: true,
            remaining_seconds: None,
        };
    }

    // BatteryFlag: bit 7 (128) means "No system battery"
    // 255 means "Unknown status"
    let has_battery = status.BatteryFlag != 128 && status.BatteryFlag != 255;

    // ACLineStatus: 0 = offline (battery), 1 = online (AC), 255 = unknown
    let is_ac_connected = status.ACLineStatus == 1;

    // BatteryFlag bit 3 (8) means Charging
    let is_charging = (status.BatteryFlag & 8) != 0 || (is_ac_connected && status.BatteryLifePercent < 100);

    // BatteryLifePercent: 0..100, or 255 if unknown
    let percentage = if status.BatteryLifePercent <= 100 {
        status.BatteryLifePercent
    } else {
        100
    };

    // BatteryLifeTime: seconds of battery life remaining, or (DWORD)-1 (0xFFFFFFFF) if unknown
    // Spec: "Never fabricate remaining time." Only expose if Windows provides a valid value.
    let remaining_seconds = if has_battery && !is_ac_connected && status.BatteryLifeTime != u32::MAX && status.BatteryLifeTime > 0 {
        Some(status.BatteryLifeTime)
    } else {
        None
    };

    BatteryState {
        has_battery,
        percentage,
        is_charging,
        is_ac_connected,
        remaining_seconds,
    }
}

pub struct PowerManager {
    current_state: Mutex<BatteryState>,
    app_handle: Mutex<Option<AppHandle>>,
    is_running: AtomicBool,
}

impl PowerManager {
    pub fn new() -> Self {
        let initial = read_system_power_status();
        Self {
            current_state: Mutex::new(initial),
            app_handle: Mutex::new(None),
            is_running: AtomicBool::new(false),
        }
    }

    pub fn get_state(&self) -> BatteryState {
        self.current_state.lock().map(|s| s.clone()).unwrap_or_else(|_| read_system_power_status())
    }

    pub fn refresh_and_notify(&self) -> BatteryState {
        let new_state = read_system_power_status();
        let changed = {
            let mut guard = self.current_state.lock().unwrap();
            if *guard != new_state {
                *guard = new_state.clone();
                true
            } else {
                false
            }
        };

        if changed {
            if let Ok(guard) = self.app_handle.lock() {
                if let Some(ref handle) = *guard {
                    let _ = handle.emit("battery.changed", &new_state);
                }
            }
        }

        new_state
    }

    pub fn init(self: &Arc<Self>, app: AppHandle) {
        if self.is_running.swap(true, Ordering::SeqCst) {
            return;
        }

        {
            let mut handle_guard = self.app_handle.lock().unwrap();
            *handle_guard = Some(app);
        }

        let this = Arc::clone(self);
        thread::Builder::new()
            .name("power-monitor".to_string())
            .spawn(move || {
                while this.is_running.load(Ordering::Relaxed) {
                    this.refresh_and_notify();
                    // Power status changes slowly; 15s cadence gives prompt reaction with zero noticeable CPU
                    thread::sleep(Duration::from_secs(15));
                }
            })
            .expect("Failed to spawn power monitor thread");
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }
}

impl Default for PowerManager {
    fn default() -> Self {
        Self::new()
    }
}
