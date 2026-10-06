import { FocusTimerEngine, formatSeconds, formatCompactLabel } from "./timer-engine.ts";
import { notificationStore } from "../notifications/notification-store.svelte";
import { mascotStore } from "../../mascot/mascot-store.svelte";
import { DEFAULT_FOCUS_TIMER_STATE, type FocusTimerState } from "./types.ts";

export { formatSeconds, formatCompactLabel };

class FocusTimerStore {
  private engine: FocusTimerEngine;
  state = $state<FocusTimerState>({ ...DEFAULT_FOCUS_TIMER_STATE });

  constructor() {
    this.engine = new FocusTimerEngine({
      callbacks: {
        onMascotFocused: () => mascotStore.setMood("focused"),
        onMascotCelebrate: (durationMs) => mascotStore.celebrate(durationMs),
        onMascotReset: () => mascotStore.resetMood(),
        onNotification: (title, message, durationMs) => {
          notificationStore.notify({ title, message, durationMs });
        },
      },
    });

    this.engine.subscribe((newState) => {
      this.state = { ...newState };
    });
  }

  get formattedTime(): string {
    return this.engine.formattedTime;
  }

  get formattedCompact(): string {
    return this.engine.formattedCompact;
  }

  get activePreset(): "25" | "50" | "custom" {
    return this.engine.activePreset;
  }

  setPreset(preset: 25 | 50): void {
    this.engine.setPreset(preset);
  }

  setCustomDuration(minutes: number): void {
    this.engine.setCustomDuration(minutes);
  }

  openTimerSurface(): void {
    this.engine.openTimerSurface();
  }

  closeTimerSurface(): void {
    this.engine.closeTimerSurface();
  }

  start(minutes?: number): void {
    this.engine.start(minutes);
  }

  pause(): void {
    this.engine.pause();
  }

  resume(): void {
    this.engine.resume();
  }

  stop(): void {
    this.engine.stop();
  }

  reset(): void {
    this.engine.reset();
  }

  destroy(): void {
    this.engine.destroy();
  }
}

export const timerStore = new FocusTimerStore();
