import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { contextEngine, ContextPriority } from "../../app/state/context-engine.ts";
import { notificationStore } from "../notifications/notification-store.svelte.ts";
import { mascotStore } from "../../mascot/mascot-store.svelte.ts";
import type { BatteryState } from "./types.ts";

class BatteryStore {
  state = $state<BatteryState>({
    hasBattery: false,
    percentage: 100,
    isCharging: false,
    isAcConnected: true,
    remainingSeconds: undefined,
  });

  isLoading = $state<boolean>(false);
  private hasNotifiedLowBattery = false;
  private unlistenChanged: UnlistenFn | null = null;

  get hasBattery(): boolean {
    return this.state.hasBattery;
  }

  get percentage(): number {
    return this.state.percentage;
  }

  get isCharging(): boolean {
    return this.state.isCharging;
  }

  get isAcConnected(): boolean {
    return this.state.isAcConnected;
  }

  get isLowBattery(): boolean {
    return this.state.hasBattery && !this.state.isAcConnected && !this.state.isCharging && this.state.percentage <= 20;
  }

  /**
   * Returns human-readable remaining time string ONLY when Windows provides a valid value.
   * Spec Invariant: "Never fabricate remaining time."
   */
  get formattedRemainingTime(): string | null {
    if (!this.state.hasBattery || this.state.isAcConnected || !this.state.remainingSeconds) {
      return null;
    }
    const secs = this.state.remainingSeconds;
    if (secs <= 0 || secs >= 86400) {
      return null;
    }
    const hours = Math.floor(secs / 3600);
    const mins = Math.floor((secs % 3600) / 60);
    if (hours > 0) {
      return `${hours}h ${mins}m`;
    }
    return `${mins}m`;
  }

  get formattedPercent(): string {
    return `${this.state.percentage}%`;
  }

  get statusSummary(): string {
    if (!this.state.hasBattery) {
      return "Desktop · AC Power";
    }
    if (this.state.isCharging) {
      return `${this.formattedPercent} · Charging`;
    }
    if (this.state.isAcConnected) {
      return `${this.formattedPercent} · AC Power`;
    }
    const remaining = this.formattedRemainingTime;
    if (remaining) {
      return `${this.formattedPercent} · ${remaining} remaining`;
    }
    return `${this.formattedPercent} · On Battery`;
  }

  async init(): Promise<void> {
    this.isLoading = true;
    try {
      const current = await invoke<BatteryState>("get_battery_state");
      this.state = current;
      this.evaluateLowBattery(current);
    } catch (err) {
      console.error("Failed to query battery state:", err);
    } finally {
      this.isLoading = false;
    }

    if (!this.unlistenChanged) {
      try {
        this.unlistenChanged = await listen<BatteryState>("battery.changed", (event) => {
          this.state = event.payload;
          this.evaluateLowBattery(event.payload);
        });
      } catch (err) {
        console.error("Failed to listen for battery events:", err);
      }
    }
  }

  /**
   * Conservative low-battery alert evaluator.
   * Pushes a local internal notification and subtle mascot warning on crossing <= 20%.
   */
  private evaluateLowBattery(status: BatteryState): void {
    const isLow = status.hasBattery && !status.isAcConnected && !status.isCharging && status.percentage <= 20;

    if (isLow && !this.hasNotifiedLowBattery) {
      this.hasNotifiedLowBattery = true;

      // Conservative internal notification
      notificationStore.notify({
        title: "Low Battery",
        message: `${status.percentage}% remaining. Connect power adapter.`,
        durationMs: 5000,
      });

      // Mascot subtle warning state
      mascotStore.setMood("warning", 4500);
    } else if (!isLow && (status.isAcConnected || status.isCharging || status.percentage > 25)) {
      // Reset notification latch when charging or battery recovers
      this.hasNotifiedLowBattery = false;
    }
  }

  /**
   * Manually opens battery context view in the center zone.
   */
  openBatterySurface(): void {
    contextEngine.registerCandidate({
      id: "battery",
      mode: "battery",
      priority: ContextPriority.P5_USER_SURFACE,
      timestamp: Date.now(),
    });
  }

  /**
   * Closes the battery surface.
   */
  closeBatterySurface(): void {
    contextEngine.unregisterCandidate("battery");
  }

  destroy(): void {
    if (this.unlistenChanged) {
      this.unlistenChanged();
      this.unlistenChanged = null;
    }
  }
}

export const batteryStore = new BatteryStore();
