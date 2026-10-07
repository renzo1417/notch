import type { MascotConfig, MascotRenderState, MascotState } from "./types";

const INACTIVITY_TIMEOUT_MS = 45_000;
const DIZZY_CLICK_THRESHOLD = 5;
const DIZZY_WINDOW_MS = 1_500;
const DIZZY_DURATION_MS = 2_600;
const DIZZY_COOLDOWN_MS = 3_500;
const BLINK_DURATION_MS = 140;

export class MascotStateMachine {
  private config: MascotConfig;
  private state: MascotRenderState;
  private lastActivityMs: number = Date.now();
  private clickTimestamps: number[] = [];
  private dizzyCooldownUntilMs: number = 0;
  private nextBlinkScheduledMs: number = 0;
  private isBlinking: boolean = false;
  private blinkTimerMs: number = 0;
  private stateTimerMs: number = 0;
  private isDoubleBlinkQueued: boolean = false;
  private onStateChangeCallback?: (state: MascotState) => void;

  constructor(config: MascotConfig, onStateChange?: (state: MascotState) => void) {
    this.config = config;
    this.onStateChangeCallback = onStateChange;

    this.state = {
      currentState: "idle",
      previousState: "idle",
      stateTimerMs: 0,
      gaze: { x: 0, y: 0 },
      targetGaze: { x: 0, y: 0 },
      blinkProgress: 0,
      squashX: 1.0,
      squashY: 1.0,
      squashVelocityY: 0,
      headTiltRad: 0,
      breathingOffset: 0,
      dizzyAngleRad: 0,
      sleepParticles: [],
      celebrateHop: 0,
      warningAlertProgress: 0,
    };

    this.scheduleNextBlink();
  }

  public getState(): Readonly<MascotRenderState> {
    return this.state;
  }

  public getCurrentState(): MascotState {
    return this.state.currentState;
  }

  public setConfig(newConfig: Partial<MascotConfig>) {
    this.config = { ...this.config, ...newConfig };
  }

  private scheduleNextBlink(minMs = 2800, maxMs = 6000) {
    const delay = minMs + Math.random() * (maxMs - minMs);
    this.nextBlinkScheduledMs = Date.now() + delay;
  }

  public setState(nextState: MascotState, customDurationMs?: number) {
    if (this.state.currentState === "dizzy" && nextState !== "idle" && nextState !== "dizzy") {
      // Dizzy state cannot be easily interrupted by non-idle states
      return;
    }

    if (this.state.currentState !== nextState) {
      this.state.previousState = this.state.currentState;
      this.state.currentState = nextState;
      this.stateTimerMs = customDurationMs ?? 0;
      this.onStateChangeCallback?.(nextState);

      // Adjust head tilt target based on state
      if (nextState === "curious") {
        this.state.headTiltRad = 0.087; // ~5 degrees
      } else if (nextState === "idle" || nextState === "focused") {
        this.state.headTiltRad = 0;
      }
    }
  }

  public recordPointerMove(relX: number, relY: number, isHoveringMascot: boolean) {
    this.lastActivityMs = Date.now();

    // Wake up if currently sleepy
    if (this.state.currentState === "sleepy") {
      this.wakeUp();
      return;
    }

    if (!this.config.trackCursor) {
      this.state.targetGaze = { x: 0, y: 0 };
      return;
    }

    // Clamp gaze to socket limits (-4 to +4 px X, -3 to +3 px Y)
    const clampedX = Math.max(-4, Math.min(4, relX * 0.12));
    const clampedY = Math.max(-3, Math.min(3, relY * 0.1));
    this.state.targetGaze = { x: clampedX, y: clampedY };

    // If mouse is near / hovering mascot and in idle, become curious or look
    if (isHoveringMascot && this.state.currentState === "idle") {
      this.setState("curious");
    } else if (!isHoveringMascot && this.state.currentState === "curious") {
      this.setState("idle");
    } else if (this.state.currentState === "idle" && (Math.abs(clampedX) > 1.5 || Math.abs(clampedY) > 1.2)) {
      this.state.currentState = "look";
    } else if (this.state.currentState === "look" && Math.abs(clampedX) <= 1.2 && Math.abs(clampedY) <= 1.0) {
      this.state.currentState = "idle";
    }
  }

  public handlePointerLeave() {
    this.state.targetGaze = { x: 0, y: 0 };
    if (this.state.currentState === "curious" || this.state.currentState === "look") {
      this.setState("idle");
    }
  }

  public recordClick() {
    const now = Date.now();
    this.lastActivityMs = now;

    // Wake up if sleepy
    if (this.state.currentState === "sleepy") {
      this.wakeUp();
      return;
    }

    // Apply tactile squash & stretch impulse
    this.state.squashY = 0.82;
    this.state.squashX = 1.14;
    this.state.squashVelocityY = -0.06;

    // Record click for dizzy detection
    this.clickTimestamps = this.clickTimestamps.filter((t) => now - t < DIZZY_WINDOW_MS);
    this.clickTimestamps.push(now);

    // Check rapid click dizzy threshold if cooldown expired
    if (
      this.clickTimestamps.length >= DIZZY_CLICK_THRESHOLD &&
      now > this.dizzyCooldownUntilMs
    ) {
      this.triggerDizzy();
      return;
    }

    // Gentle happy reaction on normal single click if in idle/curious
    if (this.state.currentState === "idle" || this.state.currentState === "curious") {
      this.setState("happy", 900);
    }
  }

  private triggerDizzy() {
    this.setState("dizzy", DIZZY_DURATION_MS);
    this.dizzyCooldownUntilMs = Date.now() + DIZZY_DURATION_MS + DIZZY_COOLDOWN_MS;
    this.clickTimestamps = [];
  }

  private wakeUp() {
    this.triggerBlink();
    this.setState("happy", 1100);
  }

  public triggerBlink() {
    this.isBlinking = true;
    this.blinkTimerMs = BLINK_DURATION_MS;
  }

  /**
   * Main simulation update step called on each frame.
   * Returns true if visual elements are active/moving (needs redraw).
   */
  public update(dtMs: number): boolean {
    const now = Date.now();
    let isDirty = false;

    // 1. Inactivity detection: drift to sleepy after 45s of quiet
    if (
      this.state.currentState === "idle" &&
      now - this.lastActivityMs > INACTIVITY_TIMEOUT_MS
    ) {
      this.setState("sleepy");
      isDirty = true;
    }

    // 2. Handle timed states (happy, dizzy, celebrate, warning)
    if (this.stateTimerMs > 0) {
      this.stateTimerMs -= dtMs;
      isDirty = true;
      if (this.stateTimerMs <= 0) {
        if (this.state.currentState === "dizzy") {
          // After dizziness, show a brief recovery happy smile before idle
          this.setState("happy", 900);
        } else {
          this.setState("idle");
        }
      }
    }

    // 3. Blink scheduling & progression
    if (!this.isBlinking && now >= this.nextBlinkScheduledMs && this.state.currentState !== "sleepy" && this.state.currentState !== "dizzy") {
      this.isBlinking = true;
      this.blinkTimerMs = BLINK_DURATION_MS;
      // 18% chance of double blink
      this.isDoubleBlinkQueued = Math.random() < 0.18;
      isDirty = true;
    }

    if (this.isBlinking) {
      this.blinkTimerMs -= dtMs;
      isDirty = true;
      if (this.blinkTimerMs <= 0) {
        if (this.isDoubleBlinkQueued) {
          this.isDoubleBlinkQueued = false;
          this.blinkTimerMs = BLINK_DURATION_MS * 0.75;
          this.state.blinkProgress = 0.5;
        } else {
          this.isBlinking = false;
          this.state.blinkProgress = 0;
          this.scheduleNextBlink();
        }
      } else {
        // Half-sine blink curve (0 -> 1 -> 0)
        const halfDuration = BLINK_DURATION_MS / 2;
        const progress = Math.abs(this.blinkTimerMs - halfDuration) / halfDuration;
        this.state.blinkProgress = 1.0 - progress;
      }
    }

    // 4. Smooth Gaze Interpolation (Lerp)
    const gazeDx = this.state.targetGaze.x - this.state.gaze.x;
    const gazeDy = this.state.targetGaze.y - this.state.gaze.y;
    if (Math.abs(gazeDx) > 0.04 || Math.abs(gazeDy) > 0.04) {
      this.state.gaze.x += gazeDx * 0.16;
      this.state.gaze.y += gazeDy * 0.16;
      isDirty = true;
    } else if (this.state.gaze.x !== this.state.targetGaze.x || this.state.gaze.y !== this.state.targetGaze.y) {
      this.state.gaze.x = this.state.targetGaze.x;
      this.state.gaze.y = this.state.targetGaze.y;
      isDirty = true;
    }

    // 5. Harmonic Spring Physics for Squash & Stretch
    // Hooke's Law: F = -k * x - c * v
    const springK = 0.022; // Spring constant
    const damping = 0.82; // Damping ratio
    const displacementY = this.state.squashY - 1.0;
    const forceY = -springK * displacementY * 100;
    this.state.squashVelocityY = (this.state.squashVelocityY + forceY * (dtMs / 16.6)) * damping;
    this.state.squashY += this.state.squashVelocityY;
    // Volume conservation: squashX is inversely proportional to squashY
    this.state.squashX = 1.0 + (1.0 - this.state.squashY) * 0.75;

    if (Math.abs(displacementY) > 0.003 || Math.abs(this.state.squashVelocityY) > 0.003) {
      isDirty = true;
    } else if (this.state.squashX !== 1.0 || this.state.squashY !== 1.0) {
      this.state.squashX = 1.0;
      this.state.squashY = 1.0;
      this.state.squashVelocityY = 0;
      isDirty = true;
    }

    // 6. Subtle Visible-Only Breathing (Sine wave)
    const isBreathingPaused =
      this.config.animationIntensity <= 0 ||
      (this.config.pauseInactiveAnimation === true &&
        now - this.lastActivityMs > 5000 &&
        this.state.currentState === "idle");

    if (isBreathingPaused) {
      if (Math.abs(this.state.breathingOffset) > 0.02) {
        this.state.breathingOffset *= 0.82; // smoothly decay to 0
        isDirty = true;
      } else if (this.state.breathingOffset !== 0) {
        this.state.breathingOffset = 0;
        isDirty = true;
      }
    } else {
      const breathingPeriodMs = this.state.currentState === "sleepy" ? 4200 : 3400;
      const breathingPhase = (now % breathingPeriodMs) / breathingPeriodMs;
      const intensity = Math.min(1.0, Math.max(0.0, this.config.animationIntensity));
      const breathingAmplitude = (this.state.currentState === "sleepy" ? 1.4 : 0.8) * intensity;
      const newBreathingOffset = Math.sin(breathingPhase * Math.PI * 2) * breathingAmplitude;

      // Only repaint if breathing movement is perceptible (>= 0.04px) or if already dirty from other motion
      if (Math.abs(newBreathingOffset - this.state.breathingOffset) >= 0.04) {
        this.state.breathingOffset = newBreathingOffset;
        isDirty = true;
      } else if (isDirty) {
        this.state.breathingOffset = newBreathingOffset;
      }
    }

    // 7. Dizzy rotation
    if (this.state.currentState === "dizzy") {
      this.state.dizzyAngleRad = (this.state.dizzyAngleRad + (dtMs * 0.012)) % (Math.PI * 2);
      isDirty = true;
    }

    // 8. Sleep particle simulation ('z' bubbles)
    if (this.state.currentState === "sleepy") {
      // Spawn new z every ~1.8s
      if (Math.random() < 0.025 && this.state.sleepParticles.length < 3) {
        this.state.sleepParticles.push({
          x: 14 + (Math.random() * 4 - 2),
          y: -10,
          opacity: 0.9,
          size: 7 + Math.random() * 2,
          age: 0,
        });
        isDirty = true;
      }

      if (this.state.sleepParticles.length > 0) {
        this.state.sleepParticles = this.state.sleepParticles
          .map((p) => ({
            ...p,
            x: p.x + Math.sin(p.age * 0.05) * 0.35,
            y: p.y - 0.45,
            opacity: Math.max(0, p.opacity - 0.012),
            age: p.age + 1,
          }))
          .filter((p) => p.opacity > 0);
        isDirty = true;
      }
    } else if (this.state.sleepParticles.length > 0) {
      this.state.sleepParticles = [];
      isDirty = true;
    }

    // 9. Celebrate hop physics
    if (this.state.currentState === "celebrate") {
      const hopCycle = (now % 800) / 800;
      this.state.celebrateHop = -Math.sin(hopCycle * Math.PI) * 4.5;
      isDirty = true;
    } else if (this.state.celebrateHop !== 0) {
      this.state.celebrateHop = 0;
      isDirty = true;
    }

    // 10. Warning alert pulse
    if (this.state.currentState === "warning") {
      const alertPhase = (now % 900) / 900;
      this.state.warningAlertProgress = Math.sin(alertPhase * Math.PI * 2);
      isDirty = true;
    } else if (this.state.warningAlertProgress !== 0) {
      this.state.warningAlertProgress = 0;
      isDirty = true;
    }

    return isDirty;
  }
}
