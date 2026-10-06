# PROJECT_NOTCH — Windows Native Integrations

**Status:** Phase 3 GSMTC Media Integration Complete  
**Reference Document:** [PRODUCT_SPEC.md](../PRODUCT_SPEC.md)

---

## 1. Overview & Native Boundaries

All operating system interactions in `PROJECT_NOTCH` are managed through isolated native Rust modules within `src-tauri/src/integrations/`. The frontend receives typed data structures and never communicates directly with Win32/COM runtime interfaces.

### Core Implementation Rules
1. **Prefer native Windows events over polling:** Windows messages and callbacks are used whenever available (e.g. WinRT GSMTC session events, `WM_CLIPBOARDUPDATE`).
2. **Never fake missing capabilities:** If metadata, battery time, or progress is unavailable from Windows, the UI cleanly presents an honest null or partial state.
3. **Graceful degradation:** If an OS component fails or is denied permission, only that feature is disabled; the notch shell and mascot remain functional.
4. **Local-first privacy:** Zero network requests, zero telemetry, zero keystroke recording, and clipboard persistence OFF by default.

---

## 2. Integration Modules

| Module | Target Windows API / Mechanism | Purpose | Status |
| :--- | :--- | :--- | :--- |
| **Media** | `Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager` (GSMTC) | Active audio session discovery, metadata, artwork stream, transport controls, native events | **Implemented (Phase 3)** in `src-tauri/src/integrations/media.rs` |
| **Power & Battery** | Win32 `GetSystemPowerStatus` | Battery percentage, AC/DC status, charging indicator | Typed placeholder model in `integrations/power.rs` |
| **Clipboard** | Win32 `AddClipboardFormatListener` & `WM_CLIPBOARDUPDATE` | Event-driven text clipboard change notifications | Typed placeholder model in `integrations/clipboard.rs` |
| **Foreground Process** | Win32 `GetForegroundWindow` + `QueryFullProcessImageNameW` | Debounced active editor detection (VS Code, Visual Studio, etc.) | Typed placeholder model in `integrations/foreground.rs` |
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


### 3.2 Battery Time Remaining
- **Limitation:** Modern Windows power drivers frequently return `BATTERY_LIFE_TIME_UNKNOWN` (`0xFFFFFFFF`).
- **Handling:** Never invent estimated time remaining. Only display percentage and charging state when duration is undefined.

### 3.3 Global Download Progress
- **Limitation:** Windows does not provide a centralized operating system API to inspect browser download byte progress.
- **Handling:** The user configures watched download folders. Filesystem events observe active `.crdownload`, `.part`, or temporary file extensions and confirm completion. Progress percentages are only rendered if a genuine size signal exists.

### 3.4 Clipboard Privacy
- **Limitation:** Clipboard listeners can capture sensitive text (passwords, tokens).
- **Handling:** History capture is paused when notch is disabled or privacy mode is engaged; clipboard history is kept strictly in volatile memory and is never written to disk unless explicitly enabled.
