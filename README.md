# Windows Notch Desktop Companion (PROJECT_NOTCH)

A lightweight top-center Windows desktop companion layer that stays quiet by default, informative when needed, and expressive on interaction.

Built with Tauri 2, Rust, and Svelte 5.

---

## 1. Overview

PROJECT_NOTCH is an original desktop companion engineered specifically for Windows 11 and 10. Positioned seamlessly along the top-center edge of the screen, it serves as an ambient, non-intrusive operating layer rather than a conventional windowed application.

Key design goals:
- Quiet by default: Disappears visually into the screen bezel during idle periods.
- Responsive on interaction: Smoothly expands on hover or focus into compact utility panels.
- Highly optimized: Engineered with event-driven Windows APIs and lightweight canvas rendering to run continuously with minimal CPU and memory impact.
- Local-first and private: Zero telemetry, zero network tracking, no third-party cloud sync, and runs strictly with standard user privileges.

---

## 2. Technology Stack

- Desktop Shell: Tauri 2.2+
- Backend Runtime: Rust (2021 edition) utilizing official Windows crate bindings
- Frontend Framework: Svelte 5 (utilizing modern runes: `$state`, `$derived`, `$props`, `$effect`)
- Language: TypeScript 5.7+
- Bundler: Vite 6+
- Styling: Scoped CSS and CSS Custom Properties (Design Tokens)
- Testing: Node.js Native Test Runner (`node --test`) with TypeScript stripping
- Windows APIs:
  - GSMTC (`Windows.Media.Control`) for media transport and metadata
  - Win32 Event Hooks (`SetWinEventHook`) for active foreground window and coding context
  - Win32 Clipboard Format Listener (`AddClipboardFormatListener`) for history
  - Win32 Power Status (`GetSystemPowerStatus`) for battery telemetry
  - Win32 Directory Changes (`ReadDirectoryChangesW`) for download detection
  - Windows Shell and Registry APIs for app launching and user autostart

---

## 3. Core Capabilities

### Compact and Expanded Shell
- Compact Mode: Rests at the top-center edge (336x60 px) displaying the mascot, current activity summary, and active media indicators.
- Expanded Mode: Expands smoothly into contextual drawers including Home, Apps Dock, Focus Timer, Downloads, Clipboard History, and Settings.
- Dimension Transitions: Reserves shadow margins and dynamically orchestrates window resize calls to eliminate clipping and visual layout jumps.

### Interactive Mascot ("Pip")
- Procedural HTML5 Canvas rendering operating at 60 FPS during motion and resting at 0 FPS when idle.
- Physics-based squash-and-stretch dynamics on cursor interaction.
- Dynamic gaze tracking reacting to pointer coordinates.
- Contextual expressions reflecting current activities (focusing, dizzy, happy, sleeping).
- Battery saver and reduced-motion modes to halt ambient breathing redraws.

### Priority-Driven Context Engine
Orchestrates competing background events into a single unified activity feed using a strict priority ladder:
1. P1: Transient Notifications (Download complete, timer finished, low battery alerts)
2. P2: Active Focus Timer (Countdown running with pause and cancel controls)
3. P3: Active File Downloads (Progress bar, speed, and remaining file counts)
4. P4: Active Coding Session (Foreground IDE tracking and duration counter)
5. P5: Idle Ambient Mode / Media Playback (Playback state, track title, and artist)

### Native Utilities
- Media Transport: Real-time track metadata, playback progress, play/pause/skip toggles, and volume/seek controls.
- Focus Timer: Pomodoro presets and custom countdowns with drift-free timestamps.
- Downloads Monitor: Tracks active file downloads across monitored directories with open-file shortcuts.
- Clipboard History: Captures text copies with deduplication, fast search, and one-click restoration.
- Power Telemetry: Battery gauge, charging status indicators, and low-power conservation triggers.
- App Launcher: Customizable quick-launch dock for frequently used applications and tools.
- Settings Panel: Comprehensive 9-section configuration suite with accent color customization, panel toggles, and automatic schema migrations.

---

## 4. Anti-Slop Design Principles

The visual layer follows strict design tokens defined in the Graphite design specification:
- Single-layer surfaces: Clean, flat containers with subtle 1px border dividers (`rgba(255, 255, 255, 0.08)`).
- No nested cards: Eliminates "card inside a card" structures in favor of consistent padding and clean horizontal rows.
- No decorative gradients: Uses deliberate charcoal and graphite surfaces rather than saturated purple or blue novelty gradients.
- Consistent typography: Built with system typography (`Segoe UI Variable`, `Segoe UI`) with strict tabular numerals for counters and clocks.
- Unified SVG iconography: 100% vector-based geometric icons with matching 1.75px stroke widths across all modules. No emojis are used in the user interface.

---

## 5. Prerequisites

Before building or running the project locally, verify that the following tools are installed:

- Operating System: Windows 11 (recommended) or Windows 10 (version 1903 or higher)
- Node.js: v18.0.0 or later (v20+ LTS recommended)
- Package Manager: npm (included with Node.js)
- Rust Toolchain: Rust 1.78+ with the MSVC toolchain (`stable-x86_64-pc-windows-msvc`)
- C/C++ Build Tools: Microsoft C++ Build Tools (from Visual Studio 2022 or Build Tools installer)
- WebView2 Runtime: Pre-installed on Windows 11 and modern Windows 10 installations

---

## 6. Development Workflow

### 1. Install Dependencies
```powershell
npm install
```

### 2. Run Frontend in Browser (UI preview)
```powershell
npm run dev
```

### 3. Run Full Desktop Application (Tauri 2 Dev Mode)
This launches the native desktop window with live hot-reloading for both Rust and Svelte layers:
```powershell
npm run tauri dev
```

### 4. Run Frontend Typecheck
Validates all TypeScript types and Svelte component contracts:
```powershell
npm run check
```

### 5. Run Automated Test Suite
Executes unit and integration tests across context prioritization, launcher models, clipboard, timer, downloads, settings, and mascot performance:
```powershell
npm test
```

### 6. Verify Rust Backend Compilation
```powershell
cargo check --manifest-path src-tauri/Cargo.toml
```

### 7. Build Frontend Static Assets
```powershell
npm run build
```

### 8. Build Production Desktop Installer
Produces an optimized, stripped standalone Windows executable and installer package:
```powershell
npm run tauri build
```

---

## 7. Repository Structure

```text
WindowsNotch/
|-- .gitignore                  # Git ignore rules for Tauri, Rust, Vite, and Windows
|-- PRODUCT_SPEC.md             # Source-of-truth functional specification
|-- README.md                   # Project overview and development guide
|-- package.json                # Frontend dependencies and npm scripts
|-- tsconfig.json               # TypeScript compiler configuration
|-- vite.config.ts              # Vite bundler configuration
|-- svelte.config.js            # Svelte 5 compiler preprocessor setup
|-- src/                        # Svelte 5 frontend application
|   |-- main.ts                 # Application mount entrypoint
|   |-- app/                    # Shell state machines and central event dispatch
|   |-- icons/                  # Unified SVG icon components
|   |-- mascot/                 # Pip mascot canvas renderer and physics models
|   |-- modules/                # Domain-specific UI components
|   |   |-- battery/            # Battery gauge and charging views
|   |   |-- clipboard/          # Clipboard history drawer and list items
|   |   |-- coding/             # Active IDE and work context views
|   |   |-- downloads/          # Download progress and notification views
|   |   |-- launcher/           # Pinned applications dock and drawer
|   |   |-- music/              # Media transport and playback controls
|   |   |-- notifications/      # Transient toast alerts
|   |   `-- timer/              # Focus timer controls and presets
|   |-- settings/               # Settings store, schema migrations, and sections
|   |-- shell/                  # Top-level Compact, Expanded, and Panel shells
|   `-- styles/                 # Design tokens, palette definitions, and resets
|-- src-tauri/                  # Tauri 2 Rust desktop backend
|   |-- Cargo.toml              # Rust crate manifest and dependency specifications
|   |-- tauri.conf.json         # Tauri application and window layout configuration
|   |-- build.rs                # Tauri build script
|   |-- icons/                  # Application binary and tray icons
|   `-- src/
|       |-- main.rs             # Application entrypoint with Windows subsystem flags
|       |-- lib.rs              # Tauri runtime setup, plugin registration, and loop
|       |-- state.rs            # Central Rust backend state container
|       |-- tray.rs             # System tray icon and context menu handlers
|       |-- window.rs           # Top-center window placement and sizing engine
|       |-- commands/           # IPC command handlers invoked from frontend
|       `-- integrations/       # Native Windows API adapters (GSMTC, Win32, Registry)
|-- tests/                      # Automated unit and integration tests
`-- docs/                       # Detailed engineering documentation
    |-- ARCHITECTURE.md         # In-depth architectural patterns and IPC contracts
    |-- DESIGN_SYSTEM.md        # Tokens, spacing, typography, and motion standards
    |-- GRAPHITE_QA.md          # Visual audit and transition QA records
    |-- MASCOT.md               # Mascot rendering, physics, and state specifications
    |-- PERFORMANCE.md          # Benchmark results, frame rates, and memory limits
    |-- RELEASE_CHECKLIST.md    # Production deployment and verification criteria
    `-- WINDOWS_INTEGRATIONS.md # Detailed breakdown of native Win32/WinRT interfaces
```

---

## 8. Documentation References

For in-depth architectural and implementation details, refer to the documents in the `docs/` directory:

- [PRODUCT_SPEC.md](./PRODUCT_SPEC.md): Full product vision, functional specifications, and design invariants.
- [ARCHITECTURE.md](./docs/ARCHITECTURE.md): Event flow, IPC interfaces, error containment, and module isolation.
- [DESIGN_SYSTEM.md](./docs/DESIGN_SYSTEM.md): Graphite palette, typography tokens, layout scales, and spacing standards.
- [MASCOT.md](./docs/MASCOT.md): Canvas drawing loop, procedural geometry, expressions, and interaction physics.
- [PERFORMANCE.md](./docs/PERFORMANCE.md): CPU/GPU budgets, frame pacing targets, and benchmark test results.
- [WINDOWS_INTEGRATIONS.md](./docs/WINDOWS_INTEGRATIONS.md): Native Windows APIs, permission models, and fallback strategies.
- [RELEASE_CHECKLIST.md](./docs/RELEASE_CHECKLIST.md): Pre-flight verification checklist for stable production packaging.

---

## 9. Privacy and Permissions

PROJECT_NOTCH is designed from the ground up with a privacy-first architecture:
- No Network Telemetry: The application does not collect usage analytics, device identifiers, or crash reports.
- Standard User Privileges: Runs completely within standard user access tokens. It does not require administrator rights or prompt for UAC elevation.
- Local Persistence: Configuration files, clipboard cache, and application lists are stored locally on the user machine in standard application data locations.
- Non-Invasive Listeners: Uses standard Windows accessibility and shell notification hooks without injecting code or modifying external application binaries.

---

## 10. License

Private and proprietary. All rights reserved.
