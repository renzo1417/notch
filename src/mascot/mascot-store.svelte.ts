import type { MascotState } from "./types";

export type MascotMoodListener = (state: MascotState, durationMs?: number) => void;

class MascotStore {
  currentMood = $state<MascotState | null>(null);
  private listeners = new Set<MascotMoodListener>();

  /**
   * Subscribes to mascot emotional triggers. Immediately fires with current active mood if present.
   */
  subscribe(listener: MascotMoodListener): () => void {
    this.listeners.add(listener);
    if (this.currentMood) {
      listener(this.currentMood);
    }
    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * Sets external mascot emotional state (e.g. 'focused' while focus timer is running).
   */
  setMood(state: MascotState, durationMs?: number): void {
    this.currentMood = state;
    for (const listener of this.listeners) {
      listener(state, durationMs);
    }
  }

  /**
   * Resets external mascot state back to default idle/autonomous behavior.
   */
  resetMood(): void {
    this.currentMood = null;
    for (const listener of this.listeners) {
      listener("idle");
    }
  }

  /**
   * Briefly triggers a celebratory hop and cheerful expression (e.g. timer complete).
   */
  celebrate(durationMs = 3500): void {
    this.currentMood = "celebrate";
    for (const listener of this.listeners) {
      listener("celebrate", durationMs);
    }
    setTimeout(() => {
      if (this.currentMood === "celebrate") {
        this.currentMood = null;
      }
    }, durationMs);
  }
}

export const mascotStore = new MascotStore();
