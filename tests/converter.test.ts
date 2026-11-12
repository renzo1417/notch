import { test } from "node:test";
import assert from "node:assert/strict";
import { insideDropTarget, supportedDrop, fileStatus, type ConversionFile } from "../src/modules/converter/types.ts";
import { migrateSettings } from "../src/settings/migrations.ts";
import { backState, SHELL_STATE_DIMENSIONS } from "../src/app/state/shell-state.ts";
import { MascotStateMachine } from "../src/mascot/state-machine.ts";

test("native drag hit testing converts physical pixels and excludes the shadow margins", () => {
  const rect = { left: 16, top: 0, right: 320, bottom: 44 };
  for (const scale of [1, 1.25, 1.5, 2]) {
    assert.equal(insideDropTarget({ x: 20 * scale, y: 20 * scale }, scale, rect), true);
    assert.equal(insideDropTarget({ x: 8 * scale, y: 20 * scale }, scale, rect), false);
    assert.equal(insideDropTarget({ x: 320 * scale, y: 20 * scale }, scale, rect), false);
    assert.equal(insideDropTarget({ x: 20 * scale, y: 44 * scale }, scale, rect), false);
  }
  assert.equal(insideDropTarget({ x: 0, y: 0 }, 0, rect), false);
});

test("drop feedback accepts everyday files and rejects unsupported or oversized batches", () => {
  assert.equal(supportedDrop(["C:\\Images\\PHOTO.JPG", "C:\\report.docx", "C:\\scan.pdf"]), true);
  assert.equal(supportedDrop(["C:\\movie.mp4"]), false);
  assert.equal(supportedDrop(["C:\\program.exe"]), false);
  assert.equal(supportedDrop([]), false);
  assert.equal(supportedDrop(Array.from({ length: 21 }, (_, i) => `${i}.png`)), false);
});

test("unknown conversion progress is never presented as a percentage", () => {
  const file = { state: "running", progress: null } as ConversionFile;
  assert.equal(fileStatus(file), "Converting…");
  assert.equal(fileStatus({ ...file, progress: 0.42 }), "42%");
});

test("v4 migration adds converter defaults without changing existing choices", () => {
  const previous = { schemaVersion: 3, appearance: { accentColor: "#ff8844", opacity: .84 }, general: { selectedMonitor: "Display 2" } };
  const current = migrateSettings(previous);
  assert.equal(current.schemaVersion, 4);
  assert.deepEqual(current.converter, { enabled: true, cliPath: null });
  assert.equal(current.appearance.toolColorsEnabled, true);
  assert.equal(current.appearance.accentColor, "#ff8844");
  assert.equal(current.general.selectedMonitor, "Display 2");
  assert.equal(current.appearance.opacity, .84);
  assert.deepEqual(migrateSettings(current), current);
  const customized = migrateSettings({ ...current, converter: { enabled: false, cliPath: " C:\\Filee\\filee-cli.exe " }, appearance: { ...current.appearance, toolColorsEnabled: false } });
  assert.equal(customized.converter.enabled, false);
  assert.equal(customized.converter.cliPath, "C:\\Filee\\filee-cli.exe");
  assert.equal(customized.appearance.toolColorsEnabled, false);
});

test("converter uses drawer bounds and returns to More", () => {
  assert.deepEqual(SHELL_STATE_DIMENSIONS.converter, SHELL_STATE_DIMENSIONS.capture);
  assert.equal(backState("converter"), "utilities");
});

test("settled mascot waits for deadlines and sleeps without a frame loop", () => {
  const originalNow = Date.now;
  let now = 10_000;
  Date.now = () => now;
  try {
    const mascot = new MascotStateMachine({ enabled: true, trackCursor: true, animationIntensity: 1, pauseInactiveAnimation: true });
    now += 10_000;
    for (let i = 0; i < 100; i++) mascot.update(16.6);
    const delay = mascot.nextWakeDelayMs();
    assert.ok(delay !== null && delay > 0);
    now += 50_000;
    mascot.update(16.6);
    assert.equal(mascot.getCurrentState(), "sleepy");
    assert.equal(mascot.nextWakeDelayMs(), null);
    mascot.recordClick();
    assert.equal(mascot.nextWakeDelayMs(), 0);
    now += 2000;
    mascot.update(16.6);
    assert.equal(mascot.getCurrentState(), "idle");
  } finally { Date.now = originalNow; }
});

test("reduced motion holds a reaction until its deadline without continuous frames", () => {
  const originalNow = Date.now;
  let now = 20_000;
  Date.now = () => now;
  try {
    const mascot = new MascotStateMachine({ enabled: true, trackCursor: true, animationIntensity: 0, pauseInactiveAnimation: true });
    mascot.recordClick(); mascot.update(16.6);
    assert.equal(mascot.nextWakeDelayMs(), 900);
    assert.equal(mascot.getState().squashY, 1);
    now += 901; mascot.update(16.6);
    assert.equal(mascot.getCurrentState(), "idle");
  } finally { Date.now = originalNow; }
});
