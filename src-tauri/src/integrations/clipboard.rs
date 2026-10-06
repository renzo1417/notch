//! Clipboard Integration module.
//!
//! Event-driven native Windows clipboard change listener using
//! AddClipboardFormatListener / WM_CLIPBOARDUPDATE.
//! Defined in PRODUCT_SPEC.md section 17 and Phase 7 requirements.
//!
//! Privacy Invariants:
//! - Persistence is OFF by default.
//! - Clipboard content is NEVER uploaded or sent anywhere.
//! - Clipboard text is NEVER logged in production.
//! - History capture can be paused or cleared by the user at any time.

use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

use windows::core::w;
use windows::Win32::Foundation::{HANDLE, HGLOBAL, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::DataExchange::{
    AddClipboardFormatListener, CloseClipboard, EmptyClipboard, GetClipboardData,
    IsClipboardFormatAvailable, OpenClipboard, RemoveClipboardFormatListener, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    PostQuitMessage, RegisterClassW, TranslateMessage, MSG,
    WNDCLASSW, WS_POPUP, WM_CLIPBOARDUPDATE, WM_DESTROY,
};

/// Win32 Standard Unicode Text clipboard format identifier.
const CF_UNICODETEXT: u32 = 13;

/// Model for captured clipboard text entries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardItem {
    pub id: String,
    pub text: String,
    pub timestamp_ms: u64,
}

/// Global callback hook for WM_CLIPBOARDUPDATE notifications.
static CLIPBOARD_NOTIFY_CALLBACK: Mutex<Option<Box<dyn Fn() + Send + Sync>>> = Mutex::new(None);

unsafe extern "system" fn clipboard_listener_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CLIPBOARDUPDATE => {
            if let Ok(guard) = CLIPBOARD_NOTIFY_CALLBACK.lock() {
                if let Some(ref cb) = *guard {
                    cb();
                }
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            let _ = RemoveClipboardFormatListener(hwnd);
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Reads the current Unicode text from Windows clipboard.
/// Returns None if format is not text or cannot be opened.
pub fn read_clipboard_text() -> Option<String> {
    unsafe {
        // Spec: "text items only"
        if IsClipboardFormatAvailable(CF_UNICODETEXT).is_err() {
            return None;
        }

        let mut opened = false;
        for _ in 0..5 {
            if OpenClipboard(None).is_ok() {
                opened = true;
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }

        if !opened {
            return None;
        }

        let result = (|| {
            let handle = GetClipboardData(CF_UNICODETEXT).ok()?;
            if handle.0.is_null() {
                return None;
            }

            let hglobal = HGLOBAL(handle.0);
            let ptr = GlobalLock(hglobal);
            if ptr.is_null() {
                return None;
            }

            let wide_slice = {
                let mut len = 0;
                let mut curr = ptr as *const u16;
                while *curr != 0 {
                    len += 1;
                    curr = curr.add(1);
                }
                std::slice::from_raw_parts(ptr as *const u16, len)
            };

            let text = String::from_utf16_lossy(wide_slice);
            let _ = GlobalUnlock(hglobal);
            Some(text)
        })();

        let _ = CloseClipboard();
        result
    }
}

/// Sets text onto the Windows clipboard.
pub fn write_clipboard_text(text: &str) -> Result<(), String> {
    unsafe {
        let mut opened = false;
        for _ in 0..5 {
            if OpenClipboard(None).is_ok() {
                opened = true;
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }

        if !opened {
            return Err("Failed to open clipboard".to_string());
        }

        let res = (|| {
            let _ = EmptyClipboard();

            let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let byte_count = utf16.len() * std::mem::size_of::<u16>();

            let hmem = GlobalAlloc(GMEM_MOVEABLE, byte_count).map_err(|e| e.to_string())?;
            if hmem.0.is_null() {
                return Err("Failed to allocate memory for clipboard".to_string());
            }

            let ptr = GlobalLock(hmem);
            if ptr.is_null() {
                return Err("Failed to lock global memory".to_string());
            }

            std::ptr::copy_nonoverlapping(utf16.as_ptr(), ptr as *mut u16, utf16.len());
            let _ = GlobalUnlock(hmem);

            SetClipboardData(CF_UNICODETEXT, Some(HANDLE(hmem.0)))
                .map_err(|e| e.to_string())?;

            Ok(())
        })();

        let _ = CloseClipboard();
        res
    }
}

/// Central manager for clipboard history, event listening, deduplication, and privacy.
pub struct ClipboardManager {
    items: Mutex<Vec<ClipboardItem>>,
    max_items: Mutex<usize>,
    is_paused: AtomicBool,
    app_handle: Mutex<Option<AppHandle>>,
    listener_hwnd: Mutex<Option<isize>>,
    is_running: AtomicBool,
}

impl ClipboardManager {
    pub fn new() -> Self {
        Self {
            items: Mutex::new(Vec::new()),
            max_items: Mutex::new(25), // Default configurable maximum: 25 items
            is_paused: AtomicBool::new(false),
            app_handle: Mutex::new(None),
            listener_hwnd: Mutex::new(None),
            is_running: AtomicBool::new(false),
        }
    }

    /// Returns the current in-memory history list.
    pub fn get_history(&self) -> Vec<ClipboardItem> {
        self.items.lock().map(|list| list.clone()).unwrap_or_default()
    }

    /// Clears the history list.
    pub fn clear_history(&self) {
        if let Ok(mut list) = self.items.lock() {
            list.clear();
        }
    }

    /// Sets whether history capture is paused.
    pub fn set_paused(&self, paused: bool) {
        self.is_paused.store(paused, Ordering::SeqCst);
    }

    /// Returns whether history capture is currently paused.
    pub fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::SeqCst)
    }

    /// Configures the maximum number of items in history.
    pub fn set_max_items(&self, max: usize) {
        let clamped = max.clamp(1, 100);
        let mut guard = self.items.lock().unwrap();
        let mut max_guard = self.max_items.lock().unwrap();
        *max_guard = clamped;
        if guard.len() > clamped {
            guard.truncate(clamped);
        }
    }

    /// Handles a clipboard change event.
    /// Deduplicates identical consecutive entries and trims to max_items.
    pub fn on_clipboard_update(&self) {
        if self.is_paused.load(Ordering::SeqCst) {
            return;
        }

        let text = match read_clipboard_text() {
            Some(t) => t,
            None => return,
        };

        // Don't record empty whitespace strings
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let mut list = self.items.lock().unwrap();

        // Deduplicate identical consecutive entries
        if let Some(first) = list.first() {
            if first.text == text {
                return;
            }
        }

        let item = ClipboardItem {
            id: format!("clip_{}_{}", now_ms, list.len()),
            text,
            timestamp_ms: now_ms,
        };

        list.insert(0, item.clone());

        let max = *self.max_items.lock().unwrap();
        if list.len() > max {
            list.truncate(max);
        }

        // Notify frontend via Tauri event
        if let Ok(handle_guard) = self.app_handle.lock() {
            if let Some(ref handle) = *handle_guard {
                let _ = handle.emit("clipboard.changed", &item);
            }
        }
    }

    /// Initializes native Windows clipboard listener on a dedicated message-pump thread.
    pub fn init(self: &Arc<Self>, app: AppHandle) {
        if self.is_running.swap(true, Ordering::SeqCst) {
            return;
        }

        {
            let mut handle_guard = self.app_handle.lock().unwrap();
            *handle_guard = Some(app);
        }

        let this = Arc::clone(self);
        {
            let mut cb_guard = CLIPBOARD_NOTIFY_CALLBACK.lock().unwrap();
            *cb_guard = Some(Box::new(move || {
                this.on_clipboard_update();
            }));
        }

        let this_for_thread = Arc::clone(self);
        thread::Builder::new()
            .name("clipboard-listener".to_string())
            .spawn(move || unsafe {
                let class_name = w!("ProjectNotchClipboardListenerClass");
                let wndclass = WNDCLASSW {
                    lpfnWndProc: Some(clipboard_listener_wndproc),
                    lpszClassName: class_name,
                    ..Default::default()
                };

                let _ = RegisterClassW(&wndclass);

                let hwnd = match CreateWindowExW(
                    Default::default(),
                    class_name,
                    w!("ProjectNotchClipboardListener"),
                    WS_POPUP,
                    0,
                    0,
                    0,
                    0,
                    None,
                    None,
                    None,
                    None,
                ) {
                    Ok(h) => h,
                    Err(_) => return,
                };

                if let Ok(mut guard) = this_for_thread.listener_hwnd.lock() {
                    *guard = Some(hwnd.0 as isize);
                }

                // Register with Windows to receive WM_CLIPBOARDUPDATE messages
                let _ = AddClipboardFormatListener(hwnd);

                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            })
            .expect("Failed to spawn clipboard listener thread");
    }

    /// Stops the clipboard listener thread and cleans up Windows hooks.
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        if let Ok(guard) = self.listener_hwnd.lock() {
            if let Some(hwnd_isize) = *guard {
                unsafe {
                    let hwnd = HWND(hwnd_isize as _);
                    let _ = DestroyWindow(hwnd);
                }
            }
        }
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}
