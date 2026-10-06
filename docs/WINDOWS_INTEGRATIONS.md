# PROJECT_NOTCH — Windows Native Integrations

**Status:** Phase 0 Technical Foundation  
**Reference Document:** [PRODUCT_SPEC.md](../PRODUCT_SPEC.md)

---

## 1. Overview & Native Boundaries

All operating system interactions in `PROJECT_NOTCH` are managed through isolated native Rust modules within `src-tauri/src/integrations/`. The frontend receives typed data structures and never communicates directly with Win32/COM runtime interfaces.

### Core Implementation Rules
1. **Prefer native Windows events over polling:** Windows messages and callbacks are used whenever available (e.g. `WM_CLIPBOARDUPDATE`, GSMTC session events).
2. **Never fake missing capabilities:** If metadata, battery time, or progress is unavailable from Windows, the UI cleanly presents an honest null or partial state.
3. **Graceful degradation:** If an OS component fails or is denied permission, only that feature is disabled; the notch shell and mascot remain functional.
4. **Local-first privacy:** Zero network requests, zero telemetry, zero keystroke recording, and clipboard persistence OFF by default.

---

## 2. Planned Integration Modules (Phases 3–9)

| Module | Target Windows API / Mechanism | Purpose | Status in Phase 0 |
| :--- | :--- | :--- | :--- |
| **Media** | `Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager` (GSMTC) | Active audio session detection, metadata, playback controls | Typed placeholder model in `integrations/media.rs` |
| **Power & Battery** | Win32 `GetSystemPowerStatus` | Battery percentage, AC/DC status, charging indicator | Typed placeholder model in `integrations/power.rs` |
| **Clipboard** | Win32 `AddClipboardFormatListener` & `WM_CLIPBOARDUPDATE` | Event-driven text clipboard change notifications | Typed placeholder model in `integrations/clipboard.rs` |
| **Foreground Process** | Win32 `GetForegroundWindow` + `QueryFullProcessImageNameW` | Debounced active editor detection (VS Code, Visual Studio, etc.) | Typed placeholder model in `integrations/foreground.rs` |
| **Downloads Watcher** | Filesystem change events on configured watch directories | Watched folder change awareness | Typed placeholder model in `integrations/downloads.rs` |
| **App Launcher** | `std::process::Command` with explicit executable and argument split | Safe application launching without raw shell evaluation | Typed placeholder model in `integrations/launcher.rs` |
| **Startup** | Windows Registry `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` | Optional launch-at-login setting | Typed placeholder model in `integrations/startup.rs` |

---

## 3. Platform Limitations & Strategy

### 3.1 Media Session (GSMTC)
- **Availability:** Supported on Windows 10 (version 1809+) and Windows 11.
- **Limitation:** Only applications that integrate with Windows System Media Transport Controls (Spotify, Apple Music, browsers, etc.) will report metadata. Legacy desktop players without SMTC will not broadcast sessions.
- **Handling:** Shows a quiet empty state without disrupting layout.

### 3.2 Battery Time Remaining
- **Limitation:** Modern Windows power drivers frequently return `BATTERY_LIFE_TIME_UNKNOWN` (`0xFFFFFFFF`).
- **Handling:** Never invent estimated time remaining. Only display percentage and charging state when duration is undefined.

### 3.3 Global Download Progress
- **Limitation:** Windows does not provide a centralized operating system API to inspect browser download byte progress.
- **Handling:** The user configures watched download folders. Filesystem events observe active `.crdownload`, `.part`, or temporary file extensions and confirm completion. Progress percentages are only rendered if a genuine size signal exists.

### 3.4 Clipboard Privacy
- **Limitation:** Clipboard listeners can capture sensitive text (passwords, tokens).
- **Handling:** History capture is paused when notch is disabled or privacy mode is engaged; clipboard history is kept strictly in volatile memory and is never written to disk unless explicitly enabled.
