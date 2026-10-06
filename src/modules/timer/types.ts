/**
 * Focus timer model and configuration.
 * Defined in PRODUCT_SPEC.md section 15.
 */

export type FocusPresetMinutes = 25 | 50;

export interface FocusTimerState {
  /** Active preset in minutes (25, 50, or custom) */
  presetMinutes: number;
  /** Custom duration in minutes configured by the user */
  customMinutes: number;
  /** Accurate remaining countdown seconds */
  remainingSeconds: number;
  /** Total duration seconds for the current session */
  totalSeconds: number;
  /** Whether the timer is currently actively running */
  isRunning: boolean;
  /** Whether the timer is currently paused */
  isPaused: boolean;
  /** Timestamp when timer was started or resumed */
  startTimestampMs: number | null;
  /** Target expiration timestamp in milliseconds (now + remainingSeconds * 1000) */
  endTimestampMs: number | null;
}

export const DEFAULT_FOCUS_TIMER_STATE: FocusTimerState = {
  presetMinutes: 25,
  customMinutes: 15,
  remainingSeconds: 25 * 60,
  totalSeconds: 25 * 60,
  isRunning: false,
  isPaused: false,
  startTimestampMs: null,
  endTimestampMs: null,
};
