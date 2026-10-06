/**
 * Mascot state model.
 * Defined in PRODUCT_SPEC.md section 12.
 */
export type MascotState =
  | "idle"
  | "blink"
  | "look"
  | "curious"
  | "happy"
  | "focused"
  | "sleepy"
  | "celebrate"
  | "warning"
  | "dizzy";

export interface MascotConfig {
  enabled: boolean;
  trackCursor: boolean;
  animationIntensity: number;
}
