# File conversion

Open **More → Convert files**, or drag files onto the visible Notch. Choose a compatible output format and select **Convert**. Up to 20 files run sequentially. Conversion continues when the drawer closes; reopen it to see results or cancel.

## Filee setup

Install [Filee](https://github.com/KnifeLemon/Filee/releases) separately. Its main app does not need to remain open. Notch discovers `filee-cli.exe` through PATH, standard installation folders, and uninstall records. For portable installations, use **Locate Filee** in the drawer or **Settings → Tools → Convert files**. Choose the executable itself, rather than the `filee.cmd` wrapper. The settings section can restore automatic discovery.

Filee remains an external dependency. This repository contains no cloned or copied Filee implementation. The Notch UI, process runner, queue, and drag feedback are implemented here.

The CLI must provide `version`, `formats --json`, `formats <extension> --json`, and `convert --json`. If an update changes that contract, the drawer reports an incompatibility instead of claiming a conversion succeeded. After installing an optional engine in Filee, select **Check again** to invalidate the format cache.

## Supported files and results

- Inputs: JPG/JPEG, PNG, WebP, HEIC/HEIF, TIFF, BMP, GIF, AVIF, PDF, DOC/DOCX, XLS/XLSX, PPT/PPTX, TXT, Markdown, and CSV.
- Targets: JPG, PNG, WebP, PDF, DOCX, XLSX, CSV, TXT, and HTML, restricted to routes supported by the installed engines for every selected file.
- Documents may need additional Filee engines. Their availability and output fidelity depend on the installed engine and source document. Unsupported target routes are never enabled speculatively.
- Folders, browser URLs, audio, video, recursive conversion, and watch folders are outside this version.

Results go into `Converted` next to each original, or a destination chosen for the current selection. Each file's outputs share a newly reserved `<source>-<target>` result folder; conflicts receive a numbered suffix. Keeping multi-file results together preserves companion assets and page order. Originals are never passed a replace or move option.

**Open** opens the first result; **Folder** reveals it so all pages and companion files remain accessible. Native result opening accepts only recorded conversion outputs with supported data extensions. The UI keeps one output path per item, avoiding a retained list of thousands of PDF page paths.

The worker writes into a unique staging directory inside the destination. It validates reported outputs, rejects escaped paths and reparse points, and publishes only successful results. Cancellation and failures clean up owned staging directories. A forcibly terminated Notch or machine shutdown can leave an unpublished `.notch-convert-*` directory; it is safe to remove after no conversion is running.

## Resource policy

- No Filee process or converter polling remains alive while idle.
- One conversion/probe at a time, with a 512 MiB committed-memory limit for its Windows Job Object, including child processes.
- The process starts suspended and joins the Job Object before execution. It has no shell or visible console. Closing the job terminates its owned descendants.
- Capability probes have a 15-second timeout; each conversion has a five-minute timeout. Oversized or stalled jobs fail with a message and can be retried.
- JSON output is limited to 1 MiB, individual status lines and retained diagnostics to 64 KiB, route cache entries to 32, and recent results to 10. Current batches contain at most 20 metadata records.
- Files remain on disk. No source bytes, decoded previews, or document thumbnails enter the webview. The converter drawer loads on demand and unmounts when closed.
- Progress is forwarded at most about every 150 ms when supplied by Filee; otherwise the UI shows an indeterminate status.

The 512 MiB cap applies to converter committed memory. It is **not** the total Notch RAM budget: Rust and WebView2 processes are measured separately. Sharing a WebView2 runtime does not make its renderer and GPU processes free.

## Verification

`npm test`, `npm run check`, and `npm run build` cover frontend behavior and compilation. Run `cargo test --manifest-path src-tauri/Cargo.toml --offline` on Windows with normal file permissions. GNU library tests share Tauri's Common Controls v6 manifest so GUI imports can load.

The native converter integration tests compile a local Rust CLI fixture and exercise the actual Windows process boundary: output parsing and bounds, argument escaping, memory containment, descendant termination, and result publication. The fixture performs no real format decoding. A separate `node tests/browser-converter.mjs <screenshot-directory>` check launches an isolated Edge browser with a synthetic Tauri bridge for UI states. It does not establish real Filee engine compatibility or native Explorer drag behavior.

The implementation check passed 66 frontend tests, 20 native library tests, and 10 native converter tests. Svelte reported zero errors and warnings, and the Windows release built and started successfully. The browser check covered selection, progress/results, native-event-shaped drag/drop, result-opening commands, missing-Filee setup, and 100 drawer open/close cycles. Listener count remained at 13 and collected JavaScript heap grew by 490,308 bytes across those cycles; this measures the UI fixture, not the full desktop process tree.

For release memory sampling, run `scripts/measure-idle-memory.ps1 -Executable <release-exe> -OutputFile <results.json>`. It warms the app for 45 seconds and samples the entire owned process tree, including WebView2. Compare matching release builds and settings, and distinguish private bytes from private resident working set. No absolute RAM guarantee is inferred from a single run.
