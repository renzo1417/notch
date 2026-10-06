/**
 * Mascot state model and configuration interfaces.
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

export interface GazeCoordinates {
  x: number;
  y: number;
}

export interface MascotConfig {
  enabled: boolean;
  trackCursor: boolean;
  animationIntensity: number; // 0.0 to 1.0 (respects reduced motion)
}

export interface MascotRenderState {
  currentState: MascotState;
  previousState: MascotState;
  stateTimerMs: number;
  // Eye & Gaze Dynamics
  gaze: GazeCoordinates;
  targetGaze: GazeCoordinates;
  blinkProgress: number; // 0.0 (open) to 1.0 (fully closed)
  // Physical Dynamics
  squashX: number; // Scale factor X (1.0 = normal)
  squashY: number; // Scale factor Y (1.0 = normal)
  squashVelocityY: number;
  headTiltRad: number;
  breathingOffset: number;
  // Special FX State
  dizzyAngleRad: number;
  sleepParticles: Array<{ x: number; y: number; opacity: number; size: number; age: number }>;
  celebrateHop: number;
  warningAlertProgress: number;
}

export interface MascotDimensions {
  width: number;
  height: number;
  scale: number;
}
