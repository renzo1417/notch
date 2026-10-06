use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchableApp {
    pub id: String,
    pub name: String,
    pub executable_path: String,
    pub args: Vec<String>,
    pub icon_path: Option<String>,
    pub icon_data_url: Option<String>,
    pub enabled: bool,
    pub order: u32,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectedApp {
    pub name: String,
    pub executable_path: String,
    pub args: Vec<String>,
    pub icon_data_url: Option<String>,
    pub is_shortcut: bool,
}

#[cfg(target_os = "windows")]
pub fn resolve_shortcut(lnk_path: &str) -> Option<(String, Vec<String>)> {
    use windows::core::{Interface, HSTRING};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED, IPersistFile, STGM_READ,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let shell_link: IShellLinkW = match CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) {
            Ok(s) => s,
            Err(_) => {
                CoUninitialize();
                return None;
            }
        };

        let persist_file: IPersistFile = match shell_link.cast() {
            Ok(p) => p,
            Err(_) => {
                CoUninitialize();
                return None;
            }
        };

        let wide_path = HSTRING::from(lnk_path);
        if persist_file.Load(&wide_path, STGM_READ).is_err() {
            CoUninitialize();
            return None;
        }

        let mut path_buf = [0u16; 1024];
        let _ = shell_link.GetPath(&mut path_buf, std::ptr::null_mut(), 0);
        let resolved_path = String::from_utf16_lossy(&path_buf);
        let trimmed_path = resolved_path.trim_matches('\0').trim().to_string();

        let mut args_buf = [0u16; 1024];
        let _ = shell_link.GetArguments(&mut args_buf);
        let args_str = String::from_utf16_lossy(&args_buf);
        let trimmed_args = args_str.trim_matches('\0').trim();

        let args = if trimmed_args.is_empty() {
            Vec::new()
        } else {
            trimmed_args.split_whitespace().map(|s| s.to_string()).collect()
        };

        CoUninitialize();

        if !trimmed_path.is_empty() {
            Some((trimmed_path, args))
        } else {
            None
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn resolve_shortcut(_lnk_path: &str) -> Option<(String, Vec<String>)> {
    None
}

#[cfg(target_os = "windows")]
pub fn extract_icon_data_url(path_str: &str) -> Option<String> {
    use windows::core::HSTRING;
    use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};

    let wide_path = HSTRING::from(path_str);
    let mut sfi = SHFILEINFOW::default();
    let res = unsafe {
        SHGetFileInfoW(
            &wide_path,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut sfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };

    if res != 0 && !sfi.hIcon.is_invalid() {
        unsafe { hicon_to_base64_bmp(sfi.hIcon) }
    } else {
        None
    }
}

#[cfg(not(target_os = "windows"))]
pub fn extract_icon_data_url(_path_str: &str) -> Option<String> {
    None
}

#[cfg(target_os = "windows")]
pub unsafe fn hicon_to_base64_bmp(hicon: windows::Win32::UI::WindowsAndMessaging::HICON) -> Option<String> {
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, GetDIBits,
        ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        DestroyIcon, DrawIconEx, GetIconInfo, DI_NORMAL, ICONINFO,
    };

    let mut icon_info = ICONINFO::default();
    if GetIconInfo(hicon, &mut icon_info).is_err() {
        let _ = DestroyIcon(hicon);
        return None;
    }

    let hdc_screen = GetDC(None);
    let hdc_mem = CreateCompatibleDC(Some(hdc_screen));

    let width = 32i32;
    let height = 32i32;

    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height, // negative for top-down DIB
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let mut pixel_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let hbm_dib = match CreateDIBSection(
        Some(hdc_mem),
        &bmi,
        DIB_RGB_COLORS,
        &mut pixel_ptr,
        None,
        0,
    ) {
        Ok(bm) => bm,
        Err(_) => {
            let _ = DeleteDC(hdc_mem);
            let _ = ReleaseDC(None, hdc_screen);
            let _ = DestroyIcon(hicon);
            return None;
        }
    };

    let old_bm = SelectObject(hdc_mem, hbm_dib.into());

    let _ = DrawIconEx(
        hdc_mem,
        0,
        0,
        hicon,
        width,
        height,
        0,
        None,
        DI_NORMAL,
    );

    let mut pixels = vec![0u8; (width * height * 4) as usize];
    let lines = GetDIBits(
        hdc_mem,
        hbm_dib,
        0,
        height as u32,
        Some(pixels.as_mut_ptr() as *mut std::ffi::c_void),
        &mut bmi,
        DIB_RGB_COLORS,
    );

    let _ = SelectObject(hdc_mem, old_bm);
    let _ = DeleteObject(hbm_dib.into());
    let _ = DeleteDC(hdc_mem);
    let _ = ReleaseDC(None, hdc_screen);
    if !icon_info.hbmColor.is_invalid() {
        let _ = DeleteObject(icon_info.hbmColor.into());
    }
    if !icon_info.hbmMask.is_invalid() {
        let _ = DeleteObject(icon_info.hbmMask.into());
    }
    let _ = DestroyIcon(hicon);

    if lines == 0 {
        return None;
    }

    let has_non_zero_alpha = pixels.chunks_exact(4).any(|p| p[3] != 0);
    if !has_non_zero_alpha {
        for chunk in pixels.chunks_exact_mut(4) {
            if chunk[0] != 0 || chunk[1] != 0 || chunk[2] != 0 {
                chunk[3] = 255;
            }
        }
    }

    let file_header_size = 14u32;
    let info_header_size = 40u32;
    let offset_data = file_header_size + info_header_size;
    let file_size = offset_data + pixels.len() as u32;

    let mut bmp_data = Vec::with_capacity(file_size as usize);
    bmp_data.extend_from_slice(b"BM");
    bmp_data.extend_from_slice(&file_size.to_le_bytes());
    bmp_data.extend_from_slice(&0u32.to_le_bytes());
    bmp_data.extend_from_slice(&offset_data.to_le_bytes());
    bmp_data.extend_from_slice(&40u32.to_le_bytes());
    bmp_data.extend_from_slice(&width.to_le_bytes());
    bmp_data.extend_from_slice(&(-height).to_le_bytes());
    bmp_data.extend_from_slice(&1u16.to_le_bytes());
    bmp_data.extend_from_slice(&32u16.to_le_bytes());
    bmp_data.extend_from_slice(&0u32.to_le_bytes());
    bmp_data.extend_from_slice(&(pixels.len() as u32).to_le_bytes());
    bmp_data.extend_from_slice(&2835u32.to_le_bytes());
    bmp_data.extend_from_slice(&2835u32.to_le_bytes());
    bmp_data.extend_from_slice(&0u32.to_le_bytes());
    bmp_data.extend_from_slice(&0u32.to_le_bytes());
    bmp_data.extend_from_slice(&pixels);

    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&bmp_data);
    Some(format!("data:image/bmp;base64,{}", encoded))
}

#[cfg(target_os = "windows")]
pub fn pick_executable_or_shortcut() -> Result<Option<String>, String> {
    std::thread::spawn(|| {
        use windows::core::{HSTRING, PCWSTR};
        use windows::Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
            COINIT_APARTMENTTHREADED,
        };
        use windows::Win32::UI::Shell::{
            Common::COMDLG_FILTERSPEC, FileOpenDialog, IFileOpenDialog, SIGDN_FILESYSPATH,
        };

        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

            let dialog: IFileOpenDialog =
                match CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER) {
                    Ok(d) => d,
                    Err(e) => {
                        CoUninitialize();
                        return Err(format!("Failed to initialize FileOpenDialog: {}", e));
                    }
                };

            let title = HSTRING::from("Select Application or Shortcut");
            let _ = dialog.SetTitle(&title);

            let filter_name = HSTRING::from("Applications and Shortcuts (*.exe; *.lnk)");
            let filter_spec = HSTRING::from("*.exe;*.lnk");
            let filters = [COMDLG_FILTERSPEC {
                pszName: PCWSTR(filter_name.as_ptr()),
                pszSpec: PCWSTR(filter_spec.as_ptr()),
            }];
            let _ = dialog.SetFileTypes(&filters);

            if dialog.Show(None).is_err() {
                CoUninitialize();
                return Ok(None);
            }

            let item = match dialog.GetResult() {
                Ok(item) => item,
                Err(e) => {
                    CoUninitialize();
                    return Err(format!("Failed to retrieve file selection: {}", e));
                }
            };

            let path_pwstr = match item.GetDisplayName(SIGDN_FILESYSPATH) {
                Ok(p) => p,
                Err(e) => {
                    CoUninitialize();
                    return Err(format!("Failed to retrieve file path: {}", e));
                }
            };

            let path_str = path_pwstr.to_string().map_err(|e| e.to_string())?;
            CoTaskMemFree(Some(path_pwstr.as_ptr() as *const _));

            CoUninitialize();
            Ok(Some(path_str))
        }
    })
    .join()
    .map_err(|_| "Dialog thread panicked".to_string())?
}

#[cfg(not(target_os = "windows"))]
pub fn pick_executable_or_shortcut() -> Result<Option<String>, String> {
    Err("File picker is only supported on Windows".to_string())
}

/// Central state manager for configured launcher applications.
pub struct LauncherManager {
    apps: Mutex<Vec<LaunchableApp>>,
    storage_path: Mutex<Option<PathBuf>>,
}

impl LauncherManager {
    pub fn new() -> Self {
        Self {
            apps: Mutex::new(Vec::new()),
            storage_path: Mutex::new(None),
        }
    }

    /// Initializes storage path and loads existing apps from local disk.
    pub fn init(&self, data_dir: PathBuf) {
        let _ = std::fs::create_dir_all(&data_dir);
        let file_path = data_dir.join("launcher_apps.json");
        *self.storage_path.lock().unwrap() = Some(file_path.clone());

        if file_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                if let Ok(mut loaded_apps) = serde_json::from_str::<Vec<LaunchableApp>>(&content) {
                    for app in loaded_apps.iter_mut() {
                        app.exists = Path::new(&app.executable_path).exists();
                        // Lazy extract icon if missing and file exists
                        if app.icon_data_url.is_none() && app.exists {
                            app.icon_data_url = extract_icon_data_url(&app.executable_path);
                        }
                    }
                    loaded_apps.sort_by_key(|a| a.order);
                    *self.apps.lock().unwrap() = loaded_apps;
                }
            }
        }
    }

    /// Saves the current list of apps to disk.
    fn save_to_disk(&self, apps: &[LaunchableApp]) -> Result<(), String> {
        let path_guard = self.storage_path.lock().unwrap();
        if let Some(ref path) = *path_guard {
            let json = serde_json::to_string_pretty(apps)
                .map_err(|e| format!("Failed to serialize launcher apps: {}", e))?;
            std::fs::write(path, json)
                .map_err(|e| format!("Failed to write launcher apps to disk: {}", e))?;
        }
        Ok(())
    }

    /// Returns all registered apps with freshly verified existence checks.
    pub fn get_apps(&self) -> Vec<LaunchableApp> {
        let mut apps = self.apps.lock().unwrap();
        for app in apps.iter_mut() {
            app.exists = Path::new(&app.executable_path).exists();
        }
        apps.sort_by_key(|a| a.order);
        apps.clone()
    }

    /// Inspects an executable or shortcut path, extracting target information and icon.
    pub fn inspect_file(&self, path_str: &str) -> Result<InspectedApp, String> {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err("Selected file does not exist on disk".to_string());
        }

        let is_lnk = path_str.to_lowercase().ends_with(".lnk");
        let (exec_path, args) = if is_lnk {
            if let Some((target_exe, target_args)) = resolve_shortcut(path_str) {
                (target_exe, target_args)
            } else {
                (path_str.to_string(), Vec::new())
            }
        } else {
            (path_str.to_string(), Vec::new())
        };

        // Prefer shortcut icon, fallback to target executable icon
        let icon_data_url = extract_icon_data_url(path_str)
            .or_else(|| extract_icon_data_url(&exec_path));

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("App")
            .to_string();

        Ok(InspectedApp {
            name,
            executable_path: exec_path,
            args,
            icon_data_url,
            is_shortcut: is_lnk,
        })
    }

    /// Adds a new application to the launcher list.
    pub fn add_app(&self, mut app: LaunchableApp) -> Result<LaunchableApp, String> {
        if app.executable_path.trim().is_empty() {
            return Err("Executable path cannot be empty".to_string());
        }

        app.exists = Path::new(&app.executable_path).exists();
        if app.icon_data_url.is_none() && app.exists {
            app.icon_data_url = extract_icon_data_url(&app.executable_path);
        }

        let mut apps = self.apps.lock().unwrap();
        app.order = apps.len() as u32;
        apps.push(app.clone());
        self.save_to_disk(&apps)?;

        Ok(app)
    }

    /// Updates an existing application in the launcher list.
    pub fn update_app(&self, mut updated: LaunchableApp) -> Result<LaunchableApp, String> {
        let mut apps = self.apps.lock().unwrap();
        let pos = apps
            .iter()
            .position(|a| a.id == updated.id)
            .ok_or_else(|| format!("App with id '{}' not found", updated.id))?;

        updated.exists = Path::new(&updated.executable_path).exists();
        if updated.icon_data_url.is_none() && updated.exists {
            updated.icon_data_url = extract_icon_data_url(&updated.executable_path);
        }

        apps[pos] = updated.clone();
        self.save_to_disk(&apps)?;

        Ok(updated)
    }

    /// Removes an application by ID.
    pub fn remove_app(&self, id: &str) -> Result<(), String> {
        let mut apps = self.apps.lock().unwrap();
        let initial_len = apps.len();
        apps.retain(|a| a.id != id);

        if apps.len() == initial_len {
            return Err(format!("App with id '{}' not found", id));
        }

        // Re-index orders
        for (idx, app) in apps.iter_mut().enumerate() {
            app.order = idx as u32;
        }

        self.save_to_disk(&apps)?;
        Ok(())
    }

    /// Reorders applications according to an explicit ordered list of IDs.
    pub fn reorder_apps(&self, ids: &[String]) -> Result<Vec<LaunchableApp>, String> {
        let mut apps = self.apps.lock().unwrap();
        for (idx, id) in ids.iter().enumerate() {
            if let Some(app) = apps.iter_mut().find(|a| &a.id == id) {
                app.order = idx as u32;
            }
        }
        apps.sort_by_key(|a| a.order);
        self.save_to_disk(&apps)?;
        Ok(apps.clone())
    }

    /// Relocates a missing or moved application to a new valid executable/shortcut path.
    pub fn locate_app(&self, id: &str, new_path: &str) -> Result<LaunchableApp, String> {
        let path = Path::new(new_path);
        if !path.exists() {
            return Err("Target path does not exist on disk".to_string());
        }

        let is_lnk = new_path.to_lowercase().ends_with(".lnk");
        let (exec_path, args) = if is_lnk {
            if let Some((target_exe, target_args)) = resolve_shortcut(new_path) {
                (target_exe, target_args)
            } else {
                (new_path.to_string(), Vec::new())
            }
        } else {
            (new_path.to_string(), Vec::new())
        };

        let icon = extract_icon_data_url(new_path)
            .or_else(|| extract_icon_data_url(&exec_path));

        let mut apps = self.apps.lock().unwrap();
        let app = apps
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or_else(|| format!("App with id '{}' not found", id))?;

        app.executable_path = exec_path;
        if !args.is_empty() {
            app.args = args;
        }
        app.exists = true;
        if icon.is_some() {
            app.icon_data_url = icon;
        }

        let updated = app.clone();
        self.save_to_disk(&apps)?;
        Ok(updated)
    }

    /// Safely launches an application through Rust standard process creation.
    /// Never passes strings through cmd.exe or PowerShell.
    pub fn launch_app(&self, id: &str) -> Result<(), String> {
        let app = {
            let apps = self.apps.lock().unwrap();
            apps.iter()
                .find(|a| a.id == id)
                .cloned()
                .ok_or_else(|| format!("App with id '{}' not found", id))?
        };

        let path = Path::new(&app.executable_path);
        if !path.exists() {
            // Update cached exists state to false
            let mut apps = self.apps.lock().unwrap();
            if let Some(a) = apps.iter_mut().find(|a| a.id == id) {
                a.exists = false;
            }
            return Err(format!(
                "AppNotFound: Executable path '{}' does not exist",
                app.executable_path
            ));
        }

        // Spawn process directly via OS process creation (no shell invocation)
        let mut cmd = std::process::Command::new(path);
        cmd.args(&app.args);

        // Set working directory to application's parent folder if available
        if let Some(parent) = path.parent() {
            cmd.current_dir(parent);
        }

        cmd.spawn()
            .map_err(|e| format!("Failed to spawn process '{}': {}", app.executable_path, e))?;

        Ok(())
    }
}

