# PROJECT_NOTCH — Production Release Checklist (Phase 10)

**Document Version:** 1.0.0  
**Phase:** 10 (Production Packaging & Stabilization)  
**Binary Identity:** `PROJECT_NOTCH` (`project-notch.exe`)  
**Package Identifier:** `com.projectnotch.app`  

---

## 1. Pre-Flight Verification Checklist

### 1.1 Compiler & Diagnostic Checks
- [x] **Frontend Checks:** `npm run check` passes with 0 errors and 0 warnings across all `.svelte` and `.ts` files.
- [x] **Automated Tests:** `npm test` passes all 42 automated tests across context engine, timer, downloads, launcher, clipboard, battery, settings, and mascot performance.
- [x] **Rust Compiler:** `cargo check --manifest-path src-tauri/Cargo.toml` finishes with 0 errors and 0 warnings.
- [x] **Vite Bundle:** `npm run build` compiles clean minified static assets (`dist/`) without chunk errors or unresolved imports.
- [x] **Rust Release Build:** `cargo build --release --manifest-path src-tauri/Cargo.toml` compiles with LTO enabled, single codegen unit, symbols stripped, and `opt-level = 3`.

### 1.2 Windows Subsystem & Console Window
- [x] **No Debug Console in Release:** Configured in `src-tauri/src/main.rs`:
  ```rust
  #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
  ```
  Guarantees that launching `project-notch.exe` in release mode spawns no black terminal console window.

### 1.3 Asset & Branding Placeholders
- [x] **Application Icons:** Multi-resolution icons verified in `src-tauri/icons/`:
  - `icon.ico` (Win32 embedded binary icon)
  - `32x32.png`, `128x128.png`, `128x128@2x.png`
  - `Square30x30Logo.png` through `Square310x310Logo.png`
- [x] **Modular Replaceability:** Application identifier (`PROJECT_NOTCH`), package name, accent color, and assets remain clean and replaceable per Spec Section 2.

### 1.4 Windows Native Capabilities & Permissions
- [x] **GSMTC Media Transport:** Uses official `Windows.Media.Control` APIs; requires no elevated administrator privileges.
- [x] **Clipboard Update Listener:** Uses `AddClipboardFormatListener` / `WM_CLIPBOARDUPDATE`; standard user token.
- [x] **Power Status:** Uses `GetSystemPowerStatus`; standard user token.
- [x] **Foreground Accessibility Hook:** Uses `SetWinEventHook(EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT)`; standard user token.
- [x] **Filesystem Watcher:** Uses `ReadDirectoryChangesW` on user-selected folders; standard user permissions.
- [x] **Registry Autostart:** Reads and writes to `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`; requires zero administrator or UAC elevation.

---

## 2. Anti-AI-Slop Design Audit Checklist

Every screen and component was audited against the Product Specification Anti-AI-Slop Rules (Section 7):
- [x] **Zero Placeholder Graphics:** No unrendered dummy states or broken images.
- [x] **No Useless Decoration:** No glowing purple/blue gradients, no random decorative blobs, no generic sparkles.
- [x] **No Duplicate Containers:** Clean, flat, single-layer surfaces. No "card inside a card inside a card".
- [x] **No Unnecessary Labels:** Information hierarchy uses typography tokens rather than repetitive subtitles.
- [x] **No Emoji Icons:** Unified SVG icons with consistent stroke and geometry across all zones and actions.
- [x] **Right-Zone Music Stability:** Fixed 176px width geometry with text truncation (`ellipsis`) prevents horizontal jumping as song titles change.
- [x] **Center Zone Transition Stability:** Dynamic transitions occur smoothly without causing layout shifts.
- [x] **Mascot Balance:** Pip acts as an expressive, restrained companion occupying only 18–20% visual balance (~96px expanded, 26px compact) without dominating utilitarian controls.

---

## 3. Exact Production Build Commands

To produce a production release bundle on Windows:

### Step 1: Install & Verify Frontend Dependencies
```powershell
npm install
```

### Step 2: Run Quality Assurance Test Suite
```powershell
npm test
npm run check
```

### Step 3: Compile Frontend Production Distribution
```powershell
npm run build
```

### Step 4: Compile Native Rust Release Binary
```powershell
cargo build --release --manifest-path src-tauri/Cargo.toml
```

*(Alternatively, to invoke the full Tauri bundler:)*
```powershell
npm run tauri build
```

---

## 4. Exact Output Artifact Locations

Following a successful release build, the compiled binaries and package distributions are located at:

### 1. Standalone Release Executable
- **Path:** `src-tauri/target/release/project-notch.exe`
- **Full Path:** `D:\Mikua code\WindowsNotch\src-tauri\target\release\project-notch.exe`
- **Characteristics:**
  - Standalone Win32 executable
  - Windows subsystem: `windows` (zero console window)
  - Optimization level: `opt-level = 3`
  - Link-time optimization: `lto = true`
  - Symbol stripping: `strip = true`
  - Panic strategy: `panic = "abort"`

### 2. Frontend Compiled Bundle
- **Path:** `dist/`
- **Artifacts:**
  - `dist/index.html` (~0.40 kB)
  - `dist/assets/index-*.css` (~55.5 kB, ~6.6 kB gzip)
  - `dist/assets/index-*.js` (~165 kB, ~47.5 kB gzip)

### 3. Tauri Installer Bundle (When packaged with `tauri build`)
- **MSI / EXE Installers:** `src-tauri/target/release/bundle/msi/` and `src-tauri/target/release/bundle/nsis/`

---

## 5. Deployment & Run Instructions

To run the standalone release build:
```powershell
.\src-tauri\target\release\project-notch.exe
```
The application will launch transparently at the top-center edge of the primary monitor, register the system tray icon, and monitor media and system events without generating a command prompt console window.
