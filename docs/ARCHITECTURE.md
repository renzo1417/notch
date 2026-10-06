# PROJECT_NOTCH — System Architecture

**Status:** Phase 0 Technical Foundation  
**Reference Document:** [PRODUCT_SPEC.md](../PRODUCT_SPEC.md)

---

## 1. High-Level Architecture

`PROJECT_NOTCH` is built as an ultra-lightweight, event-driven desktop companion layer living at the top-center edge of the screen on Windows. It combines a Rust desktop backend (via Tauri 2) with a reactive Svelte 5 frontend (compiled via Vite, without SvelteKit or heavy UI framework wrappers).

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          SVELTE 5 FRONTEND                              │
│                                                                         │
│  ┌─────────────────┐   ┌────────────────────────┐   ┌────────────────┐  │
│  │   LEFT: MASCOT   │   │  CENTER: CONTEXT ZONE  │   │  RIGHT: MUSIC  │  │
│  │  (State Machine) │   │ (Deterministic Engine) │   │ (Media Status) │  │
│  └─────────────────┘   └────────────────────────┘   └────────────────┘  │
│                                                                         │
│  Shell States: dormant | compact | peek | expanded | panel | settings   │
│  CSS Tokens & Motion: Scoped CSS + CSS Custom Properties                │
└────────────────────────────────────┬────────────────────────────────────┘
                                     │ Tauri IPC (Commands & Events)
                                     ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           RUST TAURI BACKEND                            │
│                                                                         │
│  ┌───────────────┐ ┌────────────────┐ ┌───────────────┐ ┌────────────┐  │
│  │  Window/DPI   │ │      Tray      │ │ Central State │ │  Commands  │  │
│  │  Controller   │ │   Controller   │ │   Container   │ │ (IPC API)  │  │
│  └───────────────┘ └────────────────┘ └───────────────┘ └────────────┘  │
│                                                                         │
│  Native Integration Layer (Event-Driven OS Bridges):                     │
│  - Media Session (GSMTC)        - Power / Battery Status                │
│  - Clipboard Listener (Win32)   - Foreground Process Detection          │
│  - Watched Download Watcher     - Validated App Launcher                │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Directory Structure

```text
WindowsNotch/
├─ src/                         # Frontend (Svelte 5 + TypeScript + Vite)
│  ├─ app/
│  │  ├─ App.svelte             # Root component
│  │  ├─ state/
│  │  │  ├─ shell-state.ts      # Finite shell state definitions & dimensions
│  │  │  ├─ context-engine.ts   # Deterministic priority resolver
│  │  │  └─ app-state.ts        # App-level state container
│  │  └─ events/
│  │     └─ native-events.ts    # Typed OS event contract
│  ├─ shell/
│  │  ├─ NotchShell.svelte      # Root notch window layout & IPC hook
│  │  ├─ CompactNotch.svelte    # Compact mode surface
│  │  ├─ ExpandedNotch.svelte   # 3-zone expanded surface
│  │  └─ shell-motion.ts        # Timing & curve constants
│  ├─ mascot/
│  │  └─ types.ts               # Mascot state & config types
│  ├─ modules/                  # Feature domain modules
│  │  ├─ music/types.ts         # Media state types
│  │  ├─ coding/types.ts        # Coding context types
│  │  ├─ timer/types.ts         # Focus timer types
│  │  ├─ downloads/types.ts     # Downloads awareness types
│  │  ├─ clipboard/types.ts     # Clipboard history types
│  │  ├─ battery/types.ts       # Battery state types
│  │  ├─ notifications/types.ts # Internal notification types
│  │  └─ launcher/types.ts      # App launcher types
│  ├─ settings/
│  │  └─ types.ts               # User configuration types
│  ├─ styles/
│  │  ├─ tokens.css             # Color, radius, and spacing tokens
│  │  ├─ motion.css             # Timing, easing curves, and reduced motion
│  │  └─ globals.css            # Minimal zero-margin reset and base styles
│  └─ main.ts                   # Svelte 5 mount entrypoint
│
├─ src-tauri/                   # Desktop Backend (Rust + Tauri 2)
│  ├─ src/
│  │  ├─ main.rs                # Windows binary entrypoint
│  │  ├─ lib.rs                 # Tauri builder, plugins, and command handlers
│  │  ├─ commands/              # IPC command handlers
│  │  │  ├─ mod.rs
│  │  │  └─ ping.rs             # Bridge health / round-trip verification
│  │  ├─ integrations/          # Isolated OS adapters
│  │  │  ├─ mod.rs
│  │  │  ├─ media.rs            # GSMTC media session placeholder
│  │  │  ├─ power.rs            # Power status placeholder
│  │  │  ├─ clipboard.rs        # Clipboard format listener placeholder
│  │  │  ├─ foreground.rs       # Foreground IDE detection placeholder
│  │  │  ├─ downloads.rs        # Watched-folder watcher placeholder
│  │  │  ├─ launcher.rs         # Validated app launcher placeholder
│  │  │  └─ startup.rs          # Startup registry placeholder
│  │  ├─ tray.rs                # System tray icon and menu setup
│  │  ├─ window.rs              # Top-center window placement helper
│  │  └─ state.rs               # Central backend state container
│  ├─ capabilities/
│  │  └─ default.json           # Window permission declarations
│  ├─ Cargo.toml                # Rust dependencies & profile optimization
│  ├─ build.rs                  # Tauri build codegen
│  └─ tauri.conf.json           # Tauri window and packaging configuration
│
├─ docs/                        # Project technical documentation
│  ├─ ARCHITECTURE.md
│  ├─ DESIGN_SYSTEM.md
│  └─ WINDOWS_INTEGRATIONS.md
│
├─ PRODUCT_SPEC.md              # Source of truth specification
├─ package.json
├─ tsconfig.json
├─ vite.config.ts
└─ README.md
```

---

## 3. Core Architectural Invariants

### 3.1 Strict Separation of Concerns
1. **OS integrations live behind native typed modules in Rust**: The frontend never directly invokes raw OS APIs.
2. **UI never owns native integration logic**: The frontend receives typed payloads and dispatches explicit commands.
3. **Module Independence**: Every native module (`media`, `power`, `clipboard`, `downloads`, `launcher`) operates independently.
4. **Fault Isolation**: A failure in any module (e.g., media session missing or download path moved) gracefully yields a typed fallback without crashing the shell or mascot.

### 3.2 Finite State Model
The UI shell transition model is discrete:
```ts
type NotchShellState =
  | "dormant"    // Thin minimal line when idle/unfocused
  | "compact"    // Standard pill showing immediate status
  | "peek"       // Short-lived alert or confirmation glance
  | "expanded"   // Full 3-zone surface (Mascot + Context + Music)
  | "panel"      // Dedicated popover for clipboard or dock
  | "settings";  // Configuration surface
```

### 3.3 Context Priority Engine
The center zone relies on a deterministic priority hierarchy (`P0` through `P6`):
1. **P0 (Critical System Warning):** Low battery, critical alerts.
2. **P1 (Short-lived Completion):** Download finished, timer finished.
3. **P2 (Active Focus Timer):** Active countdown.
4. **P3 (Active Download):** Verified active download in progress.
5. **P4 (Active Coding Session):** Detected IDE active session.
6. **P5 (User-Triggered Surface):** Manually opened clipboard or launcher.
7. **P6 (Idle):** Default clock or quiet status.

### 3.4 Event-Driven Communication
- Integrations emit events over the Tauri IPC channel only when state changes (`media.changed`, `battery.changed`, etc.).
- Rust does not invoke the frontend on animation frames; the frontend handles UI animations locally via CSS transforms and requestAnimationFrame.

### 3.5 Privacy & Security Baseline
- **100% Local-First:** No network requests, telemetry, accounts, or analytics.
- **Process Isolation:** The launcher module only accepts split executable paths and arguments—it never passes raw command strings to a system shell (`cmd.exe` / `powershell`).
- **Clipboard Persistence:** OFF by default.

---

## 4. Phase 1 — Native Notch Shell Lifecycle & Positioning Architecture

### 4.1 Transparent Borderless Window Model
- **Borderless & Transparent:** Configured with `decorations: false`, `transparent: true`, and `shadow: false` in `tauri.conf.json`. Disabling native window shadow ensures the Windows DWM does not cast a rectangular drop-shadow box around transparent Webview2 content.
- **Taskbar Isolation:** `skipTaskbar: true` prevents the notch from displaying a standard taskbar entry.
- **Configurable Always-On-Top:** Window maintains topmost Z-order via native `window.set_always_on_top`, backed by thread-safe `AppState` configuration and frontend IPC commands.

### 4.2 Top-Center Multi-Monitor & DPI Positioning Mathematics
The shell window positioning is computed natively in Rust physical coordinates to prevent rounding drift across arbitrary Windows DPI scaling factors (100%, 125%, 150%, 175%, 200%):

$$\text{physical\_width} = \text{round}(\text{logical\_width} \times \text{scale\_factor})$$
$$\text{center\_x}_{\text{phys}} = \text{monitor\_pos}.x + \left\lfloor\frac{\text{monitor\_size}.\text{width} - \text{physical\_width}}{2}\right\rfloor$$
$$\text{top\_y}_{\text{phys}} = \text{monitor\_pos}.y$$

- **Multi-Monitor Coordinate Offsets:** By adding `monitor_pos.x` and anchoring to `monitor_pos.y`, the window centers correctly regardless of monitor arrangement, resolution differences, or negative coordinates.
- **Monitor Selection Strategy:** Supports explicit target monitor resolution with fallback chain: requested monitor $\rightarrow$ current monitor $\rightarrow$ primary monitor $\rightarrow$ first available monitor.

### 4.3 Hit-Region & Window Bounds Management (Anti-Top-Strip Invariant)
To prevent blocking underlying Windows applications (such as browser tabs and titlebars), Project Notch strictly enforces the **minimal hit-region invariant**:
1. **No Top-Strip Window:** The window never spans 100% display width.
2. **State-Synchronized Window Geometry:**
   - **Compact Resting Bounds:** Logical $332 \times 50\,\text{px}$ wrapping the $300 \times 38\,\text{px}$ pill with $16\,\text{px}$ lateral margins to accommodate the soft ambient drop-shadow (`--shadow-soft`) without hard clipping.
   - **Expanded Resting Bounds:** Logical $552 \times 88\,\text{px}$ wrapping the $520 \times 76\,\text{px}$ expanded surface.
3. **Coordinated Asymmetric Resize Sequence:**
   - **Expansion:** The native window expands immediately to $552 \times 88\,\text{px}$ and recenters, followed by Svelte CSS transition (`220ms` natural easing curve) expanding the shell element.
   - **Collapse:** Svelte initiates the CSS collapse transition (`220ms`). Once the animation completes, the native window shrinks back to $332 \times 50\,\text{px}$ and recenters.
   - **Benefit:** GPU-accelerated 60/120fps CSS animations inside stable OS bounds during motion, with minimal OS hit-testing area at resting states.

### 4.4 Display Change & Re-evaluation
- Native `WindowEvent::ScaleFactorChanged` events automatically recalculate physical dimensions and reposition the window flush to the top-center edge of the active monitor.
- IPC command `reposition_notch` allows on-demand recalibration after display reconfiguration.

### 4.5 System Tray Lifecycle & Clean Termination
- **Tray Menu Actions:**
  - `Show Notch`: Restores visibility and sets window focus.
  - `Hide Notch`: Hides the window without terminating the background process.
  - `Settings (Placeholder)`: Disabled menu placeholder reserved for Phase 9 settings.
  - `Quit`: Issues `app.exit(0)` for clean process shutdown.
- **Close Interception:** Standard `WindowEvent::CloseRequested` is intercepted via `api.prevent_close()` and redirects to `window.hide()`, guaranteeing the notch remains alive in the tray until explicit user termination.
- **Tray Icon Click:** Left-clicking the system tray icon toggles notch visibility.

