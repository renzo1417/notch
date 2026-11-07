use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

mod catalog;
pub use catalog::{InstalledApp, InstalledAppCatalog};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LaunchTarget {
    Executable {
        path: String,
        args: Vec<String>,
        #[serde(default, rename = "workingDirectory")]
        working_directory: Option<String>,
    },
    Shortcut {
        path: String,
    },
    Packaged {
        aumid: String,
    },
}

impl LaunchTarget {
    pub fn identity(&self) -> String {
        match self {
            Self::Executable {
                path,
                args,
                working_directory,
            } => format!(
                "exe:{}|{:?}|{:?}",
                path.replace('/', "\\").to_lowercase(),
                args,
                working_directory
            ),
            Self::Shortcut { path } => format!("lnk:{}", path.replace('/', "\\").to_lowercase()),
            Self::Packaged { aumid } => format!("app:{}", aumid.to_lowercase()),
        }
    }

    pub fn file_path(&self) -> Option<&str> {
        match self {
            Self::Executable { path, .. } | Self::Shortcut { path } => Some(path),
            Self::Packaged { .. } => None,
        }
    }
}

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<LaunchTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectedApp {
    pub name: String,
    pub executable_path: String,
    pub args: Vec<String>,
    pub icon_data_url: Option<String>,
    pub is_shortcut: bool,
    pub target: LaunchTarget,
}

#[cfg(target_os = "windows")]
pub fn extract_icon_data_url(path_str: &str) -> Option<String> {
    use windows::core::HSTRING;
    use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};

    let _apartment = catalog::ComApartment::new().ok()?;
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
pub unsafe fn hicon_to_base64_bmp(
    hicon: windows::Win32::UI::WindowsAndMessaging::HICON,
) -> Option<String> {
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC,
        SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
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
    let hbm_dib =
        match CreateDIBSection(Some(hdc_mem), &bmi, DIB_RGB_COLORS, &mut pixel_ptr, None, 0) {
            Ok(bm) => bm,
            Err(_) => {
                let _ = DeleteDC(hdc_mem);
                let _ = ReleaseDC(None, hdc_screen);
                let _ = DestroyIcon(hicon);
                if !icon_info.hbmColor.is_invalid() {
                    let _ = DeleteObject(icon_info.hbmColor.into());
                }
                if !icon_info.hbmMask.is_invalid() {
                    let _ = DeleteObject(icon_info.hbmMask.into());
                }
                return None;
            }
        };

    let old_bm = SelectObject(hdc_mem, hbm_dib.into());

    let _ = DrawIconEx(hdc_mem, 0, 0, hicon, width, height, 0, None, DI_NORMAL);

    let mut pixels = vec![0u8; (width * height * 4) as usize];
    // GetDIBits requires the bitmap not to be selected into a device context.
    let _ = SelectObject(hdc_mem, old_bm);
    let lines = GetDIBits(
        hdc_mem,
        hbm_dib,
        0,
        height as u32,
        Some(pixels.as_mut_ptr() as *mut std::ffi::c_void),
        &mut bmi,
        DIB_RGB_COLORS,
    );

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
        use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_INPROC_SERVER};
        use windows::Win32::UI::Shell::{
            Common::COMDLG_FILTERSPEC, FileOpenDialog, IFileOpenDialog, SIGDN_FILESYSPATH,
        };

        let _apartment = catalog::ComApartment::new()?;
        unsafe {
            let dialog: IFileOpenDialog =
                CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)
                    .map_err(|error| format!("Failed to initialize FileOpenDialog: {error}"))?;
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
                return Ok(None);
            }
            let item = dialog
                .GetResult()
                .map_err(|error| format!("Failed to retrieve file selection: {error}"))?;
            let path = item
                .GetDisplayName(SIGDN_FILESYSPATH)
                .map_err(|error| format!("Failed to retrieve file path: {error}"))?;
            let text = path.to_string();
            CoTaskMemFree(Some(path.as_ptr() as *const _));
            text.map(Some).map_err(|error| error.to_string())
        }
    })
    .join()
    .map_err(|_| "Dialog thread panicked".to_string())?
}

#[cfg(not(target_os = "windows"))]
pub fn pick_executable_or_shortcut() -> Result<Option<String>, String> {
    Err("File picker is only supported on Windows".to_string())
}

/// Persistent pins and an on-demand, process-local installed-app catalog.
pub struct LauncherManager {
    apps: Mutex<Vec<LaunchableApp>>,
    storage_path: Mutex<Option<PathBuf>>,
    storage_error: Mutex<Option<String>>,
    catalog: Mutex<Option<InstalledAppCatalog>>,
    icons: Mutex<HashMap<String, Option<String>>>,
}

mod manager;
