use super::{extract_icon_data_url, LaunchTarget};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledApp {
    pub id: String,
    pub name: String,
    pub source: String,
    pub target: LaunchTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledAppCatalog {
    pub apps: Vec<InstalledApp>,
    pub warnings: Vec<String>,
}

pub fn validate_target(target: &LaunchTarget) -> Result<(), String> {
    match target {
        LaunchTarget::Executable { path, .. }
            if path.to_ascii_lowercase().ends_with(".exe") && Path::new(path).is_absolute() =>
        {
            Ok(())
        }
        LaunchTarget::Shortcut { path }
            if path.to_ascii_lowercase().ends_with(".lnk") && Path::new(path).is_absolute() =>
        {
            Ok(())
        }
        LaunchTarget::Packaged { aumid }
            if !aumid.trim().is_empty()
                && aumid.contains('!')
                && !aumid.contains(['\\', '/', '\0']) =>
        {
            Ok(())
        }
        _ => Err("Choose a Windows executable, shortcut, or installed application".into()),
    }
}

pub fn target_exists(target: &LaunchTarget) -> bool {
    match target {
        LaunchTarget::Executable { path, .. } | LaunchTarget::Shortcut { path } => {
            Path::new(path).is_file()
        }
        LaunchTarget::Packaged { aumid } => packaged_exists(aumid),
    }
}

fn collect_shortcuts(
    directory: &Path,
    depth: u8,
    source: &str,
    apps: &mut Vec<InstalledApp>,
    warnings: &mut Vec<String>,
) {
    if depth > 16 {
        return;
    }
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(_) => {
            warnings.push(
                "Some Start Menu shortcuts could not be read. You can still browse for an app."
                    .into(),
            );
            return;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(metadata) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::fs::MetadataExt;
            if metadata.file_attributes() & 0x400 != 0 {
                continue;
            }
        }
        if metadata.is_dir() {
            let nested_source = format!("{source} / {}", entry.file_name().to_string_lossy());
            collect_shortcuts(&path, depth + 1, &nested_source, apps, warnings);
            continue;
        }
        if !path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("lnk"))
        {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.to_ascii_lowercase().starts_with("uninstall") {
            continue;
        }
        let target = LaunchTarget::Shortcut {
            path: path.to_string_lossy().into_owned(),
        };
        apps.push(InstalledApp {
            id: target.identity(),
            name: name.into(),
            source: source.into(),
            target,
        });
    }
}

#[cfg(target_os = "windows")]
pub(super) struct ComApartment;

#[cfg(target_os = "windows")]
impl ComApartment {
    pub(super) fn new() -> Result<Self, String> {
        unsafe {
            windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_APARTMENTTHREADED,
            )
            .ok()
            .map_err(|error| format!("Windows application services are unavailable: {error}"))?;
        }
        Ok(Self)
    }
}

#[cfg(target_os = "windows")]
impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe {
            windows::Win32::System::Com::CoUninitialize();
        }
    }
}

#[cfg(target_os = "windows")]
pub fn discover_installed_apps() -> Result<InstalledAppCatalog, String> {
    use windows::core::HSTRING;
    use windows::Win32::System::Com::CoTaskMemFree;
    use windows::Win32::UI::Shell::{
        BHID_EnumItems, FOLDERID_CommonPrograms, FOLDERID_Programs, IEnumShellItems, IShellItem,
        SHCreateItemFromParsingName, SHGetKnownFolderPath, KF_FLAG_DEFAULT,
        SIGDN_DESKTOPABSOLUTEPARSING, SIGDN_NORMALDISPLAY,
    };
    let _apartment = ComApartment::new()?;
    let mut apps = Vec::new();
    let mut warnings = Vec::new();
    unsafe {
        for (folder, source) in [
            (&FOLDERID_Programs, "Start Menu · Your apps"),
            (&FOLDERID_CommonPrograms, "Start Menu · All users"),
        ] {
            match SHGetKnownFolderPath(folder, KF_FLAG_DEFAULT, None) {
                Ok(path) => {
                    let text = path.to_string(); CoTaskMemFree(Some(path.as_ptr() as *const _));
                    if let Ok(text) = text { collect_shortcuts(Path::new(&text), 0, source, &mut apps, &mut warnings); }
                }
                Err(_) => warnings.push("A Start Menu location is unavailable. Use Refresh or Browse if an app is missing.".into()),
            }
        }
        let packaged = (|| -> windows::core::Result<()> {
            let folder: IShellItem =
                SHCreateItemFromParsingName(&HSTRING::from("shell:AppsFolder"), None)?;
            let items: IEnumShellItems = folder.BindToHandler(None, &BHID_EnumItems)?;
            loop {
                let mut next = [None];
                let mut fetched = 0;
                items.Next(&mut next, Some(&mut fetched))?;
                if fetched == 0 {
                    break;
                }
                let Some(item) = next[0].take() else {
                    continue;
                };
                let Ok(parsing) = item.GetDisplayName(SIGDN_DESKTOPABSOLUTEPARSING) else {
                    continue;
                };
                let parsing_text = parsing.to_string();
                CoTaskMemFree(Some(parsing.as_ptr() as *const _));
                let Ok(parsing_text) = parsing_text else {
                    continue;
                };
                let aumid = parsing_text.rsplit('\\').next().unwrap_or(&parsing_text);
                // Desktop entries are supplied as shortcuts; only packaged IDs use activation.
                if !aumid.contains('!') || aumid.contains(['\\', '/']) {
                    continue;
                }
                let Ok(name) = item.GetDisplayName(SIGDN_NORMALDISPLAY) else {
                    continue;
                };
                let name_text = name.to_string();
                CoTaskMemFree(Some(name.as_ptr() as *const _));
                let Ok(name_text) = name_text else {
                    continue;
                };
                let target = LaunchTarget::Packaged {
                    aumid: aumid.into(),
                };
                apps.push(InstalledApp {
                    id: target.identity(),
                    name: name_text,
                    source: "Windows app".into(),
                    target,
                });
            }
            Ok(())
        })();
        if packaged.is_err() {
            warnings.push("Windows apps could not all be listed. Start Menu shortcuts remain available; try Refresh.".into());
        }
    }
    let mut seen = HashSet::new();
    apps.retain(|app| seen.insert(app.id.clone()));
    apps.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then(a.id.cmp(&b.id))
    });
    warnings.sort();
    warnings.dedup();
    Ok(InstalledAppCatalog { apps, warnings })
}

#[cfg(not(target_os = "windows"))]
pub fn discover_installed_apps() -> Result<InstalledAppCatalog, String> {
    Err("Installed applications are available in the Windows desktop app".into())
}

#[cfg(target_os = "windows")]
fn packaged_exists(aumid: &str) -> bool {
    use windows::core::HSTRING;
    use windows::Win32::UI::Shell::{IShellItem, SHCreateItemFromParsingName};
    let Ok(_apartment) = ComApartment::new() else {
        return false;
    };
    let item = unsafe {
        SHCreateItemFromParsingName::<_, _, IShellItem>(
            &HSTRING::from(format!("shell:AppsFolder\\{aumid}")),
            None,
        )
    };
    // Bind the COM result so its item releases before the apartment guard (Rust 2021).
    item.is_ok()
}

#[cfg(not(target_os = "windows"))]
fn packaged_exists(_aumid: &str) -> bool {
    false
}

#[cfg(target_os = "windows")]
pub fn launch_target(target: &LaunchTarget) -> Result<(), String> {
    use windows::core::HSTRING;
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
    use windows::Win32::UI::Shell::{
        ApplicationActivationManager, IApplicationActivationManager, ShellExecuteExW, AO_NONE,
        SEE_MASK_NOASYNC, SHELLEXECUTEINFOW,
    };
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    match target {
        LaunchTarget::Executable {
            path,
            args,
            working_directory,
        } => {
            let mut command = std::process::Command::new(path);
            command.args(args);
            if let Some(directory) = working_directory
                .as_deref()
                .map(Path::new)
                .or_else(|| Path::new(path).parent())
            {
                command.current_dir(directory);
            }
            command
                .spawn()
                .map_err(|error| format!("Couldn’t launch this application: {error}"))?;
        }
        LaunchTarget::Shortcut { path } => {
            let _apartment = ComApartment::new()?;
            let file = HSTRING::from(path);
            let verb = HSTRING::from("open");
            let mut info = SHELLEXECUTEINFOW {
                cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
                fMask: SEE_MASK_NOASYNC,
                lpVerb: windows::core::PCWSTR(verb.as_ptr()),
                lpFile: windows::core::PCWSTR(file.as_ptr()),
                nShow: SW_SHOWNORMAL.0,
                ..Default::default()
            };
            unsafe {
                ShellExecuteExW(&mut info)
                    .map_err(|error| format!("Couldn’t open this shortcut: {error}"))?;
            }
        }
        LaunchTarget::Packaged { aumid } => {
            let _apartment = ComApartment::new()?;
            unsafe {
                let manager: IApplicationActivationManager =
                    CoCreateInstance(&ApplicationActivationManager, None, CLSCTX_INPROC_SERVER)
                        .map_err(|error| error.to_string())?;
                manager
                    .ActivateApplication(&HSTRING::from(aumid), &HSTRING::new(), AO_NONE)
                    .map_err(|error| {
                        format!(
                            "Couldn’t open this Windows app. It may have been uninstalled: {error}"
                        )
                    })?;
            }
        }
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn launch_target(_target: &LaunchTarget) -> Result<(), String> {
    Err("Launching applications requires Windows".into())
}

pub fn target_icon(target: &LaunchTarget) -> Option<String> {
    match target {
        LaunchTarget::Packaged { aumid } => packaged_icon(aumid),
        _ => extract_icon_data_url(target.file_path()?),
    }
}

#[cfg(target_os = "windows")]
fn packaged_icon(aumid: &str) -> Option<String> {
    use windows::core::{HSTRING, PCWSTR};
    use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
    use windows::Win32::System::Com::CoTaskMemFree;
    use windows::Win32::UI::Shell::{
        IShellItem, SHCreateItemFromParsingName, SHGetFileInfoW, SHGetIDListFromObject,
        SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGFI_PIDL,
    };
    let _apartment = ComApartment::new().ok()?;
    unsafe {
        let item: IShellItem =
            SHCreateItemFromParsingName(&HSTRING::from(format!("shell:AppsFolder\\{aumid}")), None)
                .ok()?;
        let pidl = SHGetIDListFromObject(&item).ok()?;
        let mut info = SHFILEINFOW::default();
        let found = SHGetFileInfoW(
            PCWSTR(pidl as *const u16),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_PIDL | SHGFI_LARGEICON,
        );
        CoTaskMemFree(Some(pidl as *const _));
        if found == 0 || info.hIcon.is_invalid() {
            None
        } else {
            super::hicon_to_base64_bmp(info.hIcon)
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn packaged_icon(_aumid: &str) -> Option<String> {
    None
}
