use std::{
    env,
    path::{Path, PathBuf},
};
use windows::{
    core::{HSTRING, PCWSTR},
    Win32::System::Registry::*,
};

pub fn executable(configured: Option<&str>) -> Option<PathBuf> {
    if let Some(path) = configured {
        return validated(Path::new(path));
    }
    let mut folders = Vec::new();
    if let Some(path) = env::var_os("PATH") {
        folders.extend(env::split_paths(&path));
    }
    for root in ["LOCALAPPDATA", "ProgramFiles", "ProgramFiles(x86)"] {
        if let Some(path) = env::var_os(root) {
            folders.push(PathBuf::from(&path).join("Filee"));
            folders.push(PathBuf::from(path).join("Programs").join("Filee"));
        }
    }
    folders.extend(installed_folders());
    for folder in folders {
        for candidate in [
            folder.join("filee-cli.exe"),
            folder.join("cli").join("filee-cli.exe"),
            folder.parent().unwrap_or(&folder).join("filee-cli.exe"),
        ] {
            if let Some(path) = validated(&candidate) {
                return Some(path);
            }
        }
    }
    None
}

fn validated(path: &Path) -> Option<PathBuf> {
    if !path.is_absolute()
        || !path
            .file_name()?
            .to_str()?
            .eq_ignore_ascii_case("filee-cli.exe")
    {
        return None;
    }
    let path = std::fs::canonicalize(path).ok()?;
    path.is_file().then_some(path)
}

struct Key(HKEY);
impl Drop for Key {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}
fn open(root: HKEY, path: &str) -> Option<Key> {
    let mut key = HKEY::default();
    unsafe {
        RegOpenKeyExW(root, &HSTRING::from(path), Some(0), KEY_READ, &mut key)
            .ok()
            .ok()?;
    }
    Some(Key(key))
}
fn string(key: HKEY, name: &str) -> Option<String> {
    let name = HSTRING::from(name);
    let mut buffer = [0u16; 4096];
    let mut size = size_of_val(&buffer) as u32;
    unsafe {
        RegGetValueW(
            key,
            PCWSTR::null(),
            &name,
            RRF_RT_REG_SZ,
            None,
            Some(buffer.as_mut_ptr().cast()),
            Some(&mut size),
        )
        .ok()
        .ok()?;
    }
    let count = (size as usize / 2).min(buffer.len());
    Some(
        String::from_utf16_lossy(&buffer[..count])
            .trim_end_matches('\0')
            .to_string(),
    )
}
fn installed_folders() -> Vec<PathBuf> {
    let mut folders = Vec::new();
    for root in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        for branch in [
            "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            "Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ] {
            let Some(key) = open(root, branch) else {
                continue;
            };
            for index in 0..2048 {
                let mut name = [0u16; 512];
                let mut size = name.len() as u32;
                let result = unsafe {
                    RegEnumKeyExW(
                        key.0,
                        index,
                        Some(windows::core::PWSTR(name.as_mut_ptr())),
                        &mut size,
                        None,
                        None,
                        None,
                        None,
                    )
                };
                if result != windows::Win32::Foundation::ERROR_SUCCESS {
                    break;
                }
                let name = String::from_utf16_lossy(&name[..size as usize]);
                let Some(app) = open(key.0, &name) else {
                    continue;
                };
                if string(app.0, "DisplayName")
                    .is_some_and(|name| name == "Filee" || name.starts_with("Filee "))
                {
                    if let Some(folder) = string(app.0, "InstallLocation").filter(|s| !s.is_empty())
                    {
                        folders.push(PathBuf::from(folder));
                    }
                }
            }
        }
    }
    folders
}
