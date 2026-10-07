# PROJECT_NOTCH — Windows Native Integrations

**Status:** Phase 8 Downloads Awareness Complete  
**Reference Document:** [PRODUCT_SPEC.md](../PRODUCT_SPEC.md)

---

## 1. Overview & Native Boundaries

All operating system interactions in `PROJECT_NOTCH` are managed through isolated native Rust modules within `src-tauri/src/integrations/`. The frontend receives typed data structures and never communicates directly with Win32/COM runtime interfaces.

### Core Implementation Rules
1. **Prefer native Windows events over polling:** Windows messages and callbacks are used whenever available (e.g. WinRT GSMTC session events, Win32 `SetWinEventHook` with `EVENT_SYSTEM_FOREGROUND`, `WM_CLIPBOARDUPDATE`, Win32 `ReadDirectoryChangesW`).
2. **Never fake missing capabilities:** If metadata, battery time, or progress is unavailable from Windows, the UI cleanly presents an honest null or partial state. Never invent progress percentages.
3. **Graceful degradation:** If an OS component fails or is denied permission, only that feature is disabled; the notch shell and mascot remain functional.
4. **Local-first privacy:** Zero network requests, zero telemetry, zero keystroke recording, and clipboard persistence OFF by default.

---

## 2. Integration Modules

| Module | Target Windows API / Mechanism | Purpose | Status |
| :--- | :--- | :--- | :--- |
| **Media** | `Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager` (GSMTC) | Active audio session discovery, metadata, artwork stream, transport controls, native events | **Implemented (Phase 3)** in `src-tauri/src/integrations/media.rs` |
| **Foreground Process & Coding** | Win32 `SetWinEventHook` (`EVENT_SYSTEM_FOREGROUND`) + `QueryFullProcessImageNameW` | Event-driven debounced active editor detection (VS Code, Visual Studio, JetBrains, Terminals, Antigravity) with session duration | **Implemented (Phase 4)** in `src-tauri/src/integrations/foreground.rs` |
| **Power & Battery** | Win32 `GetSystemPowerStatus` | Battery percentage, AC/battery source, charging indicator, valid remaining time | **Implemented (Phase 7)** in `src-tauri/src/integrations/power.rs` |
| **Clipboard** | Win32 `AddClipboardFormatListener` & `WM_CLIPBOARDUPDATE` | Event-driven text clipboard change notifications, consecutive deduplication, volatile memory privacy | **Implemented (Phase 7)** in `src-tauri/src/integrations/clipboard.rs` |
| **Downloads Watcher** | Win32 `ReadDirectoryChangesW` + `IFileOpenDialog` (`FOS_PICKFOLDERS`) | Watched folder download awareness, partial download heuristics, event deduplication, clean watcher cleanup | **Implemented (Phase 8)** in `src-tauri/src/integrations/downloads.rs` |
| **App Launcher** | `std::process::Command` + Win32 `IFileOpenDialog`, `IShellLinkW`, `SHGetFileInfoW` | Safe application selection, shortcut target resolution, native icon extraction, and direct process launching without shell interpretation | **Implemented (Phase 6)** in `src-tauri/src/integrations/launcher.rs` |
| **Startup** | Windows Registry `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` | Optional launch-at-login setting, native registry read/write, safe error recovery | **Implemented (Phase 9)** in `src-tauri/src/integrations/startup.rs` |

---

## 3. Platform Limitations & Strategy

### 3.1 Media Session (GSMTC) — Phase 3 Implementation
- **API Family:** `Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager` using Microsoft `windows` v0.60 crate.
- **Availability:** Windows 10 (version 1809+) and Windows 11.
- **Event Architecture:** 100% event-driven via WinRT `TypedEventHandler`:
  - `CurrentSessionChanged`: tracks when the primary audio session switches between applications (e.g. Spotify, Chrome, Apple Music).
  - `MediaPropertiesChanged`: captures track title, artist, album, and thumbnail stream updates.
  - `PlaybackInfoChanged`: captures playing/paused status and control capabilities (`can_play`, `can_pause`, `can_toggle_play_pause`, `can_skip_previous`, `can_skip_next`).
  - `TimelinePropertiesChanged`: captures position and track duration.
- **COM Apartment:** Uses Multi-Threaded Apartment (`COINIT_MULTITHREADED`) on dedicated worker threads.
- **Artwork Stream Invariant:** Streams are read safely via `DataReader`, capped at 2MB to prevent memory bloat, converted to base64 Data URLs, and rendered in a stable 38x38px container.
- **Platform Limitations:**
  - Only applications that integrate with Windows System Media Transport Controls (SMTC) report metadata. Legacy Win32 media players without SMTC support will not broadcast sessions.
  - Some web browsers or media apps report empty string metadata until user interaction starts audio.
  - Spotify login / Spotify Web API are intentionally not used to preserve local-first zero-network architecture.
  - When no session is active or playback is stopped, the right zone gracefully renders a restrained "Nothing playing" state with disabled controls without altering the 176px boundary.

### 3.2 Foreground Application & Coding Context — Phase 4 Implementation
- **API Mechanism:** Native Win32 `SetWinEventHook` listening exclusively for `EVENT_SYSTEM_FOREGROUND` events with `WINEVENT_OUTOFCONTEXT` on a dedicated thread running a Win32 message pump (`GetMessageW`).
- **Inspection Invariant:**
  - Evaluates `GetForegroundWindow()` -> `GetWindowThreadProcessId` -> `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` -> `QueryFullProcessImageNameW`.
  - Queries top-level window title via `GetWindowTextW`.
- **Privacy Guarantees:**
  - Zero keystroke recording or monitoring.
  - Zero inspection of code files, workspace contents, or buffer text.
  - Zero activity logging or persistent storage of application history.
- **Debouncing Architecture:**
  - Fast foreground switches (e.g. rapid Alt+Tab or transient OS popups) are debounced by 300ms before state evaluation.
  - Interacting with Project Notch's own window bypasses session deactivation so clicking or expanding the companion never resets the active coding session.
- **Classification Coverage:**
  - Visual Studio Code (`Code.exe`, `code - oss.exe`, `vscodium.exe`, `cursor.exe`, `windsurf.exe`).
  - Microsoft Visual Studio (`devenv.exe`).
  - JetBrains IDE family (IntelliJ, PyCharm, WebStorm, CLion, RustRover, Rider, GoLand, DataGrip, RubyMine, PhpStorm, Fleet, Android Studio).
  - Terminal applications (Windows Terminal, PowerShell, CMD, Alacritty, WezTerm, Git Bash, Hyper, Tabby).
  - Antigravity Desktop / Coding Agent (`antigravity.exe`, or window title containing `Antigravity`).
- **Session Continuity & Idle Return:**
  - Switching between coding applications (e.g. from VS Code to Windows Terminal) maintains active session duration.
  - When the user leaves coding applications, the session is marked inactive and the context engine cleanly reveals idle or higher-priority contexts.

### 3.3 App Launcher & Safe Process Execution — Phase 6 Implementation
- **Executable & Shortcut Selection:**
  - Native Win32 `IFileOpenDialog` with `COMDLG_FILTERSPEC` filtering to `*.exe;*.lnk`.
  - Executed on a dedicated thread initialized with `COINIT_APARTMENTTHREADED` COM apartment.
- **Windows Shortcut (`.lnk`) Resolution:**
  - Utilizes Win32 `IShellLinkW` and `IPersistFile` COM interfaces.
  - Resolves target binary path via `IShellLinkW::GetPath` and default arguments via `IShellLinkW::GetArguments`.
  - Derives display name cleanly from shortcut file stem (e.g. `Cap.io.lnk` -> `Cap.io`).
- **Native Icon Extraction Invariant:**
  - Win32 `SHGetFileInfoW` (`SHGFI_ICON | SHGFI_LARGEICON`) extracts the application's native `HICON`.
  - Converts icon via GDI memory device context into 32-bit BGRA DIB bitmap pixels.
  - Encodes directly into an in-memory 32-bit BMP stream (`data:image/bmp;base64,...`), bypassing disk I/O, Webview asset protocols, and internet searches.
- **Safe Process Spawning (Anti-Shell Execution Invariant):**
  - Uses `std::process::Command::new(&app.executable_path).args(&app.args)`.
  - Executable path and argument array are strictly decoupled.
  - **Zero shell interpretation:** Never routes commands through `cmd.exe` or `powershell.exe`.
  - Working directory is anchored to the application binary's parent folder.
- **Path Verification & Broken App Handling:**
  - Verifies `std::path::Path::new(&app.executable_path).exists()`.
  - Missing/moved applications return structured `AppNotFound` errors.
  - UI offers explicit `Locate` (re-open file picker) and `Remove` options without crashing the companion.
- **Platform Limitations:**
  - Universal Windows Platform (UWP) apps packaged without a classic `.exe` (e.g. Windows Calculator UWP packaged stub) require their shortcut `.lnk` in `shell:AppsFolder` or standard `.exe` launcher.
  - Running elevated processes (requiring UAC elevation) will invoke Windows standard UAC prompt upon launch.

### 3.4 Power & Battery — Phase 7 Implementation
- **API Mechanism:** Win32 `GetSystemPowerStatus` via `windows::Win32::System::Power`.
- **Exposed Signals:**
  - `has_battery`: distinguishes desktop machines without batteries from battery-powered laptops.
  - `percentage`: 0 to 100 integer.
  - `is_charging`: derived from `BatteryFlag & 8` or AC line online status.
  - `is_ac_connected`: online status (`ACLineStatus == 1`).
  - `remaining_seconds`: exact remaining time only if Windows kernel supplies a non-sentinel value (`BatteryLifeTime != 0xFFFFFFFF` and `> 0`).
- **Invariants:**
  - **Never fabricate remaining time:** When Windows returns unknown/sentinel remaining duration or system is on AC power, remaining seconds is cleanly reported as `None`.
  - **Conservative internal notification:** Crossing below 20% on battery generates a single internal notification and sets a subtle mascot warning state; latches to avoid repetitive alert spam until recharged.
- **Platform Limitations:**
  - Many modern Windows OEM battery drivers do not compute battery duration immediately after unplugging, returning sentinel `0xFFFFFFFF` for several minutes until drain rates settle. The notch displays pure percentage without faking estimates.

### 3.5 Clipboard Listener & History — Phase 7 Implementation
- **API Mechanism:** Win32 `AddClipboardFormatListener` and `WM_CLIPBOARDUPDATE` on a dedicated message-pump thread with clean `RemoveClipboardFormatListener` on shutdown.
- **Text Retrieval:**
  - Unicode text only (`CF_UNICODETEXT`).
  - Read via `OpenClipboard`, `GetClipboardData`, `GlobalLock`, and `GlobalUnlock`.
  - Non-text formats (images, files) are strictly ignored in accordance with MVP specifications.
- **Privacy & Storage Invariants:**
  - **Volatile memory only:** History entries exist in memory; disk persistence is OFF by default.
  - **Zero logging:** Clipboard text is never output to stdout, debug logs, or production traces.
  - **Zero network transmission:** Local-first invariant strictly maintained.
  - **User-initiated surface:** Background clipboard updates do NOT replace or hijack the active center context. History surface is opened intentionally by the user via the `Clip` action trigger or shortcut.
- **History Controls:**
  - Deduplicates identical consecutive entries.
  - Configurable maximum history size (1..=100 items, default 25).
  - User pause/resume toggle (`is_paused`).
  - User clear history action.
- **Platform Limitations:**
  - If another application holds exclusive locks on the clipboard during clipboard updates, `OpenClipboard` is retried with backoff. If permanently locked, the update is dropped cleanly without crashing.

### 3.6 Downloads Awareness & Watched Folders — Phase 8 Implementation
- **API Mechanism:** Native Win32 `ReadDirectoryChangesW` via `windows::Win32::Storage::FileSystem` and `windows::Win32::System::IO` on dedicated background worker threads per watched folder.
- **Directory Watching Parameters:**
  - Opened via `CreateFileW` with `FILE_LIST_DIRECTORY.0`, `FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE`, and `FILE_FLAG_BACKUP_SEMANTICS`.
  - Watched filters: `FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_LAST_WRITE | FILE_NOTIFY_CHANGE_SIZE`.
  - Zero tight polling loops: blocks on the operating system until the filesystem kernel driver signals file modifications or renames.
- **Watched Folders Management:**
  - Default configured folder: `%USERPROFILE%\Downloads` automatically initialized.
  - Native Windows Folder Picker: `IFileOpenDialog` configured with `FOS_PICKFOLDERS` on an apartment-threaded COM thread (`pick_download_folder`).
  - Persistent configuration: saved locally to `downloads_config.json` in local app data directory.
  - User can add and remove watched folders anytime.
- **Partial Download Heuristics:**
  - Recognized partial suffixes:
    - `.crdownload` — Chromium, Google Chrome, Microsoft Edge, Brave, Opera, Vivaldi.
    - `.part` — Mozilla Firefox, Waterfox, LibreWolf, Transmission.
    - `.download` — Safari and native macOS/cross-platform download tools.
    - `.!ut` — uTorrent, BitTorrent.
    - `.mega` — Mega desktop client.
    - `.tmp` / `.temp` — temporary download files from command-line downloaders.
  - Target filename extraction: automatically strips partial extensions to surface the clean intended file name (e.g. `assets.zip.crdownload` -> `assets.zip`).
- **Anti-Fabrication Progress Invariant:**
  - Modern web browsers write temporary files dynamically from 0 bytes up to total bytes. The expected total file size is maintained in browser memory or SQLite history databases, not standard filesystem metadata headers.
  - **Never invent progress percentages:** Unless a genuine, trustworthy total size signal exists, `progressPercent` is strictly reported as `None`.
  - Compact and expanded UI surfaces display:
    ```text
    Downloading…
    filename
    ```
    Percentage is only rendered if an authentic total size signal exists.
- **Event Deduplication & Debounce Strategy:**
  - Windows emits frequent `FILE_ACTION_MODIFIED` events as download chunks write (often dozens of events per second).
  - Debouncing: active download progress events (`download.changed`) are throttled to at most once per 1000ms per file.
  - Completion Latch: when a partial file is renamed (`FILE_ACTION_RENAMED_NEW_NAME` from `.crdownload` or `.part` to final file name), completion is detected. The file path is latched in a 10-second deduplication cache so secondary write events do not emit duplicate notifications.
- **Completion Workflow (Requirement 7):**
  - **Short center notification:** Pushed to Context Engine at `P1_COMPLETION` with title "Download complete" and the completed filename (duration ~4500ms).
  - **Mascot celebration:** Triggers `mascotStore.celebrate(3500)` showing celebratory animation.
  - **Open Folder action:** Action button triggers `open_download_folder`, highlighting the completed file in Windows File Explorer via `explorer.exe /select,"<filePath>"` (or opening the parent folder).
- **Clean Watcher Cleanup (Requirement 9):**
  - Each watched folder worker thread maintains a cancellation `stop_signal: Arc<AtomicBool>` and the directory handle.
  - When a folder is removed: `CancelIoEx` aborts pending I/O, `CloseHandle` releases OS resources, and the thread is joined.
  - When `DownloadsManager` drops or the app quits: all active watchers are cleanly shut down and handles closed.
- **Platform & Browser Limitations:**
  1. *No Generic Windows API for In-Browser Progress:* Windows does not provide a universal OS-level hook to monitor byte progress inside sandboxed web browsers without browser extensions or invasive debugging APIs.
  2. *Direct Writes without Temporary Suffixes:* Some tools or browsers (e.g. "Save Image As...") write directly to the final filename without a `.crdownload` or `.part` extension.
  3. *Unwatched Directories:* Files saved outside of user-configured watched folders cannot be detected.
  4. *Cloud Storage Virtual Files:* Cloud sync providers (OneDrive, Dropbox) may emit hydration events that look like write activity when downloading files on demand.

---

### 3.7 Windows Startup & Settings Persistence — Phase 9 Implementation
- **Registry Key:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` with value name `ProjectNotch`.
- **API Mechanism:** Native Win32 Registry APIs (`RegOpenKeyExW`, `RegQueryValueExW`, `RegSetValueExW`, `RegDeleteValueW`, `RegCloseKey`) via Microsoft `windows` v0.60 crate with `Win32_System_Registry`.
- **Executable Quoting:** Properly quotes executable paths containing spaces (e.g. `"\"C:\Path With Spaces\project-notch.exe\""`).
- **Settings Persistence & Version Migration:**
  - Persisted locally to `%APPDATA%\project-notch\settings.json`.
  - Typed configuration schema (`schemaVersion: 1`).
  - Automatic migration strategy for legacy/unversioned payloads.
  - Safe error recovery: corrupted files automatically backed up to `settings.corrupted.bak` with graceful fallback to defaults without crashing.
  - Live runtime application without restart where practical (accent color, always-on-top, monitor placement, auto-collapse delay, mascot toggle, clipboard listener, performance mode).
- **Platform Limitations:**
  - In development builds (`cargo run`), `current_exe()` resolves to the target debug binary rather than an installed release shortcut.
  - If user group policies restrict registry writes to `HKCU\...\Run`, error is reported to user notification rather than crashing.


