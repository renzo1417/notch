//! Windows Startup Integration module.
//!
//! Controls launch-at-login via the Windows Current User Run key:
//! `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`

use serde::{Deserialize, Serialize};
use std::env;
use windows::core::w;
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};

const RUN_SUBKEY: windows::core::PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const APP_VALUE_NAME: windows::core::PCWSTR = w!("ProjectNotch");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupConfig {
    pub launch_at_startup: bool,
}

/// Checks whether the application is registered to launch at Windows startup.
pub fn is_autostart_enabled() -> bool {
    unsafe {
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_SUBKEY, None, KEY_READ, &mut hkey) != ERROR_SUCCESS {
            return false;
        }

        let mut data_size = 0u32;
        let query_res = RegQueryValueExW(
            hkey,
            APP_VALUE_NAME,
            None,
            None,
            None,
            Some(&mut data_size),
        );

        let _ = RegCloseKey(hkey);
        query_res == ERROR_SUCCESS && data_size > 0
    }
}

/// Enables or disables launch at Windows startup by writing to or removing from the Run registry key.
pub fn set_autostart_enabled(enabled: bool) -> Result<bool, String> {
    if enabled {
        let current_exe = env::current_exe()
            .map_err(|e| format!("Failed to determine current executable path: {e}"))?;

        let exe_str = current_exe.to_string_lossy();
        let quoted_cmd = format!("\"{}\"", exe_str);
        let wide_cmd: Vec<u16> = quoted_cmd.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(wide_cmd.as_ptr() as *const u8, wide_cmd.len() * 2)
        };

        unsafe {
            let mut hkey = HKEY::default();
            let open_res = RegOpenKeyExW(HKEY_CURRENT_USER, RUN_SUBKEY, None, KEY_WRITE, &mut hkey);
            if open_res != ERROR_SUCCESS {
                return Err(format!("Failed to open registry Run key: error code {:?}", open_res));
            }

            let set_res = RegSetValueExW(
                hkey,
                APP_VALUE_NAME,
                None,
                REG_SZ,
                Some(bytes),
            );

            let _ = RegCloseKey(hkey);

            if set_res != ERROR_SUCCESS {
                return Err(format!("Failed to write startup registry value: error code {:?}", set_res));
            }
        }
    } else {
        unsafe {
            let mut hkey = HKEY::default();
            let open_res = RegOpenKeyExW(HKEY_CURRENT_USER, RUN_SUBKEY, None, KEY_WRITE, &mut hkey);
            if open_res == ERROR_SUCCESS {
                let _ = RegDeleteValueW(hkey, APP_VALUE_NAME);
                let _ = RegCloseKey(hkey);
            }
        }
    }

    Ok(enabled)
}
