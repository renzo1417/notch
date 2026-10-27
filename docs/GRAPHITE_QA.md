# Graphite redesign verification

## Automated checks

- Svelte and TypeScript: zero errors or warnings.
- Node behavioral tests: 50 passing, including one-time theme migration, retained custom accents, resize ordering, rapid navigation, canceled collapse, and back navigation.
- Frontend production build: passes.
- Rust compile check and native debug build: pass.
- Rust unit-test binary: compiles, but execution in the current Windows environment exits before test discovery with `0xc0000139` (`STATUS_ENTRYPOINT_NOT_FOUND`). The four added Rust migration/geometry tests have not executed successfully.

## Rendered browser checks

- Compact, idle home, active media, Focus, Clipboard, More, and Settings render without runtime errors.
- Background warnings retain the Clipboard drawer.
- Incoming download context retains an edited custom timer duration.
- A running timer does not replace an explicitly opened Clipboard drawer.
- Escape closes one level and restores focus to More, then the compact opener.
- OS reduced motion changes the effective shell duration to zero.
- Emulated scale factors 1, 1.25, 1.5, and 2 preserve drawer geometry and avoid horizontal overflow.
- Light/dark backgrounds and a 360x280 logical viewport retain readable surfaces and visible navigation.

## Native manual checks remaining

- Resolve the Windows test-loader error and execute `cargo test --offline --lib`.
- Verify actual monitor switching and Windows DPI changes at 100%, 125%, 150%, and 200%.
- Exercise real media transport, app launching, clipboard copying, and download-folder actions in the Tauri app.

Browser checks use in-memory fixture data; they do not establish native Windows integration behavior.
