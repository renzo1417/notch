/**
 * Context surface modes and priority resolution engine.
 * Defined in PRODUCT_SPEC.md sections 4 & 5.
 */
export type ContextMode =
  | "idle"
  | "coding"
  | "timer"
  | "download"
  | "clipboard"
  | "notification"
  | "launcher"
  | "battery";

export enum ContextPriority {
  P0_SYSTEM_WARNING = 0,
  P1_COMPLETION = 1,
  P2_FOCUS_TIMER = 2,
  P3_DOWNLOAD = 3,
  P4_CODING = 4,
  P5_USER_SURFACE = 5,
  P6_IDLE = 6,
}

export interface ContextCandidate {
  mode: ContextMode;
  priority: ContextPriority;
  timestamp: number;
}

/**
 * Deterministically resolves which context surface outranks others.
 * Lower priority number = higher precedence.
 */
export function resolveActiveContext(candidates: ContextCandidate[]): ContextMode {
  if (candidates.length === 0) {
    return "idle";
  }

  const sorted = [...candidates].sort((a, b) => {
    if (a.priority !== b.priority) {
      return a.priority - b.priority;
    }
    // Ties resolve to most recent event
    return b.timestamp - a.timestamp;
  });

  return sorted[0].mode;
}
