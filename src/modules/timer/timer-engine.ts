/**
 * Core Focus Timer Engine.
 * Pure TypeScript implementation providing drift-free timestamp countdowns,
 * context priority engine synchronization, and companion emotional reactions.
 * Defined in PRODUCT_SPEC.md section 15.
 */

import {
  contextEngine as defaultContextEngine,
  ContextPriority,
  type ContextEngine,
} from "../../app/state/context-engine.ts";
import { DEFAULT_FOCUS_TIMER_STATE, type FocusTimerState } from "./types.ts";

export function formatSeconds(totalSeconds: number): string {
  const safeSeconds = Math.max(0, totalSeconds);
  const minutes = Math.floor(safeSeconds / 60);
  const seconds = safeSeconds % 60;
  const mm = minutes.toString().padStart(2, "0");
  const ss = seconds.toString().padStart(2, "0");
  return `${mm}:${ss}`;
}

export function formatCompactLabel(state: FocusTimerState): string {
  if (state.isRunning && !state.isPaused) {
    return `Focus ${formatSeconds(state.remainingSeconds)}`;
  }
  if (state.isPaused) {
    return `Focus · Paused ${formatSeconds(state.remainingSeconds)}`;
  }
  return "Focus Timer";
}

export interface FocusTimerEngineCallbacks {
  onMascotFocused?: () => void;
  onMascotCelebrate?: (durationMs: number) => void;
  onMascotReset?: () => void;
  onNotification?: (title: string, message: string, durationMs: number) => void;
}

export class FocusTimerEngine {
  private state: FocusTimerState;
  private tickInterval: ReturnType<typeof setInterval> | null = null;
  private listeners = new Set<(state: Readonly<FocusTimerState>) => void>();
  private contextEngine: ContextEngine;
  private callbacks: FocusTimerEngineCallbacks;

  constructor(options?: {
    contextEngine?: ContextEngine;
    callbacks?: FocusTimerEngineCallbacks;
    initialState?: Partial<FocusTimerState>;
  }) {
    this.contextEngine = options?.contextEngine ?? defaultContextEngine;
    this.callbacks = options?.callbacks ?? {};
    this.state = {
      ...DEFAULT_FOCUS_TIMER_STATE,
      ...options?.initialState,
    };
  }

  public getState(): Readonly<FocusTimerState> {
    return this.state;
  }

  public get formattedTime(): string {
    return formatSeconds(this.state.remainingSeconds);
  }

  public get formattedCompact(): string {
    return formatCompactLabel(this.state);
  }

  public get activePreset(): "25" | "50" | "custom" {
    if (this.state.presetMinutes === 25) return "25";
    if (this.state.presetMinutes === 50) return "50";
    return "custom";
  }

  public subscribe(listener: (state: Readonly<FocusTimerState>) => void): () => void {
    this.listeners.add(listener);
    listener(this.state);
    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * Sets the countdown preset to 25 or 50 minutes.
   */
  public setPreset(preset: 25 | 50): void {
    if (this.state.isRunning) return;
    this.state.presetMinutes = preset;
    this.state.totalSeconds = preset * 60;
    this.state.remainingSeconds = preset * 60;
    this.notify();
  }

  /**
   * Sets custom countdown minutes (clamped 1 to 180).
   */
  public setCustomDuration(minutes: number): void {
    const clamped = Math.max(1, Math.min(180, Math.round(minutes)));
    this.state.customMinutes = clamped;
    if (!this.state.isRunning) {
      this.state.presetMinutes = clamped;
      this.state.totalSeconds = clamped * 60;
      this.state.remainingSeconds = clamped * 60;
    }
    this.notify();
  }

  /**
   * Manually opens the timer surface at P5_USER_SURFACE before starting.
   */
  public openTimerSurface(now: number = Date.now()): void {
    if (this.state.isRunning) return;
    this.contextEngine.registerCandidate({
      id: "timer",
      mode: "timer",
      priority: ContextPriority.P5_USER_SURFACE,
      timestamp: now,
      data: {
        presetMinutes: this.state.presetMinutes,
        remainingSeconds: this.state.remainingSeconds,
      },
    });
  }

  /**
   * Closes the timer surface if opened without starting.
   */
  public closeTimerSurface(): void {
    if (this.state.isRunning) return;
    this.contextEngine.unregisterCandidate("timer");
  }

  /**
   * Starts or restarts the focus countdown.
   * Registers candidate at P2_FOCUS_TIMER, outranking active coding sessions (P4).
   */
  public start(minutes?: number, now: number = Date.now()): void {
    if (minutes !== undefined) {
      this.state.presetMinutes = minutes;
    }
    const total = this.state.presetMinutes * 60;

    this.state.totalSeconds = total;
    this.state.remainingSeconds = total;
    this.state.startTimestampMs = now;
    this.state.endTimestampMs = now + total * 1000;
    this.state.isRunning = true;
    this.state.isPaused = false;

    // Registers at P2_FOCUS_TIMER
    this.contextEngine.registerCandidate({
      id: "timer",
      mode: "timer",
      priority: ContextPriority.P2_FOCUS_TIMER,
      timestamp: now,
      data: {
        presetMinutes: this.state.presetMinutes,
        remainingSeconds: this.state.remainingSeconds,
      },
    });

    // Companion reacts by becoming focused
    this.callbacks.onMascotFocused?.();

    this.startTickLoop();
    this.notify();
  }

  /**
   * Pauses the timer with exact timestamp remaining computation.
   */
  public pause(now: number = Date.now()): void {
    if (!this.state.isRunning || this.state.isPaused) return;

    this.stopTickLoop();

    if (this.state.endTimestampMs !== null) {
      const remainingMs = Math.max(0, this.state.endTimestampMs - now);
      this.state.remainingSeconds = Math.ceil(remainingMs / 1000);
    }

    this.state.isPaused = true;
    this.state.endTimestampMs = null;

    // Keep candidate in context engine so surface remains active
    this.contextEngine.registerCandidate({
      id: "timer",
      mode: "timer",
      priority: ContextPriority.P2_FOCUS_TIMER,
      timestamp: now,
      data: {
        presetMinutes: this.state.presetMinutes,
        remainingSeconds: this.state.remainingSeconds,
        isPaused: true,
      },
    });

    this.notify();
  }

  /**
   * Resumes the paused timer.
   */
  public resume(now: number = Date.now()): void {
    if (!this.state.isRunning || !this.state.isPaused) return;

    this.state.endTimestampMs = now + this.state.remainingSeconds * 1000;
    this.state.isPaused = false;

    this.contextEngine.registerCandidate({
      id: "timer",
      mode: "timer",
      priority: ContextPriority.P2_FOCUS_TIMER,
      timestamp: now,
      data: {
        presetMinutes: this.state.presetMinutes,
        remainingSeconds: this.state.remainingSeconds,
        isPaused: false,
      },
    });

    this.callbacks.onMascotFocused?.();
    this.startTickLoop();
    this.notify();
  }

  /**
   * Stops and resets the timer back to its initial preset duration.
   */
  public stop(): void {
    this.stopTickLoop();

    this.state.isRunning = false;
    this.state.isPaused = false;
    this.state.remainingSeconds = this.state.presetMinutes * 60;
    this.state.totalSeconds = this.state.presetMinutes * 60;
    this.state.startTimestampMs = null;
    this.state.endTimestampMs = null;

    this.contextEngine.unregisterCandidate("timer");
    this.callbacks.onMascotReset?.();
    this.notify();
  }

  public reset(): void {
    this.stop();
  }

  /**
   * Timestamp-based tick calculation. Injected `now` facilitates unit testing.
   */
  public tick(now: number = Date.now()): void {
    if (!this.state.isRunning || this.state.isPaused || this.state.endTimestampMs === null) {
      return;
    }

    const diffMs = this.state.endTimestampMs - now;

    if (diffMs <= 0) {
      this.complete();
    } else {
      const nextRemaining = Math.max(0, Math.ceil(diffMs / 1000));
      if (nextRemaining !== this.state.remainingSeconds) {
        this.state.remainingSeconds = nextRemaining;
        this.notify();
      }
    }
  }

  /**
   * Natural countdown completion.
   */
  private complete(): void {
    this.stopTickLoop();

    const completedPresetMinutes = this.state.presetMinutes;

    this.state.isRunning = false;
    this.state.isPaused = false;
    this.state.remainingSeconds = this.state.presetMinutes * 60;
    this.state.totalSeconds = this.state.presetMinutes * 60;
    this.state.startTimestampMs = null;
    this.state.endTimestampMs = null;

    // Unregister active timer context
    this.contextEngine.unregisterCandidate("timer");

    // Companion briefly celebrates on completion (~3.5s)
    this.callbacks.onMascotCelebrate?.(3500);

    // Create internal completion notification
    this.callbacks.onNotification?.(
      "Focus Complete",
      `${completedPresetMinutes}m finished · Great focus session!`,
      4500
    );

    this.notify();
  }

  private startTickLoop(): void {
    this.stopTickLoop();
    this.tickInterval = setInterval(() => {
      this.tick();
    }, 250);
  }

  private stopTickLoop(): void {
    if (this.tickInterval !== null) {
      clearInterval(this.tickInterval);
      this.tickInterval = null;
    }
  }

  private notify(): void {
    for (const listener of this.listeners) {
      listener(this.state);
    }
  }

  public destroy(): void {
    this.stopTickLoop();
    this.stop();
    this.listeners.clear();
  }
}
