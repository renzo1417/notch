//! Explicit, one-shot screen capture. Pixels never leave native memory except for
//! ownership transfer to the Windows clipboard. No capture resources exist at idle.
mod pixels;
mod requests;
mod selection;
use pixels::{half_to_float, linear_to_sdr, Image};
use requests::{CaptureRequest, CaptureRequests};

use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};
use windows::core::{factory, Interface, BOOL};
use windows::Foundation::TypedEventHandler;
use windows::Graphics::Capture::{
    Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession,
};
use windows::Graphics::DirectX::{Direct3D11::IDirect3DDevice, DirectXPixelFormat};
use windows::Win32::Foundation::{GlobalFree, HANDLE, HMODULE, HWND, LPARAM, RECT, WPARAM};
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP};
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dwm::{DwmFlush, DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::Graphics::Dxgi::{IDXGIAdapter, IDXGIDevice};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, BITMAPV5HEADER, BI_BITFIELDS, HDC, HMONITOR, MONITORINFO,
    MONITORINFOEXW,
};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::System::WinRT::Direct3D11::{
    CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess,
};
use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;
use windows::Win32::System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED};
use windows::Win32::UI::HiDpi::{
    SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows::Win32::UI::WindowsAndMessaging::*;

const CAPTURE_CANCEL_MESSAGE: u32 = WM_APP + 71;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureMonitor {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub primary: bool,
    #[serde(skip)]
    handle: usize,
    #[serde(skip)]
    left: i32,
    #[serde(skip)]
    top: i32,
}

#[derive(Clone, Serialize)]
pub struct CaptureWindow {
    pub id: String,
    pub name: String,
    #[serde(skip)]
    handle: usize,
}

#[derive(Serialize)]
pub struct CaptureTargets {
    pub supported: bool,
    pub monitors: Vec<CaptureMonitor>,
    pub windows: Vec<CaptureWindow>,
}

#[derive(Serialize)]
pub struct CaptureResult {
    pub status: &'static str,
    pub width: u32,
    pub height: u32,
}

impl CaptureResult {
    fn cancelled() -> Self {
        Self {
            status: "cancelled",
            width: 0,
            height: 0,
        }
    }
}

pub struct CaptureManager {
    requests: CaptureRequests,
}

impl CaptureManager {
    pub fn new() -> Self {
        Self {
            requests: CaptureRequests::default(),
        }
    }

    pub fn cancel(&self, request_id: &str) -> Result<(), String> {
        if let Some(request) = self.requests.cancel(request_id)? {
            let overlay = request.overlay.load(Ordering::SeqCst);
            if overlay != 0 {
                unsafe {
                    let _ = PostMessageW(
                        Some(HWND(overlay as *mut _)),
                        CAPTURE_CANCEL_MESSAGE,
                        WPARAM(&request.overlay as *const _ as usize),
                        Default::default(),
                    );
                }
            }
        }
        Ok(())
    }

    pub fn capture(
        &self,
        app: &AppHandle,
        mode: &str,
        target_id: &str,
        request_id: &str,
    ) -> Result<CaptureResult, String> {
        let Some(request) = self.requests.begin(request_id)? else {
            return Ok(CaptureResult::cancelled());
        };
        let _busy = BusyGuard {
            manager: self,
            request: request.clone(),
        };
        if request.cancelled.load(Ordering::SeqCst) {
            return Ok(CaptureResult::cancelled());
        }
        let _apartment = Apartment::new()?;
        let targets = enumerate_targets()?;
        if !targets.supported {
            return Err("Screen capture is not supported on this device.".into());
        }
        let monitor = targets
            .monitors
            .iter()
            .find(|target| target.id == target_id)
            .cloned();
        let window = targets.windows.iter().find(|target| target.id == target_id);
        // Resolve IDs against a fresh native enumeration. Caller-supplied handles
        // cannot target hidden, minimized, or Project Notch windows.
        let interop: IGraphicsCaptureItemInterop =
            factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>().map_err(capture_error)?;
        let item: GraphicsCaptureItem = unsafe {
            match mode {
                "area" | "screen" => interop.CreateForMonitor(HMONITOR(
                    monitor
                        .as_ref()
                        .ok_or("The display is no longer available.")?
                        .handle as *mut _,
                )),
                "window" => interop.CreateForWindow(HWND(
                    window
                        .ok_or("The window is no longer available. Refresh the list.")?
                        .handle as *mut _,
                )),
                _ => return Err("Choose Area, Window, or Screen.".into()),
            }
        }
        .map_err(capture_error)?;
        let notch = app
            .get_webview_window("main")
            .ok_or("The notch window is unavailable.")?;
        let clipboard_owner = HWND(notch.hwnd().map_err(|error| error.to_string())?.0);
        notch.hide().map_err(|error| error.to_string())?;
        let _restore = RestoreNotch(notch);
        // Tauri may marshal the hide to its UI thread. Wait until Win32 confirms
        // it is hidden before flushing composition and starting a capture.
        let hidden_deadline = Instant::now() + Duration::from_secs(1);
        while unsafe { IsWindowVisible(clipboard_owner).as_bool() } {
            if Instant::now() >= hidden_deadline {
                return Err("Could not hide the notch before capture. Try again.".into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        unsafe {
            DwmFlush().map_err(capture_error)?;
        }
        if request.cancelled.load(Ordering::SeqCst) {
            return Ok(CaptureResult::cancelled());
        }
        let image = match snapshot(&item, &request.cancelled)? {
            Some(image) => image,
            None => return Ok(CaptureResult::cancelled()),
        };
        if let Some(original) = monitor.as_ref() {
            let current = enumerate_targets()?
                .monitors
                .into_iter()
                .find(|target| target.id == original.id)
                .ok_or("The display was disconnected. Choose a display and try again.")?;
            if current.left != original.left
                || current.top != original.top
                || current.width != image.width
                || current.height != image.height
            {
                return Err(
                    "The display configuration changed during capture. Refresh and try again."
                        .into(),
                );
            }
        }
        // Close the capture session before showing the frozen selection overlay;
        // neither overlay nor notch can become part of the captured image.
        let image = if mode == "area" {
            let monitor = monitor
                .as_ref()
                .ok_or("The display is no longer available.")?;
            let selection = selection::select(
                &image,
                monitor.left,
                monitor.top,
                &request.cancelled,
                &request.overlay,
            )?;
            let Some(rect) = selection else {
                return Ok(CaptureResult::cancelled());
            };
            image.crop(rect)?
        } else {
            image
        };
        if request.cancelled.load(Ordering::SeqCst) {
            return Ok(CaptureResult::cancelled());
        }
        if !write_image_clipboard(&image, &request.cancelled, clipboard_owner)? {
            return Ok(CaptureResult::cancelled());
        }
        Ok(CaptureResult {
            status: "copied",
            width: image.width,
            height: image.height,
        })
    }
}

struct BusyGuard<'a> {
    manager: &'a CaptureManager,
    request: Arc<CaptureRequest>,
}
impl Drop for BusyGuard<'_> {
    fn drop(&mut self) {
        self.request.overlay.store(0, Ordering::SeqCst);
        self.manager.requests.finish(&self.request.id);
    }
}
struct RestoreNotch(tauri::WebviewWindow);
impl Drop for RestoreNotch {
    fn drop(&mut self) {
        let _ = self.0.show();
        let _ = self.0.set_focus();
    }
}
struct Apartment;
impl Apartment {
    fn new() -> Result<Self, String> {
        unsafe {
            RoInitialize(RO_INIT_MULTITHREADED).map_err(capture_error)?;
        }
        Ok(Self)
    }
}
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe {
            RoUninitialize();
        }
    }
}
fn capture_error(error: windows::core::Error) -> String {
    format!("Windows could not capture this target. It may be protected or unavailable. ({error})")
}

pub fn list_targets() -> Result<CaptureTargets, String> {
    let _apartment = Apartment::new()?;
    enumerate_targets()
}

fn enumerate_targets() -> Result<CaptureTargets, String> {
    let mut monitors = Vec::<CaptureMonitor>::new();
    let mut windows = Vec::<CaptureWindow>::new();
    unsafe {
        let previous = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let enumerated = EnumDisplayMonitors(
            None,
            None,
            Some(enum_monitor),
            LPARAM(&mut monitors as *mut _ as isize),
        );
        SetThreadDpiAwarenessContext(previous);
        enumerated.ok().map_err(capture_error)?;
        EnumWindows(Some(enum_window), LPARAM(&mut windows as *mut _ as isize))
            .map_err(capture_error)?;
    }
    windows.sort_by_key(|window| window.name.to_lowercase());
    Ok(CaptureTargets {
        supported: GraphicsCaptureSession::IsSupported().unwrap_or(false),
        monitors,
        windows,
    })
}

unsafe extern "system" fn enum_monitor(
    monitor: HMONITOR,
    _: HDC,
    _: *mut RECT,
    data: LPARAM,
) -> BOOL {
    let list = &mut *(data.0 as *mut Vec<CaptureMonitor>);
    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
    if GetMonitorInfoW(monitor, &mut info as *mut _ as *mut MONITORINFO).as_bool() {
        let rect = info.monitorInfo.rcMonitor;
        let end = info
            .szDevice
            .iter()
            .position(|value| *value == 0)
            .unwrap_or(info.szDevice.len());
        list.push(CaptureMonitor {
            id: format!("monitor:{}", monitor.0 as usize),
            name: String::from_utf16_lossy(&info.szDevice[..end]),
            width: (rect.right - rect.left).max(0) as u32,
            height: (rect.bottom - rect.top).max(0) as u32,
            primary: info.monitorInfo.dwFlags & 1 != 0,
            handle: monitor.0 as usize,
            left: rect.left,
            top: rect.top,
        });
    }
    BOOL(1)
}

unsafe extern "system" fn enum_window(hwnd: HWND, data: LPARAM) -> BOOL {
    if !IsWindowVisible(hwnd).as_bool()
        || IsIconic(hwnd).as_bool()
        || GetWindow(hwnd, GW_OWNER).is_ok_and(|owner| !owner.0.is_null())
    {
        return BOOL(1);
    }
    let mut pid = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid == GetCurrentProcessId() {
        return BOOL(1);
    }
    let mut cloaked = 0u32;
    let _ = DwmGetWindowAttribute(
        hwnd,
        DWMWA_CLOAKED,
        &mut cloaked as *mut _ as *mut _,
        std::mem::size_of::<u32>() as u32,
    );
    if cloaked != 0 {
        return BOOL(1);
    }
    let mut title = [0u16; 1024];
    let length = GetWindowTextW(hwnd, &mut title).max(0) as usize;
    if length > 0 {
        let list = &mut *(data.0 as *mut Vec<CaptureWindow>);
        list.push(CaptureWindow {
            id: format!("window:{}", hwnd.0 as usize),
            name: String::from_utf16_lossy(&title[..length]),
            handle: hwnd.0 as usize,
        });
    }
    BOOL(1)
}

struct CaptureResources {
    session: GraphicsCaptureSession,
    pool: Direct3D11CaptureFramePool,
}
impl Drop for CaptureResources {
    fn drop(&mut self) {
        let _ = self.session.Close();
        let _ = self.pool.Close();
    }
}

fn snapshot(item: &GraphicsCaptureItem, cancelled: &AtomicBool) -> Result<Option<Image>, String> {
    let size = item.Size().map_err(capture_error)?;
    if size.Width <= 0
        || size.Height <= 0
        || size.Width > 32768
        || size.Height > 32768
        || size.Width as u64 * size.Height as u64 * 4 > 256 * 1024 * 1024
    {
        return Err("This target has no capturable image or is too large.".into());
    }
    let mut device = None;
    let mut context = None;
    unsafe {
        let create = |driver,
                      device: &mut Option<ID3D11Device>,
                      context: &mut Option<ID3D11DeviceContext>| {
            D3D11CreateDevice(
                None::<&IDXGIAdapter>,
                driver,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(device),
                None,
                Some(context),
            )
        };
        if create(D3D_DRIVER_TYPE_HARDWARE, &mut device, &mut context).is_err() {
            create(D3D_DRIVER_TYPE_WARP, &mut device, &mut context).map_err(capture_error)?;
        }
    }
    let device = device.ok_or("Windows could not create a capture device.")?;
    let context = context.ok_or("Windows could not create a capture context.")?;
    let dxgi: IDXGIDevice = device.cast().map_err(capture_error)?;
    let runtime: IDirect3DDevice = unsafe {
        CreateDirect3D11DeviceFromDXGIDevice(&dxgi)
            .map_err(capture_error)?
            .cast()
            .map_err(capture_error)?
    };
    // FP16 keeps HDR/scRGB values intact until explicit SDR clipboard conversion.
    let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
        &runtime,
        DirectXPixelFormat::R16G16B16A16Float,
        1,
        size,
    )
    .map_err(capture_error)?;
    let session = pool.CreateCaptureSession(item).map_err(capture_error)?;
    let resources = CaptureResources { pool, session };
    let (send, receive) = mpsc::sync_channel::<()>(1);
    let token = resources
        .pool
        .FrameArrived(&TypedEventHandler::new(move |_, _| {
            let _ = send.try_send(());
            Ok(())
        }))
        .map_err(capture_error)?;
    // Older Windows versions do not expose cursor control; their default remains.
    let _ = resources.session.SetIsCursorCaptureEnabled(false);
    resources.session.StartCapture().map_err(capture_error)?;
    let deadline = Instant::now() + Duration::from_secs(5);
    let frame = loop {
        if cancelled.load(Ordering::SeqCst) {
            return Ok(None);
        }
        if Instant::now() >= deadline {
            return Err("The capture timed out. Try another window or display.".into());
        }
        if receive.recv_timeout(Duration::from_millis(25)).is_ok() {
            if let Ok(frame) = resources.pool.TryGetNextFrame() {
                break frame;
            }
        }
    };
    let _ = resources.pool.RemoveFrameArrived(token);
    let image = (|| -> Result<Image, String> {
        let content = frame.ContentSize().map_err(capture_error)?;
        let surface = frame.Surface().map_err(capture_error)?;
        let access: IDirect3DDxgiInterfaceAccess = surface.cast().map_err(capture_error)?;
        let texture: ID3D11Texture2D = unsafe { access.GetInterface().map_err(capture_error)? };
        let mut description = D3D11_TEXTURE2D_DESC::default();
        unsafe {
            texture.GetDesc(&mut description);
        }
        let width = (content.Width.max(0) as u32).min(description.Width);
        let height = (content.Height.max(0) as u32).min(description.Height);
        let byte_count = width as usize * height as usize * 4;
        if width == 0 || height == 0 || byte_count > 256 * 1024 * 1024 {
            return Err("The selected image is empty or too large.".into());
        }
        description.Usage = D3D11_USAGE_STAGING;
        description.BindFlags = 0;
        description.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
        description.MiscFlags = 0;
        let mut staging = None;
        unsafe {
            device
                .CreateTexture2D(&description, None, Some(&mut staging))
                .map_err(capture_error)?;
        }
        let staging = staging.ok_or("Windows could not prepare capture pixels.")?;
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        unsafe {
            context.CopyResource(&staging, &texture);
            context
                .Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
                .map_err(capture_error)?;
        }
        let mut pixels = Vec::with_capacity(byte_count);
        for y in 0..height as usize {
            let row = unsafe {
                std::slice::from_raw_parts(
                    (mapped.pData as *const u8).add(y * mapped.RowPitch as usize) as *const u16,
                    width as usize * 4,
                )
            };
            for pixel in row.chunks_exact(4) {
                let rgb = [
                    half_to_float(pixel[0]),
                    half_to_float(pixel[1]),
                    half_to_float(pixel[2]),
                ];
                pixels.extend_from_slice(&[
                    linear_to_sdr(rgb[2]),
                    linear_to_sdr(rgb[1]),
                    linear_to_sdr(rgb[0]),
                    255,
                ]);
            }
        }
        unsafe {
            context.Unmap(&staging, 0);
        }
        Ok(Image {
            width,
            height,
            pixels,
        })
    })();
    let _ = frame.Close();
    image.map(Some)
}

fn write_image_clipboard(
    image: &Image,
    cancelled: &AtomicBool,
    owner: HWND,
) -> Result<bool, String> {
    let header = BITMAPV5HEADER {
        bV5Size: std::mem::size_of::<BITMAPV5HEADER>() as u32,
        bV5Width: image.width as i32,
        bV5Height: -(image.height as i32),
        bV5Planes: 1,
        bV5BitCount: 32,
        bV5Compression: BI_BITFIELDS,
        bV5SizeImage: image.pixels.len() as u32,
        bV5RedMask: 0x00ff0000,
        bV5GreenMask: 0x0000ff00,
        bV5BlueMask: 0x000000ff,
        bV5AlphaMask: 0xff000000,
        bV5CSType: 0x73524742,
        ..Default::default()
    };
    unsafe {
        let memory = GlobalAlloc(
            GMEM_MOVEABLE,
            std::mem::size_of::<BITMAPV5HEADER>() + image.pixels.len(),
        )
        .map_err(|_| "Not enough memory to copy the screenshot.")?;
        let destination = GlobalLock(memory);
        if destination.is_null() {
            let _ = GlobalFree(Some(memory));
            return Err("Could not prepare the screenshot clipboard image.".into());
        }
        std::ptr::copy_nonoverlapping(
            &header as *const _ as *const u8,
            destination as *mut u8,
            std::mem::size_of::<BITMAPV5HEADER>(),
        );
        std::ptr::copy_nonoverlapping(
            image.pixels.as_ptr(),
            (destination as *mut u8).add(std::mem::size_of::<BITMAPV5HEADER>()),
            image.pixels.len(),
        );
        let _ = GlobalUnlock(memory);
        let result = (|| {
            let mut opened = false;
            for _ in 0..10 {
                if cancelled.load(Ordering::SeqCst) {
                    return Ok(false);
                }
                if OpenClipboard(Some(owner)).is_ok() {
                    opened = true;
                    break;
                }
                std::thread::sleep(Duration::from_millis(30));
            }
            if !opened {
                return Err(
                    "The clipboard is busy. Close the other clipboard action and try again.".into(),
                );
            }
            if cancelled.load(Ordering::SeqCst) {
                let _ = CloseClipboard();
                return Ok(false);
            }
            // Allocation and cancellation checks precede EmptyClipboard; cancel
            // never clears the user's existing clipboard contents.
            let copied = EmptyClipboard()
                .and_then(|_| SetClipboardData(17, Some(HANDLE(memory.0))))
                .map(|_| true)
                .map_err(|_| "Windows could not copy the screenshot. Try again.".to_string());
            let _ = CloseClipboard();
            copied
        })();
        // Successful SetClipboardData transfers ownership to Windows.
        if result.as_ref() != Ok(&true) {
            let _ = GlobalFree(Some(memory));
        }
        result
    }
}
