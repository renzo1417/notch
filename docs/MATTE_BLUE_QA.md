# Matte Blue verification and native checklist

## Automated verification

- Svelte/TypeScript check: zero errors and zero warnings.
- Frontend behavioral suite: 59 passing tests, covering legacy settings/pins, surface migration, capture defaults, nested navigation, app identities/search, and existing utilities.
- Frontend production build: passes.
- Native `cargo check --offline`: passes with the Windows capture/D3D11 feature flags. No additional crates were introduced.
- Native debug build: passes.
- Eleven tests of actual production Rust modules pass through standalone `rustc --test` wrappers: seven capture geometry/conversion/request-cancellation tests and four launcher persistence/identity tests. This includes cancellation before worker startup, stale cancellation isolation, bounded request bookkeeping, and preserving corrupt saved launcher data.
- `cargo test --offline --lib`: test executable builds but exits before discovery with `0xc0000139` (`STATUS_ENTRYPOINT_NOT_FOUND`). The complete Tauri-linked suite, including Rust settings migrations, must not be reported as passed; the standalone results above are separate. This loader failure also occurred before these changes.

## Native launcher smoke verification

A read-only executable importing the production launcher module found 222 installed entries: 166 Start Menu shortcuts and 56 packaged apps, with zero discovery warnings. Shortcut and packaged-app icons loaded, and a cached query matched. Packaged existence checks accepted a real installed app and rejected a bogus application ID using temporary launcher fixtures. No applications were launched and real launcher data was not changed.

## Rendered browser verification

An isolated local browser profile and in-memory native-command fixtures were used; no real clipboard, installed apps, media sessions or device capture were accessed by these checks.

- Compact, home, four-item navigation, Installed apps, Capture, utilities and Settings render without runtime exceptions.
- Search/add shows Added state; replacement retains a pin and reopening normal Add remains usable.
- Picker Back/Escape returns to Apps or the opening Settings category and restores the triggering control.
- Switching Apps to Focus and returning Home focuses Focus rather than a stale Apps opener.
- Visited panels retain edited timer input while navigating and during a background warning.
- Utility Escape returns to More, then Home, then compact, with appropriate focus restoration.
- Capture requests carry unique IDs through cancellation so queued and stale cancellation cannot affect the wrong operation. Capture success is reported only on a copied result. Cancellation stays in Capture. Failed capture unlocks navigation and displays an inline error. Controls are disabled while a request is pending.
- The main Capture action fits the normal drawer; constrained displays scroll tool content and retain navigation.
- Matte is fully opaque; selecting Translucent reveals the opacity slider; switching back and forth preserves the saved opacity.
- OS reduced motion yields zero shell transition duration.
- Emulated 100%, 125%, 150%, and 200% scale factors preserve drawer dimensions. A 360 × 280 logical viewport retains all four navigation destinations without horizontal overflow.

Browser fixtures establish frontend behavior only. Native discovery was verified separately above; activation, screen capture, and clipboard integration were not exercised.

## Native manual acceptance checklist — not yet exercised

- [ ] Open the native application and confirm matte defaults, custom accent preservation, and schema-3 settings round trips.
- [ ] Discover and launch desktop and Microsoft Store apps, plus a portable executable through Browse.
- [ ] Launch a shortcut with quoted arguments and a custom working directory.
- [ ] Check duplicate names, missing icons, partial catalog errors, uninstall/repair, and pin order/enabled-state preservation.
- [ ] Capture Area, Window, and Screen, then paste each into Paint and another image-capable application.
- [ ] Confirm the notch and selection overlay are absent from copied images.
- [ ] Exercise mouse Copy/Cancel, Enter/Escape, Tab, arrows, Shift+arrows, empty selection, and cancellation before copy.
- [ ] Test clipboard contention, protected content, unsupported capture, closed/minimized target windows, display disconnects and repeated capture requests.
- [ ] Verify real mixed-DPI/secondary/negative-coordinate monitors at 100–200% scaling and inspect HDR-to-SDR color appearance.
- [ ] Confirm resources are released after repeated captures and no capture session runs while idle.
- [ ] Verify real media controls, timer completion, text clipboard history, watched downloads and battery behavior.
- [ ] Resolve the Windows test-loader failure and execute the complete Rust suite.

## Boundaries and implementation notes

The landing page was not modified. Installed-app catalogs/icons are memory-only and loaded on request. Launcher files that cannot be read or parsed are protected from subsequent writes, avoiding silent loss of saved pins. Legacy bare-array launcher files and executable argument arrays remain supported.

Screenshot capture is explicit and one-shot. Pixels remain native, CF_DIBV5 image ownership transfers to Windows, and the text-history module ignores images. Area selection uses a frozen monitor image after the capture session closes. Windows indicators/restrictions remain intact; protected content may be blank or unavailable. There is no screenshot gallery, automatic saving, OCR, annotation, recording, cloud service, or global shortcut.
