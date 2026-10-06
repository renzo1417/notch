/**
 * Settings configuration model.
 * Defined in PRODUCT_SPEC.md section 22.
 */
export interface SettingsConfig {
  general: {
    launchAtStartup: boolean;
    alwaysOnTop: boolean;
    autoCollapseDelayMs: number;
  };
  appearance: {
    accentColor: string;
    opacity: number;
    reducedMotion: boolean;
  };
  mascot: {
    enabled: boolean;
    cursorTracking: boolean;
  };
  clipboard: {
    enabled: boolean;
    historyLimit: number;
    persistHistory: boolean;
  };
  downloads: {
    watchedFolders: string[];
  };
}
