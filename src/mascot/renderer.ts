import { GRAPHITE } from "../styles/palette.ts";
import type { MascotDimensions, MascotRenderState, MascotState } from "./types";

export interface DrawOptions {
  ctx: CanvasRenderingContext2D;
  state: MascotRenderState;
  dimensions: MascotDimensions;
  dpr: number;
}

export class MascotRenderer {
  /**
   * Main render method for procedural companion "Pip".
   */
  public render({ ctx, state, dimensions, dpr }: DrawOptions): void {
    const { width, height } = dimensions;

    // Reset transform and clear viewport
    ctx.save();
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, width, height);

    const centerX = width / 2;
    const centerY = height / 2;

    // 1. Calculate base dynamics: breathing, hop, and squash anchor
    const breathingY = state.breathingOffset;
    const hopY = state.celebrateHop;
    const warningShakeX =
      state.currentState === "warning" ? state.warningAlertProgress * 1.2 : 0;

    const scale = dimensions.scale || 1.0;
    const anchorY = centerY + 18 * scale; // Base anchor for squash & stretch

    ctx.save();
    // Translate to bottom anchor of mascot
    ctx.translate(centerX + warningShakeX * scale, anchorY + (breathingY + hopY) * scale);

    // Apply head tilt
    if (state.headTiltRad !== 0) {
      ctx.rotate(state.headTiltRad);
    }

    // Apply squash and stretch relative to ground anchor, multiplied by target scale
    ctx.scale(scale * state.squashX, scale * state.squashY);

    // Translate back up so mascot center is at (0, -18)
    const localCenterY = -18;

    // 2. Render Sensor Ears / Antennae (behind chassis)
    this.drawSensorEars(ctx, localCenterY, state);

    // 3. Render Chassis Body
    this.drawChassis(ctx, localCenterY);

    // 4. Render Visor Faceplate
    this.drawVisor(ctx, localCenterY);

    // 5. Render Eyes & Expression
    this.drawEyes(ctx, localCenterY, state);

    // 6. Render Overlays (Dizzy Stars, Sleep Particles)
    this.drawOverlays(ctx, localCenterY, state);

    ctx.restore(); // Restore anchor transform
    ctx.restore(); // Restore DPR transform
  }

  /**
   * Sensor ears at top of chassis that pivot with physical state.
   */
  private drawSensorEars(
    ctx: CanvasRenderingContext2D,
    centerY: number,
    state: MascotRenderState
  ): void {
    const earWidth = 4.5;
    const earHeight = 8;
    const earRadius = 2.25;

    // Ear pivot angles based on emotional state
    let leftEarAngle = -0.32; // ~ -18 deg
    let rightEarAngle = 0.32; // ~ +18 deg

    if (state.currentState === "curious") {
      leftEarAngle = -0.52; // Perk up left ear
      rightEarAngle = 0.18;
    } else if (state.currentState === "happy" || state.currentState === "celebrate") {
      leftEarAngle = -0.55;
      rightEarAngle = 0.55;
    } else if (state.currentState === "sleepy") {
      leftEarAngle = -0.12; // Droop ears down
      rightEarAngle = 0.12;
    } else if (state.currentState === "dizzy") {
      const wobble = Math.sin(state.dizzyAngleRad * 2) * 0.35;
      leftEarAngle = -0.32 + wobble;
      rightEarAngle = 0.32 - wobble;
    }

    const tipColor = this.getAccentColorForState(state.currentState);

    // Left Ear
    ctx.save();
    ctx.translate(-13, centerY - 15);
    ctx.rotate(leftEarAngle);
    this.drawSingleEar(ctx, earWidth, earHeight, earRadius, tipColor);
    ctx.restore();

    // Right Ear
    ctx.save();
    ctx.translate(13, centerY - 15);
    ctx.rotate(rightEarAngle);
    this.drawSingleEar(ctx, earWidth, earHeight, earRadius, tipColor);
    ctx.restore();
  }

  private drawSingleEar(
    ctx: CanvasRenderingContext2D,
    w: number,
    h: number,
    r: number,
    tipColor: string
  ): void {
    // Ear base
    ctx.fillStyle = GRAPHITE.ear;
    this.roundedRect(ctx, -w / 2, -h, w, h, r);
    ctx.fill();

    // Ear tip LED
    ctx.fillStyle = tipColor;
    ctx.beginPath();
    ctx.arc(0, -h + 2.5, 1.4, 0, Math.PI * 2);
    ctx.fill();
  }

  /**
   * Main chassis body: dark slate titanium rounded squircle with rim highlight.
   */
  private drawChassis(
    ctx: CanvasRenderingContext2D,
    centerY: number
  ): void {
    const chassisW = 46;
    const chassisH = 38;
    const radius = 13;
    const x = -chassisW / 2;
    const y = centerY - chassisH / 2;

    // Body base
    ctx.fillStyle = GRAPHITE.chassis;
    this.roundedRect(ctx, x, y, chassisW, chassisH, radius);
    ctx.fill();

    // Subtle 1px chassis border
    ctx.strokeStyle = "rgba(255, 255, 255, 0.08)";
    ctx.lineWidth = 1;
    ctx.stroke();

    // Top specular rim highlight
    ctx.save();
    ctx.beginPath();
    this.roundedRect(ctx, x + 1, y + 1, chassisW - 2, 8, radius - 1);
    ctx.clip();
    const rimGrad = ctx.createLinearGradient(0, y, 0, y + 6);
    rimGrad.addColorStop(0, "rgba(255, 255, 255, 0.12)");
    rimGrad.addColorStop(1, "rgba(255, 255, 255, 0.0)");
    ctx.fillStyle = rimGrad;
    ctx.fillRect(x, y, chassisW, 7);
    ctx.restore();
  }

  /**
   * Visor: Dark glossy inner display faceplate.
   */
  private drawVisor(ctx: CanvasRenderingContext2D, centerY: number): void {
    const visorW = 38;
    const visorH = 26;
    const radius = 9;
    const x = -visorW / 2;
    const y = centerY - visorH / 2 + 1;

    // Deep obsidian background
    ctx.fillStyle = GRAPHITE.visor;
    this.roundedRect(ctx, x, y, visorW, visorH, radius);
    ctx.fill();

    // Subtle inner stroke
    ctx.strokeStyle = "rgba(255, 255, 255, 0.05)";
    ctx.lineWidth = 1;
    ctx.stroke();

    // Subtle diagonal gloss reflection across upper right
    ctx.save();
    ctx.beginPath();
    this.roundedRect(ctx, x, y, visorW, visorH, radius);
    ctx.clip();

    const gloss = ctx.createLinearGradient(x + 5, y - 5, x + visorW, y + visorH);
    gloss.addColorStop(0, "rgba(255, 255, 255, 0.07)");
    gloss.addColorStop(0.35, "rgba(255, 255, 255, 0.02)");
    gloss.addColorStop(0.5, "rgba(255, 255, 255, 0.0)");
    ctx.fillStyle = gloss;
    ctx.fillRect(x, y, visorW, visorH);
    ctx.restore();
  }

  /**
   * Procedural expressive eyes.
   */
  private drawEyes(
    ctx: CanvasRenderingContext2D,
    centerY: number,
    state: MascotRenderState
  ): void {
    const eyeBaseY = centerY + 1.5;
    const eyeSpacing = 8.5; // Offset from center X
    const leftEyeX = -eyeSpacing + state.gaze.x;
    const rightEyeX = eyeSpacing + state.gaze.x;
    const eyeY = eyeBaseY + state.gaze.y;

    const accentColor = this.getAccentColorForState(state.currentState);

    ctx.save();

    // Handle states
    switch (state.currentState) {
      case "blink":
        this.drawBlinkEyes(ctx, leftEyeX, rightEyeX, eyeY, accentColor, state.blinkProgress);
        break;

      case "happy":
        this.drawHappyEyes(ctx, leftEyeX, rightEyeX, eyeY, accentColor);
        break;

      case "focused":
        this.drawFocusedEyes(ctx, leftEyeX, rightEyeX, eyeY, accentColor);
        break;

      case "sleepy":
        this.drawSleepyEyes(ctx, leftEyeX, rightEyeX, eyeY, accentColor);
        break;

      case "dizzy":
        this.drawDizzyEyes(ctx, leftEyeX, rightEyeX, eyeY, state.dizzyAngleRad);
        break;

      case "celebrate":
        this.drawCelebrateEyes(ctx, leftEyeX, rightEyeX, eyeY, accentColor);
        break;

      case "warning":
        this.drawWarningEyes(ctx, leftEyeX, rightEyeX, eyeY, accentColor);
        break;

      case "curious":
        this.drawCuriousEyes(ctx, leftEyeX, rightEyeX, eyeY, accentColor);
        break;

      case "look":
      case "idle":
      default:
        this.drawStandardEyes(ctx, leftEyeX, rightEyeX, eyeY, accentColor, state.blinkProgress);
        break;
    }

    ctx.restore();
  }

  private drawStandardEyes(
    ctx: CanvasRenderingContext2D,
    lx: number,
    rx: number,
    y: number,
    color: string,
    blinkProgress: number
  ): void {
    const width = 6;
    const height = Math.max(1.5, 8 * (1 - blinkProgress));
    const radius = Math.min(width, height) / 2;

    this.drawGlowingPill(ctx, lx, y, width, height, radius, color);
    this.drawGlowingPill(ctx, rx, y, width, height, radius, color);
  }

  private drawBlinkEyes(
    ctx: CanvasRenderingContext2D,
    lx: number,
    rx: number,
    y: number,
    color: string,
    progress: number
  ): void {
    const width = 7.5;
    const height = Math.max(1.2, 8 * (1 - progress));
    const radius = height / 2;

    this.drawGlowingPill(ctx, lx, y, width, height, radius, color);
    this.drawGlowingPill(ctx, rx, y, width, height, radius, color);
  }

  private drawCuriousEyes(
    ctx: CanvasRenderingContext2D,
    lx: number,
    rx: number,
    y: number,
    color: string
  ): void {
    // Left eye is widened inquisitively, right eye slightly narrowed
    this.drawGlowingPill(ctx, lx, y - 0.5, 6.8, 8.8, 3.4, color);
    this.drawGlowingPill(ctx, rx, y + 0.5, 5.2, 7.2, 2.6, color);
  }

  private drawHappyEyes(
    ctx: CanvasRenderingContext2D,
    lx: number,
    rx: number,
    y: number,
    color: string
  ): void {
    // Cheerful upward arcs ◠ ◠
    ctx.strokeStyle = color;
    ctx.lineWidth = 2.2;
    ctx.lineCap = "round";

    // Left arch
    ctx.beginPath();
    ctx.arc(lx, y + 1, 3.8, Math.PI * 1.15, Math.PI * 1.85);
    ctx.stroke();

    // Right arch
    ctx.beginPath();
    ctx.arc(rx, y + 1, 3.8, Math.PI * 1.15, Math.PI * 1.85);
    ctx.stroke();
  }

  private drawFocusedEyes(
    ctx: CanvasRenderingContext2D,
    lx: number,
    rx: number,
    y: number,
    color: string
  ): void {
    // Narrow determined horizontal visor apertures
    const width = 8;
    const height = 3.2;
    const radius = 1.6;

    this.drawGlowingPill(ctx, lx, y, width, height, radius, color);
    this.drawGlowingPill(ctx, rx, y, width, height, radius, color);
  }

  private drawSleepyEyes(
    ctx: CanvasRenderingContext2D,
    lx: number,
    rx: number,
    y: number,
    color: string
  ): void {
    // Drooping heavy eyelids with subdued glow
    const width = 6.5;
    const height = 2.6;
    const radius = 1.3;

    ctx.save();
    ctx.globalAlpha = 0.72;
    this.drawGlowingPill(ctx, lx, y + 1.2, width, height, radius, color);
    this.drawGlowingPill(ctx, rx, y + 1.2, width, height, radius, color);
    ctx.restore();
  }

  private drawDizzyEyes(
    ctx: CanvasRenderingContext2D,
    lx: number,
    rx: number,
    y: number,
    angleRad: number
  ): void {
    // Spinning spirals in opposite directions
    this.drawSingleSpiral(ctx, lx, y, 4, angleRad, GRAPHITE.text);
    this.drawSingleSpiral(ctx, rx, y, 4, -angleRad, GRAPHITE.text);
  }

  private drawSingleSpiral(
    ctx: CanvasRenderingContext2D,
    x: number,
    y: number,
    maxRadius: number,
    angle: number,
    color: string
  ): void {
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(angle);
    ctx.strokeStyle = color;
    ctx.lineWidth = 1.6;
    ctx.lineCap = "round";

    ctx.beginPath();
    for (let theta = 0; theta < Math.PI * 3; theta += 0.25) {
      const r = (theta / (Math.PI * 3)) * maxRadius;
      const px = Math.cos(theta) * r;
      const py = Math.sin(theta) * r;
      if (theta === 0) {
        ctx.moveTo(px, py);
      } else {
        ctx.lineTo(px, py);
      }
    }
    ctx.stroke();
    ctx.restore();
  }

  private drawCelebrateEyes(
    ctx: CanvasRenderingContext2D,
    lx: number,
    rx: number,
    y: number,
    color: string
  ): void {
    // Four-point sparkle stars for celebration
    this.drawSparkle(ctx, lx, y, 4.5, color);
    this.drawSparkle(ctx, rx, y, 4.5, color);
  }

  private drawSparkle(
    ctx: CanvasRenderingContext2D,
    x: number,
    y: number,
    size: number,
    color: string
  ): void {
    ctx.save();
    ctx.fillStyle = color;
    ctx.beginPath();
    ctx.moveTo(x, y - size);
    ctx.quadraticCurveTo(x, y, x + size, y);
    ctx.quadraticCurveTo(x, y, x, y + size);
    ctx.quadraticCurveTo(x, y, x - size, y);
    ctx.quadraticCurveTo(x, y, x, y - size);
    ctx.fill();
    ctx.restore();
  }

  private drawWarningEyes(
    ctx: CanvasRenderingContext2D,
    lx: number,
    rx: number,
    y: number,
    color: string
  ): void {
    // Alert apertures
    const width = 6.5;
    const height = 6.5;
    const radius = 2.0;

    this.drawGlowingPill(ctx, lx, y, width, height, radius, color);
    this.drawGlowingPill(ctx, rx, y, width, height, radius, color);
  }

  /**
   * Helper to draw a luminous rounded pill eye with subtle glow.
   */
  private drawGlowingPill(
    ctx: CanvasRenderingContext2D,
    x: number,
    y: number,
    w: number,
    h: number,
    r: number,
    color: string
  ): void {
    // Ambient soft glow
    ctx.save();
    ctx.shadowColor = color;
    ctx.shadowBlur = 1.5;
    ctx.fillStyle = color;
    this.roundedRect(ctx, x - w / 2, y - h / 2, w, h, r);
    ctx.fill();
    ctx.restore();

    // Inner bright core
    ctx.fillStyle = "#ffffff";
    ctx.globalAlpha = 0.35;
    this.roundedRect(ctx, x - w / 4, y - h / 4, w / 2, h / 2, r / 2);
    ctx.fill();
    ctx.globalAlpha = 1.0;
  }

  /**
   * Overlays: Dizzy stars and Sleep 'z' particles.
   */
  private drawOverlays(
    ctx: CanvasRenderingContext2D,
    centerY: number,
    state: MascotRenderState
  ): void {
    // Dizzy orbital stars
    if (state.currentState === "dizzy") {
      const orbitRadiusX = 22;
      const orbitRadiusY = 7;
      const angle = state.dizzyAngleRad;

      this.drawOrbitStar(
        ctx,
        Math.cos(angle) * orbitRadiusX,
        centerY - 22 + Math.sin(angle) * orbitRadiusY,
        3.0,
        GRAPHITE.text
      );

      this.drawOrbitStar(
        ctx,
        Math.cos(angle + Math.PI) * orbitRadiusX,
        centerY - 22 + Math.sin(angle + Math.PI) * orbitRadiusY,
        2.5,
        GRAPHITE.text
      );
    }

    // Sleep 'z' particles
    if (state.sleepParticles.length > 0) {
      ctx.save();
      ctx.font = "bold 9px 'Segoe UI Variable', sans-serif";
      ctx.fillStyle = GRAPHITE.secondary;

      for (const p of state.sleepParticles) {
        ctx.globalAlpha = p.opacity;
        ctx.fillText("z", p.x, centerY + p.y);
      }
      ctx.restore();
    }
  }

  private drawOrbitStar(
    ctx: CanvasRenderingContext2D,
    x: number,
    y: number,
    size: number,
    color: string
  ): void {
    ctx.save();
    ctx.fillStyle = color;
    ctx.beginPath();
    for (let i = 0; i < 5; i++) {
      const outerAngle = (i * Math.PI * 2) / 5 - Math.PI / 2;
      const innerAngle = outerAngle + Math.PI / 5;
      const ox = x + Math.cos(outerAngle) * size;
      const oy = y + Math.sin(outerAngle) * size;
      const ix = x + Math.cos(innerAngle) * (size * 0.45);
      const iy = y + Math.sin(innerAngle) * (size * 0.45);
      if (i === 0) ctx.moveTo(ox, oy);
      else ctx.lineTo(ox, oy);
      ctx.lineTo(ix, iy);
    }
    ctx.closePath();
    ctx.fill();
    ctx.restore();
  }

  private getAccentColorForState(state: MascotState): string {
    if (state === "warning") return GRAPHITE.warning;
    if (state === "sleepy") return GRAPHITE.secondary;
    return GRAPHITE.text;
  }

  /**
   * Pure canvas 2D rounded rectangle path utility.
   */
  private roundedRect(
    ctx: CanvasRenderingContext2D,
    x: number,
    y: number,
    w: number,
    h: number,
    r: number
  ): void {
    const clampedR = Math.max(0, Math.min(r, w / 2, h / 2));
    ctx.beginPath();
    ctx.moveTo(x + clampedR, y);
    ctx.arcTo(x + w, y, x + w, y + h, clampedR);
    ctx.arcTo(x + w, y + h, x, y + h, clampedR);
    ctx.arcTo(x, y + h, x, y, clampedR);
    ctx.arcTo(x, y, x + w, y, clampedR);
    ctx.closePath();
  }
}
