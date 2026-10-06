//! Foreground Process Integration Module.
//!
//! Event-driven native Windows foreground window inspection and active coding context detection.
//! Defined in PRODUCT_SPEC.md sections 14, 26, & 32.
//!
//! Privacy Invariants:
//! - Never inspects typed keystrokes or source code contents.
//! - Queries only executable file name and top-level window title.
//! - Retains zero persistent activity logs or historical tracking.
//! - Debounces rapid window switching to eliminate UI jitter.
//! - Automatically returns to idle when the user leaves coding applications.

use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

#[allow(unused_imports)]
use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HWND};
use windows::Win32::System::Threading::{
    GetCurrentThreadId, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetForegroundWindow, GetMessageW, GetWindowTextW,
    GetWindowThreadProcessId, PostThreadMessageW, TranslateMessage, EVENT_SYSTEM_FOREGROUND,
    MSG, WINEVENT_OUTOFCONTEXT, WM_QUIT,
};

/// Coding session model matching PRODUCT_SPEC.md section 14.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodingSession {
    pub app_name: String,
    pub project_label: Option<String>,
    pub elapsed_minutes: u32,
    pub process_name: String,
    pub window_title: String,
    pub is_active: bool,
    pub session_start_ms: u64,
}

impl Default for CodingSession {
    fn default() -> Self {
        Self {
            app_name: String::new(),
            project_label: None,
            elapsed_minutes: 0,
            process_name: String::new(),
            window_title: String::new(),
            is_active: false,
            session_start_ms: 0,
        }
    }
}

/// Classifies an application executable and window title into a human-readable IDE/editor identity.
/// Returns (app_display_name, parsed_project_label_hint).
pub fn classify_coding_app(process_name: &str, window_title: &str) -> Option<(&'static str, Option<String>)> {
    let lower_proc = process_name.to_lowercase();
    let lower_title = window_title.to_lowercase();

    // 1. Visual Studio Code & derivatives
    if lower_proc == "code.exe" || lower_proc == "code - oss.exe" || lower_proc == "vscodium.exe" || lower_proc == "code-insiders.exe" {
        let project = extract_vscode_project(window_title);
        return Some(("VS Code", project));
    }
    if lower_proc == "cursor.exe" {
        let project = extract_vscode_project(window_title);
        return Some(("Cursor", project));
    }
    if lower_proc == "windsurf.exe" {
        let project = extract_vscode_project(window_title);
        return Some(("Windsurf", project));
    }

    // 2. Microsoft Visual Studio
    if lower_proc == "devenv.exe" {
        let project = extract_visual_studio_project(window_title);
        return Some(("Visual Studio", project));
    }

    // 3. JetBrains IDE family
    if lower_proc == "idea64.exe" || lower_proc == "idea.exe" {
        return Some(("IntelliJ IDEA", extract_jetbrains_project(window_title)));
    }
    if lower_proc == "pycharm64.exe" || lower_proc == "pycharm.exe" {
        return Some(("PyCharm", extract_jetbrains_project(window_title)));
    }
    if lower_proc == "webstorm64.exe" || lower_proc == "webstorm.exe" {
        return Some(("WebStorm", extract_jetbrains_project(window_title)));
    }
    if lower_proc == "clion64.exe" || lower_proc == "clion.exe" {
        return Some(("CLion", extract_jetbrains_project(window_title)));
    }
    if lower_proc == "rustrover64.exe" || lower_proc == "rustrover.exe" {
        return Some(("RustRover", extract_jetbrains_project(window_title)));
    }
    if lower_proc == "rider64.exe" || lower_proc == "rider.exe" {
        return Some(("Rider", extract_jetbrains_project(window_title)));
    }
    if lower_proc == "goland64.exe" || lower_proc == "goland.exe" {
        return Some(("GoLand", extract_jetbrains_project(window_title)));
    }
    if lower_proc == "datagrip64.exe" || lower_proc == "datagrip.exe" {
        return Some(("DataGrip", extract_jetbrains_project(window_title)));
    }
    if lower_proc == "rubymine64.exe" || lower_proc == "rubymine.exe" {
        return Some(("RubyMine", extract_jetbrains_project(window_title)));
    }
    if lower_proc == "phpstorm64.exe" || lower_proc == "phpstorm.exe" {
        return Some(("PhpStorm", extract_jetbrains_project(window_title)));
    }
    if lower_proc == "fleet.exe" {
        return Some(("Fleet", extract_jetbrains_project(window_title)));
    }
    if lower_proc == "studio64.exe" || lower_proc == "androidstudio64.exe" {
        return Some(("Android Studio", extract_jetbrains_project(window_title)));
    }

    // 4. Terminal applications
    if lower_proc == "windowsterminal.exe" || lower_proc == "wt.exe" {
        return Some(("Windows Terminal", None));
    }
    if lower_proc == "powershell.exe" || lower_proc == "pwsh.exe" {
        return Some(("PowerShell", None));
    }
    if lower_proc == "cmd.exe" {
        return Some(("Command Prompt", None));
    }
    if lower_proc == "alacritty.exe" {
        return Some(("Alacritty", None));
    }
    if lower_proc == "wezterm-gui.exe" || lower_proc == "wezterm.exe" {
        return Some(("WezTerm", None));
    }
    if lower_proc == "mintty.exe" || lower_proc == "git-bash.exe" {
        return Some(("Git Bash", None));
    }
    if lower_proc == "hyper.exe" {
        return Some(("Hyper", None));
    }
    if lower_proc == "tabby.exe" {
        return Some(("Tabby", None));
    }

    // 5. Antigravity Desktop / Coding Agent
    if lower_proc == "antigravity.exe" || lower_title.contains("antigravity") {
        return Some(("Antigravity", extract_generic_project(window_title, "Antigravity")));
    }

    // 6. Other recognized coding editors
    if lower_proc == "sublime_text.exe" {
        return Some(("Sublime Text", None));
    }
    if lower_proc == "notepad++.exe" {
        return Some(("Notepad++", None));
    }
    if lower_proc == "zed.exe" {
        return Some(("Zed", None));
    }
    if lower_proc == "neovide.exe" || lower_proc == "gvim.exe" {
        return Some(("Neovim", None));
    }

    None
}

/// Heuristically extracts workspace or project folder name from VS Code window title.
/// Example format: "● file.ts — project_name — Visual Studio Code" or "project_name - Visual Studio Code"
fn extract_vscode_project(title: &str) -> Option<String> {
    if title.is_empty() {
        return None;
    }
    let parts: Vec<&str> = if title.contains(" — ") {
        title.split(" — ").collect()
    } else {
        title.split(" - ").collect()
    };

    if parts.len() >= 2 {
        // Typically the middle or second-to-last part is the project/workspace name
        let candidate = if parts.len() >= 3 {
            parts[parts.len() - 2].trim()
        } else {
            parts[0].trim()
        };
        // Clean out WSL / SSH badges
        let cleaned = candidate
            .trim_start_matches("● ")
            .split(" [")
            .next()
            .unwrap_or(candidate)
            .trim();
        if !cleaned.is_empty() && cleaned != "Visual Studio Code" {
            return Some(cleaned.to_string());
        }
    }
    None
}

/// Extracts project name from Visual Studio title ("ProjectName - Microsoft Visual Studio").
fn extract_visual_studio_project(title: &str) -> Option<String> {
    if let Some((proj, _)) = title.split_once(" - Microsoft Visual Studio") {
        let clean = proj.trim();
        if !clean.is_empty() {
            return Some(clean.to_string());
        }
    }
    None
}

/// Extracts project name from JetBrains title ("ProjectName [C:\...] - ...").
fn extract_jetbrains_project(title: &str) -> Option<String> {
    if title.is_empty() {
        return None;
    }
    let before_bracket = title.split('[').next().unwrap_or(title).trim();
    let project = before_bracket.split(" – ").next().unwrap_or(before_bracket).trim();
    if !project.is_empty() {
        Some(project.to_string())
    } else {
        None
    }
}

/// Extracts project from generic title like "Antigravity - ProjectName".
fn extract_generic_project(title: &str, app_name: &str) -> Option<String> {
    for part in title.split(&['-', '—'][..]) {
        let trimmed = part.trim();
        if !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case(app_name) {
            return Some(trimmed.to_string());
        }
    }
    None
}

/// Inspects a given HWND safely, querying executable filename and window title.
pub fn inspect_window(hwnd: HWND) -> Option<(String, String)> {
    let mut pid: u32 = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
    }
    if pid == 0 {
        return None;
    }

    let handle = match unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) } {
        Ok(h) => h,
        Err(_) => return None,
    };

    let mut buffer = [0u16; 1024];
    let mut size = buffer.len() as u32;
    let res = unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        )
    };
    let _ = unsafe { CloseHandle(handle) };

    if res.is_ok() && size > 0 {
        let full_path = String::from_utf16_lossy(&buffer[..size as usize]);
        let filename = std::path::Path::new(&full_path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or(full_path);

        let mut title_buf = [0u16; 512];
        let len = unsafe { GetWindowTextW(hwnd, &mut title_buf) };
        let title = if len > 0 {
            String::from_utf16_lossy(&title_buf[..len as usize])
        } else {
            String::new()
        };

        Some((filename, title))
    } else {
        None
    }
}

// Global hook channel sender for WinEventHook callback
static HOOK_SENDER: Mutex<Option<std::sync::mpsc::Sender<isize>>> = Mutex::new(None);

/// Global WinEventProc callback invoked by Windows on foreground window changes.
unsafe extern "system" fn foreground_win_event_proc(
    _h_win_event_hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    if let Ok(guard) = HOOK_SENDER.lock() {
        if let Some(ref sender) = *guard {
            let _ = sender.send(hwnd.0 as isize);
        }
    }
}

/// Central Foreground Manager managing native events, debouncing, session tracking,
/// and elapsed session calculations.
pub struct ForegroundManager {
    session: Mutex<CodingSession>,
    user_project_label: Mutex<Option<String>>,
    app_handle: Mutex<Option<AppHandle>>,
    hook_thread_id: Mutex<Option<u32>>,
    is_running: AtomicBool,
}

impl ForegroundManager {
    pub fn new() -> Self {
        Self {
            session: Mutex::new(CodingSession::default()),
            user_project_label: Mutex::new(None),
            app_handle: Mutex::new(None),
            hook_thread_id: Mutex::new(None),
            is_running: AtomicBool::new(false),
        }
    }

    /// Initializes native Windows foreground detection and background debouncing worker.
    pub fn init(self: &Arc<Self>, app: AppHandle) {
        if self.is_running.swap(true, Ordering::SeqCst) {
            return;
        }

        if let Ok(mut handle_guard) = self.app_handle.lock() {
            *handle_guard = Some(app);
        }

        let (tx, rx) = std::sync::mpsc::channel::<isize>();
        if let Ok(mut sender_guard) = HOOK_SENDER.lock() {
            *sender_guard = Some(tx);
        }

        let this_for_hook = Arc::clone(self);
        // Dedicated thread for WinEventHook with Win32 message pump
        std::thread::Builder::new()
            .name("windows-foreground-hook".into())
            .spawn(move || {
                let thread_id = unsafe { GetCurrentThreadId() };
                if let Ok(mut id_guard) = this_for_hook.hook_thread_id.lock() {
                    *id_guard = Some(thread_id);
                }

                let hook = unsafe {
                    SetWinEventHook(
                        EVENT_SYSTEM_FOREGROUND,
                        EVENT_SYSTEM_FOREGROUND,
                        None,
                        Some(foreground_win_event_proc),
                        0,
                        0,
                        WINEVENT_OUTOFCONTEXT,
                    )
                };

                // Win32 message loop required for WINEVENT_OUTOFCONTEXT hook
                let mut msg = MSG::default();
                while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
                    unsafe {
                        let _ = TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }

                // Clean up hook on shutdown
                if !hook.0.is_null() {
                    let _ = unsafe { UnhookWinEvent(hook) };
                }
            })
            .expect("Failed to spawn foreground hook thread");

        let this_for_worker = Arc::clone(self);
        // Worker thread for debouncing and state management
        std::thread::Builder::new()
            .name("windows-foreground-debouncer".into())
            .spawn(move || {
                let mut last_hwnd: Option<HWND> = None;
                let mut last_event_time = Instant::now();
                let debounce_duration = Duration::from_millis(300);
                let mut pending_check = false;

                // Initial probe of the current foreground window at startup
                let initial_hwnd = unsafe { GetForegroundWindow() };
                if !initial_hwnd.0.is_null() {
                    this_for_worker.evaluate_window(initial_hwnd);
                }

                while this_for_worker.is_running.load(Ordering::SeqCst) {
                    // Try to receive foreground events with a short timeout
                    match rx.recv_timeout(Duration::from_millis(100)) {
                        Ok(raw_hwnd) => {
                            last_hwnd = Some(HWND(raw_hwnd as *mut _));
                            last_event_time = Instant::now();
                            pending_check = true;
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                            // If debounce interval elapsed, process pending window
                            if pending_check && last_event_time.elapsed() >= debounce_duration {
                                pending_check = false;
                                if let Some(hwnd) = last_hwnd {
                                    this_for_worker.evaluate_window(hwnd);
                                }
                            } else {
                                // Ticker: periodically update elapsed session minutes while active
                                this_for_worker.tick_elapsed();
                            }
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                            break;
                        }
                    }
                }
            })
            .expect("Failed to spawn foreground debouncer worker");
    }

    /// Evaluates a focused window and updates coding session state accordingly.
    fn evaluate_window(&self, hwnd: HWND) {
        let own_pid = std::process::id();
        let mut window_pid: u32 = 0;
        unsafe {
            GetWindowThreadProcessId(hwnd, Some(&mut window_pid));
        }

        // If the focused window belongs to Project Notch, preserve the current coding session
        if window_pid == own_pid {
            return;
        }

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        if let Some((process_name, window_title)) = inspect_window(hwnd) {
            if let Some((app_name, parsed_project)) = classify_coding_app(&process_name, &window_title) {
                // User is in a recognized coding application!
                let user_label = self.user_project_label.lock().ok().and_then(|g| g.clone());
                let project_label = user_label.or(parsed_project);

                let mut session_guard = match self.session.lock() {
                    Ok(g) => g,
                    Err(_) => return,
                };

                if session_guard.is_active {
                    // Continuing or switching between editors: maintain elapsed time
                    session_guard.app_name = app_name.to_string();
                    session_guard.process_name = process_name;
                    session_guard.window_title = window_title;
                    if project_label.is_some() {
                        session_guard.project_label = project_label;
                    }
                    if session_guard.session_start_ms > 0 && now_ms >= session_guard.session_start_ms {
                        session_guard.elapsed_minutes =
                            ((now_ms - session_guard.session_start_ms) / 60_000) as u32;
                    }
                } else {
                    // Starting a new coding session
                    session_guard.app_name = app_name.to_string();
                    session_guard.process_name = process_name;
                    session_guard.window_title = window_title;
                    session_guard.project_label = project_label;
                    session_guard.is_active = true;
                    session_guard.session_start_ms = now_ms;
                    session_guard.elapsed_minutes = 0;
                }

                let session_clone = session_guard.clone();
                drop(session_guard);

                self.broadcast_session(&session_clone);
                return;
            }
        }

        // User is not in a coding application: mark inactive and return to idle
        let mut session_guard = match self.session.lock() {
            Ok(g) => g,
            Err(_) => return,
        };

        if session_guard.is_active {
            session_guard.is_active = false;
            let session_clone = session_guard.clone();
            drop(session_guard);

            self.broadcast_session(&session_clone);
        }
    }

    /// Periodic ticker that refreshes elapsed minutes during active coding sessions.
    fn tick_elapsed(&self) {
        let mut session_guard = match self.session.lock() {
            Ok(g) => g,
            Err(_) => return,
        };

        if !session_guard.is_active || session_guard.session_start_ms == 0 {
            return;
        }

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let minutes = ((now_ms - session_guard.session_start_ms) / 60_000) as u32;
        if minutes != session_guard.elapsed_minutes {
            session_guard.elapsed_minutes = minutes;
            let session_clone = session_guard.clone();
            drop(session_guard);
            self.broadcast_session(&session_clone);
        }
    }

    /// Emits the `coding.changed` Tauri event to the frontend.
    fn broadcast_session(&self, session: &CodingSession) {
        if let Ok(handle_guard) = self.app_handle.lock() {
            if let Some(ref app) = *handle_guard {
                let _ = app.emit("coding.changed", session);
            }
        }
    }

    /// Returns the current coding session.
    pub fn get_session(&self) -> CodingSession {
        self.session
            .lock()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    /// Sets or clears a user-defined custom project/session label.
    pub fn set_user_project_label(&self, label: Option<String>) {
        if let Ok(mut user_guard) = self.user_project_label.lock() {
            *user_guard = label.clone();
        }

        let mut session_guard = match self.session.lock() {
            Ok(g) => g,
            Err(_) => return,
        };

        session_guard.project_label = label;
        let session_clone = session_guard.clone();
        drop(session_guard);

        self.broadcast_session(&session_clone);
    }

    /// Resets the elapsed time and starts a fresh session.
    pub fn reset_session(&self) {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let mut session_guard = match self.session.lock() {
            Ok(g) => g,
            Err(_) => return,
        };

        session_guard.session_start_ms = now_ms;
        session_guard.elapsed_minutes = 0;
        let session_clone = session_guard.clone();
        drop(session_guard);

        self.broadcast_session(&session_clone);
    }

    /// Stops the worker threads and detaches the Win32 hook.
    pub fn shutdown(&self) {
        self.is_running.store(false, Ordering::SeqCst);

        // Terminate hook thread message loop
        if let Ok(id_guard) = self.hook_thread_id.lock() {
            if let Some(thread_id) = *id_guard {
                unsafe {
                    let _ = PostThreadMessageW(
                        thread_id,
                        WM_QUIT,
                        windows::Win32::Foundation::WPARAM(0),
                        windows::Win32::Foundation::LPARAM(0),
                    );
                }
            }
        }
    }
}

