/**
 * Focus timer model.
 * Defined in PRODUCT_SPEC.md section 15.
 */
export interface FocusTimerState {
  presetMinutes: number;
  remainingSeconds: number;
  isRunning: boolean;
  isPaused: boolean;
}
