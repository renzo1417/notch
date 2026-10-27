import { GRAPHITE, normalizeHex, accentForeground } from "../styles/palette.ts";
import { SHELL_MOTION } from "../shell/shell-motion.ts";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  DEFAULT_SETTINGS_CONFIG,
  type SettingsConfig,
  type SettingsSection,
} from "./types.ts";
import { migrateSettings } from "./migrations.ts";
import { notificationStore } from "../modules/notifications/notification-store.svelte.ts";

export interface MonitorInfo {
  name: string | null;
  width: number;
  height: number;
  scale_factor: number;
  is_primary: boolean;
}

class SettingsStore {
  state = $state<SettingsConfig>(structuredClone(DEFAULT_SETTINGS_CONFIG));
  activeSection = $state<SettingsSection>("general");
  availableMonitors = $state<MonitorInfo[]>([]);
  isLoading = $state<boolean>(false);
  isSaving = $state<boolean>(false);
  saveError = $state("");

  private unlistenChanged: UnlistenFn | null = null;
  private saveTimeout: ReturnType<typeof setTimeout> | null = null;

  async init(): Promise<void> {
    this.isLoading = true;
    try {
      // 1. Fetch persistent settings from native backend
      try {
        const backendConfig = await invoke<SettingsConfig>("get_settings");
        this.state = migrateSettings(backendConfig);
      } catch (err) {
        // Fallback for non-tauri or testing environments
        const stored = typeof localStorage !== "undefined" ? localStorage.getItem("project_notch_settings") : null;
        if (stored) {
          try {
            this.state = migrateSettings(JSON.parse(stored));
          } catch {
            this.state = structuredClone(DEFAULT_SETTINGS_CONFIG);
          }
        }
      }

      // 2. Query connected monitors
      try {
        const monitors = await invoke<MonitorInfo[]>("list_monitors");
        this.availableMonitors = monitors;
      } catch {
        this.availableMonitors = [
          { name: "Primary Display", width: 1920, height: 1080, scale_factor: 1.0, is_primary: true },
        ];
      }

      // 3. Apply settings immediately to DOM tokens and runtime environment
      this.applyLiveTokens(this.state);

      // 4. Listen to settings-changed events from backend or other windows
      if (!this.unlistenChanged && typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
        try {
          this.unlistenChanged = await listen<SettingsConfig>("settings-changed", (event) => {
            if (event.payload) {
              this.state = migrateSettings(event.payload);
              this.applyLiveTokens(this.state);
            }
          });
        } catch {
          // Ignore if event listening is unavailable
        }
      }
    } finally {
      this.isLoading = false;
    }
  }

  /**
   * Applies CSS design tokens and performance configurations live without restart.
   */
  applyLiveTokens(cfg: SettingsConfig): void {
    if (typeof document === "undefined") return;

    const root = document.documentElement;
    const { appearance, performance } = cfg;

    const palette = {
      "--surface-raised": GRAPHITE.raised,
      "--text-primary": GRAPHITE.text,
      "--text-secondary": GRAPHITE.secondary,
      "--status-warning": GRAPHITE.warning,
      "--status-error": GRAPHITE.error,
    };
    for (const [token, value] of Object.entries(palette)) root.style.setProperty(token, value);

    // Accent color live update
    root.style.setProperty("--accent-primary", normalizeHex(appearance.accentColor));
    root.style.setProperty("--accent-muted", `${normalizeHex(appearance.accentColor)}26`); // 15% opacity hex
    root.style.setProperty("--accent-focus", normalizeHex(appearance.accentColor)); // 40% opacity hex

    root.style.setProperty("--on-accent", accentForeground(appearance.accentColor));

    // Opacity
    root.style.setProperty("--surface-root", `rgba(${GRAPHITE.surfaceRgb}, ${appearance.opacity})`);

    // Animation intensity & Reduced motion
    const isReduced = appearance.reducedMotion || performance.batterySaverMode || window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const factor = isReduced ? 0 : appearance.animationIntensity;

    root.style.setProperty("--motion-micro", `${Math.round(SHELL_MOTION.microMs * factor)}ms`);
    root.style.setProperty("--motion-shell", `${Math.round(SHELL_MOTION.shellMs * factor)}ms`);
    root.style.setProperty("--motion-context", `${Math.round(SHELL_MOTION.contextMs * factor)}ms`);
    root.style.setProperty("--motion-mascot", `${Math.round(SHELL_MOTION.mascotMs * factor)}ms`);
  }

  /**
   * Updates a section of configuration and persists to native backend.
   */
  async update<K extends keyof Omit<SettingsConfig, "schemaVersion">>(
    section: K,
    patch: Partial<SettingsConfig[K]>
  ): Promise<void> {
    const updatedSection = { ...this.state[section], ...patch };
    this.state[section] = updatedSection as any;

    this.applyLiveTokens(this.state);

    // Save to localStorage as immediate offline cache
    if (typeof localStorage !== "undefined") {
      try {
        localStorage.setItem("project_notch_settings", JSON.stringify(this.state));
      } catch {
        // Ignore quota errors
      }
    }

    this.saveError = "";

    // Debounce save to backend
    if (this.saveTimeout) {
      clearTimeout(this.saveTimeout);
    }

    this.saveTimeout = setTimeout(async () => {
      this.isSaving = true;
      try {
        const saved = await invoke<SettingsConfig>("save_settings", {
          settings: this.state,
        });
        this.state = migrateSettings(saved);
      } catch (err) {
        if ("__TAURI_INTERNALS__" in window) this.saveError = "Couldn’t save settings. Try again.";
        console.error("Failed to save settings to backend:", err);
      } finally {
        this.isSaving = false;
      }
    }, 250);
  }

  /**
   * Resets all settings to their default values.
   */
  async resetToDefaults(): Promise<void> {
    try {
      const reset = await invoke<SettingsConfig>("reset_settings");
      this.state = migrateSettings(reset);
    } catch {
      this.state = structuredClone(DEFAULT_SETTINGS_CONFIG);
      if (typeof localStorage !== "undefined") {
        localStorage.removeItem("project_notch_settings");
      }
    }

    this.applyLiveTokens(this.state);
    notificationStore.notify({
      title: "Settings Reset",
      message: "Restored default configuration",
      durationMs: 2500,
    });
  }

  setSection(section: SettingsSection): void {
    this.activeSection = section;
  }

  destroy(): void {
    if (this.unlistenChanged) {
      this.unlistenChanged();
      this.unlistenChanged = null;
    }
    if (this.saveTimeout) {
      clearTimeout(this.saveTimeout);
      this.saveTimeout = null;
    }
  }
}

export const settingsStore = new SettingsStore();
