import { test } from "node:test";
import assert from "node:assert/strict";

import {
  DEFAULT_SETTINGS_CONFIG,
  SETTINGS_SCHEMA_VERSION,
  ACCENT_COLOR_PRESETS,
  type SettingsConfig,
} from "../src/settings/types.ts";
import {
  migrateSettings,
  isValidHexColor,
  clamp,
} from "../src/settings/migrations.ts";

test("Settings: default configuration contains all 9 sections with schemaVersion 1", () => {
  assert.equal(DEFAULT_SETTINGS_CONFIG.schemaVersion, SETTINGS_SCHEMA_VERSION);
  assert.equal(DEFAULT_SETTINGS_CONFIG.schemaVersion, 1);

  // 1. General
  assert.equal(typeof DEFAULT_SETTINGS_CONFIG.general.launchAtStartup, "boolean");
  assert.equal(DEFAULT_SETTINGS_CONFIG.general.selectedMonitor, null);
  assert.equal(DEFAULT_SETTINGS_CONFIG.general.alwaysOnTop, true);
  assert.equal(DEFAULT_SETTINGS_CONFIG.general.autoCollapseDelayMs, 4000);

  // 2. Appearance
  assert.equal(DEFAULT_SETTINGS_CONFIG.appearance.accentColor, "#3b82f6");
  assert.equal(DEFAULT_SETTINGS_CONFIG.appearance.opacity, 0.96);
  assert.equal(DEFAULT_SETTINGS_CONFIG.appearance.animationIntensity, 1.0);
  assert.equal(DEFAULT_SETTINGS_CONFIG.appearance.reducedMotion, false);

  // 3. Mascot
  assert.equal(DEFAULT_SETTINGS_CONFIG.mascot.enabled, true);
  assert.equal(DEFAULT_SETTINGS_CONFIG.mascot.cursorTracking, true);
  assert.equal(DEFAULT_SETTINGS_CONFIG.mascot.animationIntensity, 1.0);
  assert.equal(DEFAULT_SETTINGS_CONFIG.mascot.soundEnabled, false);

  // 4. Music
  assert.equal(DEFAULT_SETTINGS_CONFIG.music.enabled, true);
  assert.equal(DEFAULT_SETTINGS_CONFIG.music.showControls, true);

  // 5. Timer
  assert.equal(DEFAULT_SETTINGS_CONFIG.timer.defaultPreset, 25);
  assert.equal(DEFAULT_SETTINGS_CONFIG.timer.customMinutes, 15);
  assert.equal(DEFAULT_SETTINGS_CONFIG.timer.autoStartOnPresetSelect, false);

  // 6. Clipboard (Privacy invariant: persistHistory is OFF by default)
  assert.equal(DEFAULT_SETTINGS_CONFIG.clipboard.enabled, true);
  assert.equal(DEFAULT_SETTINGS_CONFIG.clipboard.historyLimit, 25);
  assert.equal(DEFAULT_SETTINGS_CONFIG.clipboard.persistHistory, false);
  assert.equal(DEFAULT_SETTINGS_CONFIG.clipboard.clearOnExit, true);

  // 7. Downloads
  assert.equal(DEFAULT_SETTINGS_CONFIG.downloads.enabled, true);
  assert.deepEqual(DEFAULT_SETTINGS_CONFIG.downloads.watchedFolders, []);
  assert.equal(DEFAULT_SETTINGS_CONFIG.downloads.autoOpenCompleted, false);

  // 8. Apps
  assert.equal(DEFAULT_SETTINGS_CONFIG.apps.quickDockEnabled, true);
  assert.equal(DEFAULT_SETTINGS_CONFIG.apps.maxQuickPinned, 6);

  // 9. Performance
  assert.equal(DEFAULT_SETTINGS_CONFIG.performance.batterySaverMode, false);
  assert.equal(DEFAULT_SETTINGS_CONFIG.performance.pauseInactiveAnimations, true);
  assert.equal(DEFAULT_SETTINGS_CONFIG.performance.targetFps, 60);
});

test("Settings Recovery: null, undefined, or empty payload safely recovers to defaults", () => {
  const fromNull = migrateSettings(null);
  assert.equal(fromNull.schemaVersion, SETTINGS_SCHEMA_VERSION);
  assert.equal(fromNull.general.autoCollapseDelayMs, 4000);
  assert.equal(fromNull.appearance.accentColor, "#3b82f6");

  const fromUndefined = migrateSettings(undefined);
  assert.equal(fromUndefined.schemaVersion, SETTINGS_SCHEMA_VERSION);

  const fromEmptyObj = migrateSettings({});
  assert.equal(fromEmptyObj.schemaVersion, SETTINGS_SCHEMA_VERSION);
  assert.equal(fromEmptyObj.clipboard.persistHistory, false);
});

test("Settings Validation: clamps invalid numbers and rejects malformed hex colors", () => {
  const corrupted = {
    schemaVersion: 0,
    appearance: {
      accentColor: "not-a-color",
      opacity: 2.5, // out of range
      animationIntensity: -0.5, // out of range
      reducedMotion: true,
    },
    general: {
      autoCollapseDelayMs: 999999, // exceeds max clamp
    },
    clipboard: {
      historyLimit: -10, // below min clamp
      persistHistory: true,
    },
    apps: {
      maxQuickPinned: 20, // exceeds dock max clamp
    },
  };

  const recovered = migrateSettings(corrupted);
  assert.equal(recovered.schemaVersion, 1);
  // Color should fall back to default
  assert.equal(recovered.appearance.accentColor, "#3b82f6");
  // Opacity should clamp to 1.0
  assert.equal(recovered.appearance.opacity, 1.0);
  // Animation intensity should clamp to 0.0
  assert.equal(recovered.appearance.animationIntensity, 0.0);
  assert.equal(recovered.appearance.reducedMotion, true);
  // Auto collapse delay clamps to 60000ms
  assert.equal(recovered.general.autoCollapseDelayMs, 60000);
  // History limit clamps to 5
  assert.equal(recovered.clipboard.historyLimit, 5);
  assert.equal(recovered.clipboard.persistHistory, true);
  // Max quick pinned clamps to 6
  assert.equal(recovered.apps.maxQuickPinned, 6);
});

test("Settings Helpers: isValidHexColor and clamp handle boundary conditions", () => {
  assert.equal(isValidHexColor("#3b82f6"), true);
  assert.equal(isValidHexColor("#fff"), true);
  assert.equal(isValidHexColor("#AABBCC"), true);
  assert.equal(isValidHexColor("3b82f6"), false);
  assert.equal(isValidHexColor("rgb(0,0,0)"), false);
  assert.equal(isValidHexColor(null), false);
  assert.equal(isValidHexColor(12345), false);

  assert.equal(clamp(50, 0, 100, 10), 50);
  assert.equal(clamp(-10, 0, 100, 10), 0);
  assert.equal(clamp(150, 0, 100, 10), 100);
  assert.equal(clamp("invalid", 0, 100, 25), 25);
  assert.equal(clamp(NaN, 0, 100, 25), 25);
});

test("Settings Migration: preserves legacy v0 partial config and fills missing v1 sections", () => {
  const legacyV0Config = {
    general: {
      launchAtStartup: true,
      alwaysOnTop: false,
      autoCollapseDelayMs: 6000,
    },
    appearance: {
      accentColor: "#10b981",
      opacity: 0.9,
    },
    mascot: {
      enabled: false,
    },
    downloads: {
      watchedFolders: ["C:\\Users\\Test\\Downloads", "D:\\Games\\Downloads"],
    },
  };

  const migrated = migrateSettings(legacyV0Config);
  assert.equal(migrated.schemaVersion, 1);
  assert.equal(migrated.general.launchAtStartup, true);
  assert.equal(migrated.general.alwaysOnTop, false);
  assert.equal(migrated.general.autoCollapseDelayMs, 6000);
  assert.equal(migrated.appearance.accentColor, "#10b981");
  assert.equal(migrated.appearance.opacity, 0.9);
  assert.equal(migrated.mascot.enabled, false);
  assert.equal(migrated.downloads.watchedFolders.length, 2);
  assert.equal(migrated.downloads.watchedFolders[0], "C:\\Users\\Test\\Downloads");

  // Missing sections in v0 are safely populated with defaults
  assert.equal(migrated.music.enabled, true);
  assert.equal(migrated.timer.defaultPreset, 25);
  assert.equal(migrated.clipboard.persistHistory, false);
  assert.equal(migrated.performance.targetFps, 60);
});

test("Settings Presets: accent color presets have valid labels and hex formats", () => {
  assert.ok(ACCENT_COLOR_PRESETS.length >= 6);
  for (const preset of ACCENT_COLOR_PRESETS) {
    assert.ok(preset.label.length > 0);
    assert.ok(isValidHexColor(preset.value));
  }
});
