# PROJECT_NOTCH

> Lightweight top-center Windows desktop companion layer.

**Current Stage:** Phase 0 — Technical Foundation Complete  
**Reference Document:** [PRODUCT_SPEC.md](./PRODUCT_SPEC.md)

---

## 1. Overview

`PROJECT_NOTCH` is an original Windows desktop companion designed to be quiet by default, expressive on interaction, and lightweight enough to run all day.

This repository implements **Phase 0 (Technical Foundation)**:
- Clean module-oriented source structure.
- Tauri 2 desktop shell with Rust backend.
- Svelte 5 frontend with TypeScript, Vite, and scoped CSS + CSS variables (no SvelteKit, no Tailwind).
- Typed placeholder models for shell state, context mode, media, battery, coding, timer, downloads, clipboard, and launcher apps.
- Working bidirectional IPC bridge (`ping` command) between Svelte 5 and Rust.
- Documented system architecture, design tokens, and Windows native integration strategies.

---

## 2. Technology Stack

- **Desktop Framework:** Tauri 2
- **Backend Runtime:** Rust (2021 edition)
- **Frontend Framework:** Svelte 5 (using modern runes: `$state`, `$derived`, `$props`)
- **Language:** TypeScript 5.7+
- **Bundler:** Vite 6+
- **Styles:** Scoped CSS + CSS Custom Properties (Tokens)
- **Architecture Invariants:** Local-first, zero telemetry, event-driven OS integration, non-crashing isolated modules.

---

## 3. Prerequisites

- **Node.js:** v18.0.0 or later (v22 LTS recommended)
- **Package Manager:** `npm`
- **Rust Toolchain:** Rust 1.78+ (`stable-x86_64-pc-windows-gnu` or `stable-x86_64-pc-windows-msvc`)
- **C/C++ Build Tools:** MinGW-w64 (via MSYS2) or Microsoft C++ Build Tools

---

## 4. Development & Build Commands

### Install Dependencies
```powershell
npm install
```

### Run Frontend Development Server (Browser preview)
```powershell
npm run dev
```

### Run Full Desktop Application (Tauri 2 Dev Mode)
```powershell
npm run tauri dev
```

### Run Frontend Typecheck
```powershell
npm run check
```

### Build Frontend Production Assets
```powershell
npm run build
```

### Verify Rust Backend Compilation
```powershell
cd src-tauri
cargo check
```

### Build Rust Debug Binary
```powershell
cd src-tauri
cargo build
```

### Build Desktop Production Package
```powershell
npm run tauri build
```

---

## 5. Repository Structure

```text
WindowsNotch/
├─ src/                         # Frontend application
│  ├─ app/                      # App root, state stores, and event contracts
│  ├─ shell/                    # Notch shell UI (Compact, Expanded, Motion)
│  ├─ mascot/                   # Mascot state models & config
│  ├─ modules/                  # Isolated domain models (music, battery, etc.)
│  ├─ settings/                 # Configuration types
│  ├─ styles/                   # Design tokens, motion variables, global reset
│  └─ main.ts                   # Svelte 5 mount entrypoint
├─ src-tauri/                   # Desktop backend
│  ├─ src/
│  │  ├─ commands/              # IPC command handlers (e.g. ping)
│  │  ├─ integrations/          # Isolated Windows integration placeholders
│  │  ├─ tray.rs                # System tray icon setup
│  │  ├─ window.rs              # Top-center window placement
│  │  └─ state.rs               # Central backend state
│  └─ Cargo.toml                # Rust dependencies and release profiles
├─ docs/                        # Engineering documentation
│  ├─ ARCHITECTURE.md           # Modular system architecture & invariants
│  ├─ DESIGN_SYSTEM.md          # Tokens, typography, motion & anti-slop rules
│  └─ WINDOWS_INTEGRATIONS.md   # Windows APIs, contracts & limitations
├─ PRODUCT_SPEC.md              # Source of truth specification
└─ README.md                    # Project documentation & dev instructions
```

---

## 6. Phase 0 Limitations

Per Phase 0 requirements:
- Visible UI is intentionally minimal to prove Svelte 5 renders and Tauri IPC communicates.
- Mascot rendering, media sessions, clipboard listeners, active downloads, timer logic, foreground coding detection, and app launching are reserved for subsequent phases (Phases 1 through 9).
