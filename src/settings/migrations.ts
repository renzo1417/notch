import {
  DEFAULT_SETTINGS_CONFIG,
  SETTINGS_SCHEMA_VERSION,
  type SettingsConfig,
} from "./types.ts";

/**
 * Validates a color hex string (#RGB or #RRGGBB).
 */
export function isValidHexColor(hex: unknown): boolean {
  if (typeof hex !== "string") return false;
  return /^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6})$/.test(hex.trim());
}

/**
 * Clamps a numeric value safely between min and max.
 */
export function clamp(value: unknown, min: number, max: number, fallback: number): number {
  if (typeof value !== "number" || isNaN(value)) return fallback;
  return Math.min(Math.max(value, min), max);
}

/**
 * Migration strategy & safe recovery pipeline for application settings.
 * Ensures backward compatibility across schema versions and gracefully repairs corrupted data.
 */
export function migrateSettings(raw: unknown): SettingsConfig {
  if (!raw || typeof raw !== "object") {
    console.warn("[Settings Migration] Received invalid or null settings payload. Falling back to defaults.");
    return structuredClone(DEFAULT_SETTINGS_CONFIG);
  }

  const result = structuredClone(DEFAULT_SETTINGS_CONFIG);
  const data = raw as Record<string, unknown>;

  // Check version (0 = unversioned legacy or missing)
  const version = typeof data.schemaVersion === "number" ? data.schemaVersion : 0;

  // 1. General section recovery
  if (data.general && typeof data.general === "object") {
    const gen = data.general as Record<string, unknown>;
    result.general.launchAtStartup = Boolean(gen.launchAtStartup);
    result.general.selectedMonitor = typeof gen.selectedMonitor === "string" ? gen.selectedMonitor : null;
    result.general.alwaysOnTop = gen.alwaysOnTop !== undefined ? Boolean(gen.alwaysOnTop) : true;
    result.general.autoCollapseDelayMs = clamp(gen.autoCollapseDelayMs, 1000, 60000, 4000);
  }

  // 2. Appearance section recovery
  if (data.appearance && typeof data.appearance === "object") {
    const app = data.appearance as Record<string, unknown>;
    result.appearance.accentColor = isValidHexColor(app.accentColor)
      ? (app.accentColor as string).trim()
      : result.appearance.accentColor;
    result.appearance.opacity = clamp(app.opacity, 0.7, 1.0, 0.96);
    result.appearance.animationIntensity = clamp(app.animationIntensity, 0.0, 1.0, 1.0);
    result.appearance.reducedMotion = Boolean(app.reducedMotion);
  }

  // 3. Mascot section recovery
  if (data.mascot && typeof data.mascot === "object") {
    const mas = data.mascot as Record<string, unknown>;
    result.mascot.enabled = mas.enabled !== undefined ? Boolean(mas.enabled) : true;
    result.mascot.cursorTracking = mas.cursorTracking !== undefined ? Boolean(mas.cursorTracking) : true;
    result.mascot.animationIntensity = clamp(mas.animationIntensity, 0.0, 1.0, 1.0);
    result.mascot.soundEnabled = Boolean(mas.soundEnabled);
  }

  // 4. Music section recovery
  if (data.music && typeof data.music === "object") {
    const mus = data.music as Record<string, unknown>;
    result.music.enabled = mus.enabled !== undefined ? Boolean(mus.enabled) : true;
    result.music.showControls = mus.showControls !== undefined ? Boolean(mus.showControls) : true;
  }

  // 5. Timer section recovery
  if (data.timer && typeof data.timer === "object") {
    const tim = data.timer as Record<string, unknown>;
    result.timer.defaultPreset = tim.defaultPreset === 50 ? 50 : 25;
    result.timer.customMinutes = clamp(tim.customMinutes, 1, 180, 15);
    result.timer.autoStartOnPresetSelect = Boolean(tim.autoStartOnPresetSelect);
  }

  // 6. Clipboard section recovery
  if (data.clipboard && typeof data.clipboard === "object") {
    const clp = data.clipboard as Record<string, unknown>;
    result.clipboard.enabled = clp.enabled !== undefined ? Boolean(clp.enabled) : true;
    result.clipboard.historyLimit = clamp(clp.historyLimit, 5, 200, 25);
    // Strict privacy guarantee: persistHistory defaults to false
    result.clipboard.persistHistory = Boolean(clp.persistHistory);
    result.clipboard.clearOnExit = clp.clearOnExit !== undefined ? Boolean(clp.clearOnExit) : true;
  }

  // 7. Downloads section recovery
  if (data.downloads && typeof data.downloads === "object") {
    const dls = data.downloads as Record<string, unknown>;
    result.downloads.enabled = dls.enabled !== undefined ? Boolean(dls.enabled) : true;
    if (Array.isArray(dls.watchedFolders)) {
      result.downloads.watchedFolders = dls.watchedFolders.filter(
        (p): p is string => typeof p === "string" && p.trim().length > 0
      );
    }
    result.downloads.autoOpenCompleted = Boolean(dls.autoOpenCompleted);
  }

  // 8. Apps section recovery
  if (data.apps && typeof data.apps === "object") {
    const app = data.apps as Record<string, unknown>;
    result.apps.quickDockEnabled = app.quickDockEnabled !== undefined ? Boolean(app.quickDockEnabled) : true;
    result.apps.maxQuickPinned = clamp(app.maxQuickPinned, 4, 6, 6);
  }

  // 9. Performance section recovery
  if (data.performance && typeof data.performance === "object") {
    const perf = data.performance as Record<string, unknown>;
    result.performance.batterySaverMode = Boolean(perf.batterySaverMode);
    result.performance.pauseInactiveAnimations =
      perf.pauseInactiveAnimations !== undefined ? Boolean(perf.pauseInactiveAnimations) : true;
    result.performance.targetFps = perf.targetFps === 30 ? 30 : 60;
  }

  if (version < 2 && result.appearance.accentColor.toLowerCase() === "#3b82f6") {
    result.appearance.accentColor = DEFAULT_SETTINGS_CONFIG.appearance.accentColor;
  }

  // Advance schema version to current
  result.schemaVersion = SETTINGS_SCHEMA_VERSION;
  return result;
}
