/**
 * Settings configuration model & schema definitions.
 * Defined in PRODUCT_SPEC.md section 22.
 */

export const SETTINGS_SCHEMA_VERSION = 1;

export interface GeneralSettings {
  /** Launch application on Windows startup */
  launchAtStartup: boolean;
  /** Designated monitor name or null for primary/current */
  selectedMonitor: string | null;
  /** Always keep notch window on top of other windows */
  alwaysOnTop: boolean;
  /** Inactivity duration before auto-collapsing back to compact mode (in ms) */
  autoCollapseDelayMs: number;
}

export interface AppearanceSettings {
  /** Primary accent color hex code */
  accentColor: string;
  /** Notch surface opacity (0.80 - 1.00) */
  opacity: number;
  /** Motion scale factor (0.0 - 1.0) */
  animationIntensity: number;
  /** Explicit reduced-motion accessibility preference */
  reducedMotion: boolean;
}

export interface MascotSettings {
  /** Whether companion mascot is enabled and visible */
  enabled: boolean;
  /** Dynamic pupil tracking of cursor coordinates */
  cursorTracking: boolean;
  /** Mascot physical squash & blink animation intensity (0.0 - 1.0) */
  animationIntensity: number;
  /** Companion audio cue toggle */
  soundEnabled: boolean;
}

export interface MusicSettings {
  /** Whether right-zone media module is active */
  enabled: boolean;
  /** Show playback transport controls */
  showControls: boolean;
}

export interface TimerSettings {
  /** Default focus preset in minutes (25 or 50) */
  defaultPreset: number;
  /** Custom duration preset in minutes */
  customMinutes: number;
  /** Automatically commence countdown upon preset button tap */
  autoStartOnPresetSelect: boolean;
}

export interface ClipboardSettings {
  /** Enable native Windows clipboard change listener */
  enabled: boolean;
  /** Maximum number of history entries to keep in memory */
  historyLimit: number;
  /**
   * Persist clipboard history across app restarts.
   * Spec Invariant: Local-first privacy, OFF by default!
   */
  persistHistory: boolean;
  /** Cleanly wipe volatile clipboard buffer on application exit */
  clearOnExit: boolean;
}

export interface DownloadsSettings {
  /** Enable watched folders download awareness */
  enabled: boolean;
  /** List of watched directory paths */
  watchedFolders: string[];
  /** Open containing folder automatically upon completion */
  autoOpenCompleted: boolean;
}

export interface AppsSettings {
  /** Show quick apps dock in expanded notch */
  quickDockEnabled: boolean;
  /** Maximum apps in quick dock before overflow */
  maxQuickPinned: number;
}

export interface PerformanceSettings {
  /** Battery-saver mode (caps render frame rate, suspends dormant loops) */
  batterySaverMode: boolean;
  /** Pause canvas animations when notch is inactive or collapsed */
  pauseInactiveAnimations: boolean;
  /** Target animation frames per second (30 or 60) */
  targetFps: number;
}

export interface SettingsConfig {
  schemaVersion: number;
  general: GeneralSettings;
  appearance: AppearanceSettings;
  mascot: MascotSettings;
  music: MusicSettings;
  timer: TimerSettings;
  clipboard: ClipboardSettings;
  downloads: DownloadsSettings;
  apps: AppsSettings;
  performance: PerformanceSettings;
}

export type SettingsSection =
  | "general"
  | "appearance"
  | "mascot"
  | "music"
  | "timer"
  | "clipboard"
  | "downloads"
  | "apps"
  | "performance";

export const ACCENT_COLOR_PRESETS: Array<{ label: string; value: string }> = [
  { label: "Windows Blue", value: "#3b82f6" },
  { label: "Emerald", value: "#10b981" },
  { label: "Violet", value: "#8b5cf6" },
  { label: "Amber", value: "#f59e0b" },
  { label: "Rose", value: "#f43f5e" },
  { label: "Slate", value: "#64748b" },
];

export const DEFAULT_SETTINGS_CONFIG: SettingsConfig = {
  schemaVersion: SETTINGS_SCHEMA_VERSION,
  general: {
    launchAtStartup: false,
    selectedMonitor: null,
    alwaysOnTop: true,
    autoCollapseDelayMs: 4000,
  },
  appearance: {
    accentColor: "#3b82f6",
    opacity: 0.96,
    animationIntensity: 1.0,
    reducedMotion: false,
  },
  mascot: {
    enabled: true,
    cursorTracking: true,
    animationIntensity: 1.0,
    soundEnabled: false,
  },
  music: {
    enabled: true,
    showControls: true,
  },
  timer: {
    defaultPreset: 25,
    customMinutes: 15,
    autoStartOnPresetSelect: false,
  },
  clipboard: {
    enabled: true,
    historyLimit: 25,
    persistHistory: false, // Privacy invariant: OFF by default
    clearOnExit: true,
  },
  downloads: {
    enabled: true,
    watchedFolders: [],
    autoOpenCompleted: false,
  },
  apps: {
    quickDockEnabled: true,
    maxQuickPinned: 6,
  },
  performance: {
    batterySaverMode: false,
    pauseInactiveAnimations: true,
    targetFps: 60,
  },
};
