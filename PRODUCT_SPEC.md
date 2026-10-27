# Windows Notch Desktop Companion — Product & Engineering Specification

**Status:** DRAFT FOR REVIEW  
**Platform:** Windows 11 first; Windows 10 best-effort  
**Product type:** Lightweight top-center desktop companion / notch layer  
**Purpose:** This file is the source of truth for Antigravity before implementation.

> This is an original product. Coucou, Dynamic Island, and similar products may inspire interaction ideas, but their code, mascot, branding, assets, layouts, sounds, and distinctive implementation details must not be copied.

---

## 0. Instructions for Antigravity

Before implementing any phase:

1. Read this entire specification.
2. Inspect the current repository before modifying anything.
3. Implement only the requested phase.
4. Preserve completed and working functionality.
5. Prefer small, typed, modular architecture.
6. Prefer Windows event-driven APIs over polling.
7. Keep the product local-first and lightweight.
8. Avoid unnecessary dependencies.
9. Do not add analytics, telemetry, accounts, cloud sync, ads, or network services.
10. Do not invent integrations that are not technically available.
11. Document Windows limitations instead of faking functionality.
12. Follow the Anti-AI-Slop rules in this document for every UI decision.

After every implementation phase:
- run frontend checks,
- run Rust checks,
- build/run where possible,
- fix errors introduced by the phase,
- update relevant documentation,
- summarize changed files and remaining limitations.

---

# 1. Product Vision

Build an original Windows desktop companion that lives at the **top-center edge of the screen**.

It should feel like a small living layer of Windows rather than a normal application window.

The core experience combines:

- an interactive mascot,
- a context-aware center area,
- music controls in home and the companion header,
- focus timer,
- coding/activity context,
- downloads awareness,
- clipboard history,
- battery status,
- useful local notifications,
- quick app launching,
- lightweight system actions.

The product should feel:

**quiet by default, expressive on interaction, informative when something matters, and light enough to run all day.**

It must not feel like:
- a macOS clone,
- a Dynamic Island clone,
- a widget dashboard,
- a gaming overlay,
- a giant launcher,
- a browser page pretending to be a desktop app,
- generic AI-generated SaaS UI.

---

# 2. Temporary Project Identity

Working name:

`PROJECT_NOTCH`

This is temporary.

All of these must remain replaceable:
- application name,
- package identifier,
- icon,
- mascot,
- sounds,
- accent color,
- branding.

Do not use Coucou/Mochi/Apple branding.

---

# 3. Focused Companion Layout

The approved graphite redesign replaces the permanent three-zone layout.

- Compact: Pip, one activity summary, and an active media indicator.
- Home: companion header, current activity, available music controls, and Apps / Focus / More navigation.
- Tool drawers: a compact companion header above one full-width tool. Active media stays accessible through artwork and play/pause in the header.
- More opens an inline utility drawer containing Clipboard, Downloads, Battery when available, and Settings.
- Settings groups all existing fields under Appearance (appearance / mascot), Tools (music / timer / clipboard / downloads / apps), and System (general / performance).

Open drawers remain selected when context priorities change. Warnings and completions update the companion header without replacing tool content or interrupting input.

---

# 4. Context Priority Engine

The center panel must use explicit priority rules.

Recommended order:

```text
P0  important system warning
P1  short-lived completion / confirmation
P2  active focus timer
P3  active download
P4  active coding context
P5  manually opened clipboard / launcher
P6  idle context
```

Example:

```text
Timer running + music playing

LEFT             CENTER              RIGHT
Mascot           Focus 18:42         Current Song
```

Example:

```text
Coding + music

LEFT             CENTER              RIGHT
Mascot           VS Code · 42m       Current Song
```

Example:

```text
Download finishes

LEFT             CENTER              RIGHT
Celebrate        Download complete   Current Song
```

After several seconds, the center returns to the previous meaningful context.

The priority system must be centralized and deterministic.

---

# 5. Notch State Model

Use a finite state model.

Suggested shell states:

```ts
type NotchShellState =
  | "dormant"
  | "compact"
  | "peek"
  | "expanded"
  | "panel"
  | "settings";
```

Suggested center modes:

```ts
type ContextMode =
  | "idle"
  | "coding"
  | "timer"
  | "download"
  | "clipboard"
  | "notification"
  | "launcher"
  | "battery";
```

Avoid many overlapping boolean flags.

---

# 6. Visual Direction

The UI should feel:
- premium,
- restrained,
- tactile,
- compact,
- calm,
- desktop-native,
- highly polished.

Outstanding design does **not** mean visually loud.

The notch should look designed intentionally for Windows rather than generated from a component library.

---

# 7. Anti-AI-Slop Rules

These are hard rules.

Do not create:
- giant rounded cards inside giant rounded cards,
- excessive glassmorphism,
- purple/blue AI gradients,
- neon glowing borders,
- random decorative blobs,
- fake analytics charts,
- large empty hero areas,
- unnecessary subtitles under every label,
- oversized icons,
- emojis as production icons,
- excessive pill buttons,
- every item inside a separate container,
- meaningless badges,
- generic AI sparkles,
- SaaS sidebars,
- dashboard grids,
- giant headings,
- gradient text,
- excessive shadows,
- constant bouncing,
- decorative animation with no purpose.

Do not make it look like:
- a mobile app stretched horizontally,
- a website navbar,
- an admin dashboard,
- an Apple screenshot traced pixel-for-pixel.

Every visible element must have a functional purpose.

---

# 8. Design System

## Base palette

Default theme: dark.

Suggested tokens:

```css
--surface-root: rgba(21, 22, 24, 0.96);
--surface-raised: #202226;
--surface-hover: rgba(255, 255, 255, 0.07);

--text-primary: #eceef1;
--text-secondary: #adb1b8;
--text-tertiary: #858a94;

--border-subtle: rgba(255, 255, 255, 0.08);
--shadow-soft: rgba(0, 0, 0, 0.30);
```

Accent defaults to soft silver `#d8dce2`; custom accents remain configurable. Use muted amber `#e4b46a` for warnings and muted red `#df9191` for errors. Ordinary activity categories use icons and neutral text.

## Typography

Preferred:

`Segoe UI Variable`

Fallback:

`Segoe UI, Inter, system-ui, sans-serif`

Suggested sizes:
- main track/timer: 13–14px semibold,
- primary label: 12–13px medium,
- metadata: 10–12px,
- tiny status: 10–11px.

Avoid large headings.

## Iconography

Use one consistent SVG icon family.

Typical size:
- 16–18px.

No emoji icons.

## Radius

Use a small radius system:
- shell: 16–22px,
- inner controls: 8–12px,
- circular icon buttons only where appropriate.

## Spacing

Use:
`4, 6, 8, 12, 16, 20, 24`

Avoid arbitrary spacing values unless animation geometry requires them.

---

# 9. Motion Design

Motion communicates state, not decoration.

Use:
- opacity,
- scale,
- transform,
- width/height interpolation,
- masked reveals,
- restrained spring-like easing.

Avoid:
- exaggerated overshoot,
- constant floating,
- spinning decorations,
- animation on every element.

Typical timing:
- micro interaction: 100–160ms,
- shell transition: 180–260ms,
- context transition: 160–220ms,
- mascot expression: 250–700ms.

Respect reduced-motion preferences where practical.

---

# 10. Compact State

Compact is 304 × 44 logical pixels. It shows Pip, one current summary, and an active media indicator. Click or Enter/Space opens home. Hover changes Pip's expression without expanding. Background events never auto-expand the shell.

---

# 11. Expanded Home and Drawers

| State | Visible size | Native window including shadow margins |
|---|---|---|
| Compact | 304 × 44 | 336 × 60 |
| Home | 480 × 160 | 512 × 176 |
| Apps, Focus, Clipboard, Downloads, Battery, More | 480 × 328 | 512 × 344 |
| Settings | 640 × 456 | 672 × 472 |

All shells stay attached to the top edge, with square top corners and 18px lower corners. Retain 16px transparent space at each side and below; clamp the native window to the selected monitor in logical units. Content scrolls within constrained heights.

Back or Escape returns from a drawer/settings to home; Escape on home collapses. Restore focus to the opening control. Home auto-collapses after the configured delay after pointer exit; keyboard interaction pauses this timer. Drawers and settings stay open until dismissed.

Native bounds grow before content expands and shrink after the effective CSS transition. Serialize native resizes and cancel pending shrink delays when a newer request arrives. Reduced motion makes these delays zero.

---

# 12. Mascot

The mascot must be original.

## MVP rendering recommendation

Use:
- Canvas 2D,
- procedural drawing and/or tiny vector assets,
- no video,
- no large sprite sheet,
- no heavy mascot runtime.

Rive can be evaluated later only if profiling proves it worthwhile.

## Mascot state model

```ts
type MascotState =
  | "idle"
  | "blink"
  | "look"
  | "curious"
  | "happy"
  | "focused"
  | "sleepy"
  | "celebrate"
  | "warning"
  | "dizzy";
```

## Behaviors

Idle:
- subtle breathing,
- irregular blink cadence,
- slight gaze movement.

Cursor:
- pupils follow cursor,
- clamp movement realistically,
- use interpolation rather than snapping.

Hover:
- subtle curiosity.

Click:
- quick squash/recovery.

Rapid clicks:
- dizzy reaction,
- cooldown prevents animation spam.

Focus timer:
- focused expression.

Download complete:
- celebration.

Battery low:
- subtle worried state.

Long inactivity:
- sleepy.

The mascot must not continuously consume CPU when nothing changes.

---

# 13. Music Module

Goal: show and control the current compatible Windows media session.

Preferred Windows API family:

`Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager`

Expected information when available:
- title,
- artist,
- album,
- artwork,
- playing/paused,
- playback controls,
- timeline.

Never fake missing metadata.

UI:

```text
[art] Track title
      Artist
      ◀   ▶/❚❚   ▶
```

The right-side geometry should remain stable while tracks change.

---

# 14. Coding Context

The center should understand when the user is coding without spying on content.

## MVP detection

Foreground application/process only:
- Visual Studio Code,
- Visual Studio,
- common JetBrains IDEs,
- terminal apps,
- Antigravity if its process/window can be identified,
- optionally developer browser sessions later.

Possible center UI:

```text
CODE
VS Code · ReadyApp
42 min
```

The user may manually name a coding session/project.

Do not:
- record keystrokes,
- read code contents,
- create screenshots,
- upload activity,
- keep detailed activity history by default.

Later optional adapters may expose:
- project/workspace,
- git branch,
- build status,
- dev server state.

Those adapters must be modular.

---

# 15. Focus Timer

Built-in local timer.

Presets:
- 25 minutes,
- 50 minutes,
- custom.

Controls:
- start,
- pause,
- resume,
- stop/reset.

Example:

```text
FOCUS
24:31
Pause   Stop
```

Rules:
- continues while notch is collapsed,
- timer context outranks coding context,
- mascot changes to focused state,
- completion creates a local notch notification.

Do not turn this into a full productivity platform.

---

# 16. Downloads Awareness

Generic global browser download progress is not always available through one reliable Windows API.

MVP strategy:
- user chooses watched download folders,
- use filesystem events,
- detect new/partial/completed files where practical,
- show exact percentage only when a trustworthy progress signal exists,
- always support completion detection.

Example:

```text
Downloading
assets.zip
68%
```

If progress is unknown:

```text
Downloading…
assets.zip
```

Never fabricate percentages.

Completion:

```text
Download complete
assets.zip
Open folder
```

---

# 17. Clipboard Module

Use Windows clipboard change notifications instead of polling.

Potential native mechanism:
`AddClipboardFormatListener` / `WM_CLIPBOARDUPDATE`.

MVP:
- text items only,
- recent history,
- configurable maximum,
- clear history,
- pause capture.

Privacy:
- persistence OFF by default,
- never upload,
- never log clipboard contents,
- no clipboard history when disabled.

Clipboard should not constantly hijack the center. It is mainly opened deliberately by the user.

---

# 18. Battery Module

Use native Windows power state.

Potential native API:
`GetSystemPowerStatus`.

Display:
- percentage,
- charging,
- AC/battery,
- estimated time only when Windows actually provides it.

Example:

```text
Laptop
72%
3h 42m remaining
```

Never fabricate time remaining.

---

# 19. Notification System

Two categories:

## A. Internal events — MVP

Implement:
- timer complete,
- download complete,
- application launched,
- invalid application path,
- low battery,
- settings confirmation.

## B. Windows notification mirroring — later research

Do not block MVP on full Windows notification-center mirroring.

Keep the architecture capable of supporting a future notification provider.

Example:

```text
mascot peeks
Download complete
ready-assets.zip
```

---

# 20. App Launcher

The notch can launch configured Windows applications.

MVP:
- Add App,
- choose executable or supported shortcut,
- save it locally,
- extract/use app icon where practical,
- reorder,
- remove,
- launch.

Suggested model:

```ts
type LaunchableApp = {
  id: string;
  name: string;
  executablePath: string;
  args: string[];
  iconPath?: string;
  enabled: boolean;
  order: number;
};
```

Security:
- never execute arbitrary shell text,
- executable and arguments remain separate,
- validate paths,
- gracefully handle moved/deleted programs.

Cap.io should be addable through this normal system.

Do not hard-code the product around Cap.io.

---

# 21. Quick App Dock

Expanded notch may show:

```text
Apps
[Cap.io] [VS Code] [Browser] [+]
```

Rules:
- 4–6 pinned apps maximum before overflow,
- compact,
- icon-led,
- no giant application grid,
- no marketplace.

---

# 22. Settings

Possible dedicated settings window or notch panel.

Sections:

### General
- launch at startup,
- monitor,
- always on top,
- auto-collapse delay.

### Appearance
- accent,
- opacity within safe limits,
- animation intensity,
- reduced motion.

### Mascot
- enable,
- cursor tracking,
- animation intensity,
- sound toggle if sound is later added.

### Music
- enable,
- controls.

### Timer
- presets.

### Clipboard
- enable,
- history limit,
- persistence,
- clear.

### Downloads
- watched folders.

### Apps
- add/remove/reorder.

### Performance
- battery-saver mode,
- pause inactive animation.

Do not overbuild Settings.

---

# 23. Recommended Tech Stack

## Desktop shell
**Tauri 2**

Why:
- suitable for a small always-running utility,
- native packaging,
- Rust backend,
- uses the system web renderer instead of bundling a full Electron Chromium runtime.

## Frontend
**Svelte 5 + TypeScript + Vite**

Use plain Svelte, not SvelteKit.

Why:
- strong component model,
- compact compiled UI,
- good fit for state-driven desktop surfaces.

## Styling
**Scoped CSS + CSS custom properties**

Do not use Tailwind initially.

Why:
- explicit design tokens,
- easier pixel-level desktop UI refinement,
- less utility-class noise.

## Backend
**Rust**

Responsibilities:
- Windows window behavior,
- music/media sessions,
- battery,
- clipboard events,
- foreground process detection,
- app launching,
- startup,
- filesystem watchers,
- system tray.

## Windows bindings
Use current maintained Rust Windows bindings, typically the `windows` crate, where native APIs are required.

## Persistence
Use a lightweight typed local config:
- Tauri Store or small JSON config.

Do not add a database unless later features justify it.

## Networking
None for MVP.

---

# 24. Architecture Principles

1. OS integrations live behind native typed modules.
2. UI never directly owns native integration logic.
3. Modules are independent.
4. Every module can fail without crashing the shell.
5. Every module can be disabled.
6. Prefer events over polling.
7. No internet dependency.
8. No mandatory account.
9. No telemetry.
10. No hidden helper processes unless technically required and documented.

---

# 25. Proposed Source Structure

```text
project-notch/
├─ src/
│  ├─ app/
│  │  ├─ App.svelte
│  │  ├─ state/
│  │  │  ├─ shell-state.ts
│  │  │  ├─ context-engine.ts
│  │  │  └─ app-state.ts
│  │  └─ events/
│  │
│  ├─ shell/
│  │  ├─ NotchShell.svelte
│  │  ├─ CompactNotch.svelte
│  │  ├─ ExpandedNotch.svelte
│  │  └─ shell-motion.ts
│  │
│  ├─ mascot/
│  │  ├─ MascotCanvas.svelte
│  │  ├─ renderer.ts
│  │  ├─ state-machine.ts
│  │  └─ behaviors/
│  │
│  ├─ modules/
│  │  ├─ music/
│  │  ├─ coding/
│  │  ├─ timer/
│  │  ├─ downloads/
│  │  ├─ clipboard/
│  │  ├─ battery/
│  │  ├─ notifications/
│  │  └─ launcher/
│  │
│  ├─ settings/
│  ├─ components/
│  ├─ icons/
│  ├─ styles/
│  │  ├─ tokens.css
│  │  ├─ globals.css
│  │  └─ motion.css
│  └─ main.ts
│
├─ src-tauri/
│  ├─ src/
│  │  ├─ main.rs
│  │  ├─ commands/
│  │  ├─ integrations/
│  │  │  ├─ media.rs
│  │  │  ├─ power.rs
│  │  │  ├─ clipboard.rs
│  │  │  ├─ foreground.rs
│  │  │  ├─ downloads.rs
│  │  │  ├─ launcher.rs
│  │  │  └─ startup.rs
│  │  ├─ tray.rs
│  │  ├─ window.rs
│  │  └─ state.rs
│  ├─ capabilities/
│  ├─ icons/
│  └─ tauri.conf.json
│
├─ docs/
│  ├─ ARCHITECTURE.md
│  ├─ DESIGN_SYSTEM.md
│  ├─ PERFORMANCE.md
│  └─ WINDOWS_INTEGRATIONS.md
│
├─ PRODUCT_SPEC.md
├─ package.json
└─ README.md
```

Antigravity may simplify the structure only when there is a concrete reason.

---

# 26. Native Event Contract

Prefer event-driven communication.

Conceptual frontend events:

```ts
type NativeEvent =
  | { type: "media.changed"; payload: MediaState }
  | { type: "battery.changed"; payload: BatteryState }
  | { type: "clipboard.changed"; payload: ClipboardItem }
  | { type: "foreground.changed"; payload: ForegroundApp }
  | { type: "download.changed"; payload: DownloadState }
  | { type: "download.completed"; payload: DownloadState };
```

Do not call Rust every animation frame.

---

# 27. Performance Requirements

This application should remain running all day.

Priorities:
- zero unnecessary network traffic,
- no busy loops,
- event-driven integrations,
- animation only while needed,
- no continuous Canvas redraw while static,
- pause inactive work in dormant state,
- debounce foreground changes,
- lazy-load infrequent panels,
- avoid large assets,
- avoid multiple UI frameworks,
- clean up listeners/watchers,
- cache expensive icon extraction.

Create development diagnostics for:
- FPS during animation,
- active timers,
- active watchers,
- shell state.

Diagnostics must not clutter production UI.

---

# 28. Accessibility

Minimum:
- keyboard-accessible controls,
- accessible labels,
- logical tab order,
- sufficient contrast,
- reduced motion support where practical,
- no control communicated only through color.

The mascot must never block functional keyboard navigation.

---

# 29. Multi-Monitor and DPI

Support:
- 100%,
- 125%,
- 150%,
- 175%,
- 200% scale.

Initial behavior:
- selected monitor,
- top-center placement.

Possible later option:
- follow active monitor.

Do not assume physical pixels equal CSS pixels.

---

# 30. Window Behavior

Desired:
- transparent,
- borderless,
- no normal title bar,
- no ordinary taskbar button for the notch,
- configurable always-on-top,
- top-center,
- small hit area,
- tray available,
- hide does not quit,
- explicit Quit terminates.

Do not create a huge invisible window across the top of the display.

---

# 31. Failure States

Every integration needs a graceful fallback.

Examples:

Media unavailable:
`Nothing playing`

Battery remaining time unknown:
show percentage only.

Deleted app:
`App not found`
with `Locate` and `Remove`.

Clipboard disabled:
show disabled/privacy state.

Download folder inaccessible:
offer to select another folder.

Foreground detection failure:
return to idle context.

A module failure must not crash the mascot or shell.

---

# 32. Privacy Rules

Default: local-only.

Do not implement:
- analytics,
- telemetry,
- cloud clipboard,
- cloud sync,
- screenshots,
- screen recording,
- keylogging,
- typed-code capture,
- application-activity upload.

Foreground app detection should retain only what is needed for current UI unless future history is explicitly enabled.

Clipboard persistence is OFF by default.

---

# 33. MVP Scope

## Shell
- clean Windows launch,
- transparent top-center notch,
- compact/expanded states,
- correct monitor positioning,
- tray,
- settings persistence.

## Mascot
- original design,
- idle/blink/look,
- cursor awareness,
- timer/download reactions.

## Music
- current compatible Windows media session,
- metadata where provided,
- playback controls.

## Center context
- explicit priority engine,
- coding state,
- timer,
- internal notifications.

## Launcher
- add,
- pin,
- launch,
- remove,
- broken-path handling.

## Battery
- percentage,
- charging state.

## Clipboard
- event listener,
- text history,
- privacy controls.

## Downloads
- watched folders,
- completion awareness,
- exact progress only where genuinely known.

Full Windows notification mirroring is not MVP.

---

# 34. Post-MVP Ideas

Only after MVP is stable and profiled:

- global hotkey,
- command palette,
- drag-file quick actions,
- richer IDE adapters,
- Git branch/build status,
- microphone/camera privacy indicator,
- Bluetooth battery where feasible,
- meeting/calendar timer,
- audio output selector,
- network indicator,
- quick handoff to Cap.io,
- optional Rive mascot,
- provider/plugin architecture,
- themes,
- follow-active-monitor,
- Windows notification provider.

These are ideas, not requirements.

---

# 35. Explicitly Out of Scope Initially

Do not implement unless this spec changes:

- AI chat assistant,
- LLM API,
- cloud sync,
- login/account,
- social features,
- screenshot engine,
- video recorder,
- OCR,
- browser extension,
- file manager,
- task manager replacement,
- weather,
- news,
- crypto,
- fake system metrics,
- analytics dashboard,
- plugin marketplace.

---

# 36. Development Phases

## Phase 0 — Foundation
Create clean Tauri + Svelte + TypeScript foundation.

## Phase 1 — Native Notch Shell
Perfect Windows window behavior before features.

## Phase 2 — Visual System + Mascot
Create the distinctive UI and original mascot.

## Phase 3 — Music
Implement the persistent right-side media module.

## Phase 4 — Context Engine + Coding
Build the center priority engine and coding context.

## Phase 5 — Focus Timer
Add the first persistent productivity context.

## Phase 6 — App Launcher
Add configurable app launching and compact dock.

## Phase 7 — Clipboard + Battery
Add lightweight native utilities.

## Phase 8 — Downloads
Add watched-folder download awareness.

## Phase 9 — Settings + Startup
Make the app comfortable for daily use.

## Phase 10 — Performance + QA + Packaging
Stabilize for all-day use.

---

# 37. Definition of Done for Every Phase

A phase is not done until:

1. App compiles.
2. Rust check passes.
3. Frontend checks pass.
4. No obvious console errors.
5. Existing completed features still work.
6. New listeners/watchers clean themselves up.
7. No unnecessary dependency was added.
8. UI obeys this design specification.
9. Failure states are handled.
10. Relevant documentation is updated.

---

# 38. Code Quality Rules

- Strong typing across module boundaries.
- Avoid `any`.
- Small focused modules.
- No giant Svelte component.
- No giant Rust `main.rs`.
- No duplicate state.
- Centralize design tokens.
- Centralize context priority.
- Centralize animation timing.
- Prefer pure functions for priority decisions.
- Comment reasoning, not obvious syntax.
- Remove abandoned experiments.
- Never leave generated placeholder UI in final phases.

---

# 39. UI Review Checklist

Before completing any UI-heavy phase ask:

- Is the notch still compact?
- Is there unnecessary text?
- Are cards nested without reason?
- Are there too many rounded rectangles?
- Is accent color restrained?
- Are icons consistent?
- Is spacing token-based?
- Does every animation communicate something?
- Does long music metadata still fit?
- Does it work at 125% and 150% scaling?
- Does center context change without making music jump around?
- Is the mascot supportive rather than dominant?
- Does it feel like a Windows desktop product instead of a generated web dashboard?

If not, refine it before continuing.

---

# 40. Engineering Notes

Where practical, verify and prefer current official Windows APIs.

Media:
- Global System Media Transport Controls for compatible sessions.

Clipboard:
- event-driven clipboard update listener.

Battery:
- `GetSystemPowerStatus`.

The coding agent must verify exact bindings and current crate APIs before implementation.

Do not blindly use outdated snippets.

---

# 41. Final Product Principle

The notch should disappear from attention when nothing matters and become useful immediately when something does.

**Quiet by default. Expressive on interaction. Useful on events. Lightweight enough to run all day.**
