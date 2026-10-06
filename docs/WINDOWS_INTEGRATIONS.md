# PROJECT_NOTCH — Windows Native Integrations

**Status:** Phase 4 Context Engine & Coding Detection Complete  
**Reference Document:** [PRODUCT_SPEC.md](../PRODUCT_SPEC.md)

---

## 1. Overview & Native Boundaries

All operating system interactions in `PROJECT_NOTCH` are managed through isolated native Rust modules within `src-tauri/src/integrations/`. The frontend receives typed data structures and never communicates directly with Win32/COM runtime interfaces.

### Core Implementation Rules
1. **Prefer native Windows events over polling:** Windows messages and callbacks are used whenever available (e.g. WinRT GSMTC session events, Win32 `SetWinEventHook` with `EVENT_SYSTEM_FOREGROUND`, `WM_CLIPBOARDUPDATE`).
2. **Never fake missing capabilities:** If metadata, battery time, or progress is unavailable from Windows, the UI cleanly presents an honest null or partial state.
3. **Graceful degradation:** If an OS component fails or is denied permission, only that feature is disabled; the notch shell and mascot remain functional.
4. **Local-first privacy:** Zero network requests, zero telemetry, zero keystroke recording, and clipboard persistence OFF by default.

---

## 2. Integration Modules

| Module | Target Windows API / Mechanism | Purpose | Status |
| :--- | :--- | :--- | :--- |
| **Media** | `Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager` (GSMTC) | Active audio session discovery, metadata, artwork stream, transport controls, native events | **Implemented (Phase 3)** in `src-tauri/src/integrations/media.rs` |
| **Foreground Process & Coding** | Win32 `SetWinEventHook` (`EVENT_SYSTEM_FOREGROUND`) + `QueryFullProcessImageNameW` | Event-driven debounced active editor detection (VS Code, Visual Studio, JetBrains, Terminals, Antigravity) with session duration | **Implemented (Phase 4)** in `src-tauri/src/integrations/foreground.rs` |
| **Power & Battery** | Win32 `GetSystemPowerStatus` | Battery percentage, AC/DC status, charging indicator | Typed placeholder model in `integrations/power.rs` |
| **Clipboard** | Win32 `AddClipboardFormatListener` & `WM_CLIPBOARDUPDATE` | Event-driven text clipboard change notifications | Typed placeholder model in `integrations/clipboard.rs` |
| **Downloads Watcher** | Filesystem change events on configured watch directories | Watched folder change awareness | Typed placeholder model in `integrations/downloads.rs` |
| **App Launcher** | `std::process::Command` with explicit executable and argument split | Safe application launching without raw shell evaluation | Typed placeholder model in `integrations/launcher.rs` |
| **Startup** | Windows Registry `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` | Optional launch-at-login setting | Typed placeholder model in `integrations/startup.rs` |

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

### 3.3 Battery Time Remaining
- **Limitation:** Modern Windows power drivers frequently return `BATTERY_LIFE_TIME_UNKNOWN` (`0xFFFFFFFF`).
- **Handling:** Never invent estimated time remaining. Only display percentage and charging state when duration is undefined.

### 3.4 Global Download Progress
- **Limitation:** Windows does not provide a centralized operating system API to inspect browser download byte progress.
- **Handling:** The user configures watched download folders. Filesystem events observe active `.crdownload`, `.part`, or temporary file extensions and confirm completion. Progress percentages are only rendered if a genuine size signal exists.

### 3.5 Clipboard Privacy
- **Limitation:** Clipboard listeners can capture sensitive text (passwords, tokens).
- **Handling:** History capture is paused when notch is disabled or privacy mode is engaged; clipboard history is kept strictly in volatile memory and is never written to disk unless explicitly enabled.

