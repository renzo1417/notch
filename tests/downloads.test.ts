import test from "node:test";
import assert from "node:assert/strict";
import {
  ContextPriority,
  ContextEngine,
} from "../src/app/state/context-engine.ts";
import type { DownloadState } from "../src/modules/downloads/types.ts";

/**
 * Mirror of backend partial download extension recognition logic.
 */
const KNOWN_PARTIAL_EXTENSIONS = [
  ".crdownload",
  ".part",
  ".download",
  ".!ut",
  ".mega",
  ".tmp",
  ".temp",
];

function extractPartialTargetName(fileName: string): string | null {
  const lower = fileName.toLowerCase();
  for (const ext of KNOWN_PARTIAL_EXTENSIONS) {
    if (lower.endsWith(ext) && fileName.length > ext.length) {
      const base = fileName.slice(0, fileName.length - ext.length);
      if (base.length > 0) {
        return base;
      }
    }
  }
  return null;
}

function formatDownloadCompact(state: DownloadState): string {
  if (state.isCompleted) {
    return `Complete · ${state.fileName}`;
  }
  if (state.progressPercent !== undefined && state.progressPercent !== null) {
    return `${state.fileName} · ${state.progressPercent}%`;
  }
  return `Downloading… · ${state.fileName}`;
}

test("Downloads Heuristics: extracts correct target filename from partial download extensions", () => {
  // Chromium / Edge / Chrome
  assert.equal(extractPartialTargetName("archive.zip.crdownload"), "archive.zip");
  assert.equal(extractPartialTargetName("complex.name.with.dots.tar.gz.crdownload"), "complex.name.with.dots.tar.gz");

  // Firefox / Mozilla
  assert.equal(extractPartialTargetName("ubuntu-24.04-desktop-amd64.iso.part"), "ubuntu-24.04-desktop-amd64.iso");

  // Safari / WebKit
  assert.equal(extractPartialTargetName("setup-installer.exe.download"), "setup-installer.exe");

  // BitTorrent
  assert.equal(extractPartialTargetName("dataset.parquet.!ut"), "dataset.parquet");

  // Mega
  assert.equal(extractPartialTargetName("video.mp4.mega"), "video.mp4");

  // Temp files
  assert.equal(extractPartialTargetName("curl_download.bin.tmp"), "curl_download.bin");
  assert.equal(extractPartialTargetName("curl_download.bin.temp"), "curl_download.bin");

  // Normal files should return null (not partials)
  assert.equal(extractPartialTargetName("already_complete.zip"), null);
  assert.equal(extractPartialTargetName("document.pdf"), null);
  assert.equal(extractPartialTargetName("song.mp3"), null);
});

test("Downloads Spec Invariant: never fabricates progress percentages", () => {
  // Requirement 5 & 6: When total size is unknown, percentage is undefined/null and shows Downloading…
  const unknownProgressState: DownloadState = {
    fileName: "assets.zip",
    progressPercent: undefined,
    isCompleted: false,
    filePath: "C:\\Users\\User\\Downloads\\assets.zip",
  };

  assert.equal(unknownProgressState.progressPercent, undefined);
  assert.equal(formatDownloadCompact(unknownProgressState), "Downloading… · assets.zip");

  // Trustworthy progress only when explicitly supplied
  const reliableProgressState: DownloadState = {
    fileName: "assets.zip",
    progressPercent: 68,
    isCompleted: false,
    filePath: "C:\\Users\\User\\Downloads\\assets.zip",
  };

  assert.equal(formatDownloadCompact(reliableProgressState), "assets.zip · 68%");

  // Completed state
  const completedState: DownloadState = {
    fileName: "assets.zip",
    isCompleted: true,
    filePath: "C:\\Users\\User\\Downloads\\assets.zip",
  };

  assert.equal(formatDownloadCompact(completedState), "Complete · assets.zip");
});

test("Downloads Context Engine: active download (P3) outranks coding (P4) and user surface (P5)", () => {
  const engine = new ContextEngine();

  // Baseline coding session at P4
  engine.registerCandidate({
    id: "coding",
    mode: "coding",
    priority: ContextPriority.P4_CODING,
    timestamp: 1000,
  });
  assert.equal(engine.currentMode, "coding");

  // Active download appears at P3
  engine.registerCandidate({
    id: "download",
    mode: "download",
    priority: ContextPriority.P3_DOWNLOAD,
    timestamp: 2000,
  });
  // P3 > P4 => active download wins the center context
  assert.equal(engine.currentMode, "download");

  // User opens clipboard at P5
  engine.registerCandidate({
    id: "clipboard",
    mode: "clipboard",
    priority: ContextPriority.P5_USER_SURFACE,
    timestamp: 3000,
  });
  // Active download (P3) still outranks manually opened P5 surface
  assert.equal(engine.currentMode, "download");

  // When active download finishes or is dismissed, reveals coding
  engine.unregisterCandidate("download");
  assert.equal(engine.currentMode, "coding");
});

test("Downloads Context Engine: active focus timer (P2) outranks active download (P3)", () => {
  const engine = new ContextEngine();

  // Active download at P3
  engine.registerCandidate({
    id: "download",
    mode: "download",
    priority: ContextPriority.P3_DOWNLOAD,
    timestamp: 1000,
  });
  assert.equal(engine.currentMode, "download");

  // Active focus timer at P2
  engine.registerCandidate({
    id: "timer",
    mode: "timer",
    priority: ContextPriority.P2_FOCUS_TIMER,
    timestamp: 2000,
  });
  // P2 > P3 => timer wins
  assert.equal(engine.currentMode, "timer");

  // Timer stops => download re-emerges
  engine.unregisterCandidate("timer");
  assert.equal(engine.currentMode, "download");
});

test("Downloads Completion: P1 completion notification temporarily outranks timer and returns cleanly", (t) => {
  const engine = new ContextEngine();

  // Timer running at P2
  engine.registerCandidate({
    id: "timer",
    mode: "timer",
    priority: ContextPriority.P2_FOCUS_TIMER,
    timestamp: 1000,
  });
  assert.equal(engine.currentMode, "timer");

  // Download completes: triggers P1 notification for 50ms
  const notifId = engine.pushTemporaryContext({
    id: "download_completed_notif",
    mode: "notification",
    priority: ContextPriority.P1_COMPLETION,
    durationMs: 50,
    data: { title: "Download complete", message: "ready-assets.zip" },
  });

  // P1 outranks P2 timer during notification window
  assert.equal(engine.currentMode, "notification");

  return new Promise<void>((resolve) => {
    setTimeout(() => {
      // After expiration, returns seamlessly to timer
      assert.equal(engine.currentMode, "timer");
      engine.destroy();
      resolve();
    }, 70);
  });
});

test("Downloads Deduplication: debounces rapid filesystem event emissions", () => {
  let emittedCount = 0;
  let lastEmittedMs = 0;
  const DEBOUNCE_WINDOW_MS = 1000;

  function simulateFsChangeEvent(nowMs: number) {
    if (nowMs - lastEmittedMs >= DEBOUNCE_WINDOW_MS) {
      lastEmittedMs = nowMs;
      emittedCount++;
    }
  }

  // Rapid chunk writes at 0ms, 100ms, 250ms, 500ms, 800ms
  simulateFsChangeEvent(1000);
  simulateFsChangeEvent(1100);
  simulateFsChangeEvent(1250);
  simulateFsChangeEvent(1500);
  simulateFsChangeEvent(1800);
  // Only the first event emitted
  assert.equal(emittedCount, 1);

  // Next write after 1100ms (> 1000ms debounce threshold)
  simulateFsChangeEvent(2105);
  assert.equal(emittedCount, 2);

  // Duplicate rename / finalize event at 2150ms should be dropped
  simulateFsChangeEvent(2150);
  assert.equal(emittedCount, 2);
});
