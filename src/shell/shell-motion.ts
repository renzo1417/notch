/**
 * Shell transition animation timing and easing curves.
 * Defined in PRODUCT_SPEC.md section 9.
 */
export const SHELL_MOTION = {
  microMs: 120,
  shellMs: 220,
  contextMs: 180,
  mascotMs: 350,
  easingNatural: "cubic-bezier(0.16, 1, 0.3, 1)",
} as const;
