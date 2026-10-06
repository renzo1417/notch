/**
 * Centralized deterministic context-priority resolver and state engine.
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

export const ContextPriority = {
  P0_SYSTEM_WARNING: 0,
  P1_COMPLETION: 1,
  P2_FOCUS_TIMER: 2,
  P3_DOWNLOAD: 3,
  P4_CODING: 4,
  P5_USER_SURFACE: 5,
  P6_IDLE: 6,
} as const;

export type ContextPriority = (typeof ContextPriority)[keyof typeof ContextPriority];


export interface ContextCandidate<T = unknown> {
  id: string;
  mode: ContextMode;
  priority: ContextPriority;
  timestamp: number;
  expiresAt?: number;
  data?: T;
}

export const DEFAULT_IDLE_CANDIDATE: ContextCandidate = {
  id: "idle",
  mode: "idle",
  priority: ContextPriority.P6_IDLE,
  timestamp: 0,
};

/**
 * Checks whether a candidate is currently unexpired.
 */
export function isCandidateActive(candidate: ContextCandidate, now: number = Date.now()): boolean {
  if (candidate.expiresAt !== undefined && candidate.expiresAt <= now) {
    return false;
  }
  return true;
}

/**
 * Deterministically resolves the winning candidate from a list of candidates.
 * Lower priority number = higher precedence (P0 > P1 > P2 > P3 > P4 > P5 > P6).
 * Ties are broken by the most recent timestamp.
 * Unexpired candidates only; returns DEFAULT_IDLE_CANDIDATE if no candidates qualify.
 */
export function resolveActiveCandidate(
  candidates: ContextCandidate[],
  now: number = Date.now()
): ContextCandidate {
  const activeCandidates = candidates.filter((c) => isCandidateActive(c, now));

  if (activeCandidates.length === 0) {
    return DEFAULT_IDLE_CANDIDATE;
  }

  const sorted = [...activeCandidates].sort((a, b) => {
    // 1. Primary: Priority order (P0 < P1 < ... < P6)
    if (a.priority !== b.priority) {
      return a.priority - b.priority;
    }
    // 2. Secondary: Most recent timestamp wins
    if (a.timestamp !== b.timestamp) {
      return b.timestamp - a.timestamp;
    }
    // 3. Stable tie-breaker: ID alphabetical comparison
    return a.id.localeCompare(b.id);
  });

  return sorted[0];
}

/**
 * Deterministically resolves the active ContextMode.
 */
export function resolveActiveContext(
  candidates: ContextCandidate[],
  now: number = Date.now()
): ContextMode {
  return resolveActiveCandidate(candidates, now).mode;
}

/**
 * Reactive Context Engine that manages persistent and temporary candidates,
 * schedules clean expiration of temporary contexts, and reveals the previous
 * meaningful state upon expiration.
 */
export class ContextEngine {
  private candidates = new Map<string, ContextCandidate>();
  private expirationTimers = new Map<string, ReturnType<typeof setTimeout>>();
  private listeners = new Set<(candidate: ContextCandidate) => void>();

  private activeCandidate: ContextCandidate = DEFAULT_IDLE_CANDIDATE;
  private previousMeaningfulCandidate: ContextCandidate = DEFAULT_IDLE_CANDIDATE;

  constructor() {
    this.candidates.set("idle", DEFAULT_IDLE_CANDIDATE);
    this.recompute();
  }

  public get current(): ContextCandidate {
    return this.activeCandidate;
  }

  public get currentMode(): ContextMode {
    return this.activeCandidate.mode;
  }

  public get previousMeaningful(): ContextCandidate {
    return this.previousMeaningfulCandidate;
  }

  public subscribe(callback: (candidate: ContextCandidate) => void): () => void {
    this.listeners.add(callback);
    callback(this.activeCandidate);
    return () => {
      this.listeners.delete(callback);
    };
  }

  /**
   * Registers or updates a stateful candidate (e.g. active coding session, focus timer).
   */
  public registerCandidate(candidate: ContextCandidate): void {
    this.candidates.set(candidate.id, candidate);
    this.recompute();
  }

  /**
   * Unregisters a candidate by ID (e.g. coding session ended, timer stopped).
   */
  public unregisterCandidate(id: string): void {
    if (id === "idle") return; // idle is the base fallback

    this.clearTimer(id);
    this.candidates.delete(id);
    this.recompute();
  }

  /**
   * Pushes a temporary short-lived context (e.g. completion notification, warning).
   * Automatically expires after `durationMs` and seamlessly reveals the previous state.
   */
  public pushTemporaryContext(params: {
    id?: string;
    mode: ContextMode;
    priority?: ContextPriority;
    durationMs: number;
    data?: unknown;
  }): string {
    const id = params.id ?? `temp_${params.mode}_${Date.now()}`;
    const now = Date.now();
    const priority = params.priority ?? ContextPriority.P1_COMPLETION;

    // Clear existing timer if updating the same ID
    this.clearTimer(id);

    const candidate: ContextCandidate = {
      id,
      mode: params.mode,
      priority,
      timestamp: now,
      expiresAt: now + params.durationMs,
      data: params.data,
    };

    this.candidates.set(id, candidate);
    this.recompute();

    // Schedule clean expiration
    const timer = setTimeout(() => {
      this.expirationTimers.delete(id);
      this.candidates.delete(id);
      this.recompute();
    }, params.durationMs);

    this.expirationTimers.set(id, timer);
    return id;
  }

  /**
   * Manually dismisses a temporary context before natural expiration.
   */
  public dismissTemporaryContext(id: string): void {
    this.clearTimer(id);
    this.candidates.delete(id);
    this.recompute();
  }

  /**
   * Cleans up all timers on shutdown.
   */
  public destroy(): void {
    for (const timer of this.expirationTimers.values()) {
      clearTimeout(timer);
    }
    this.expirationTimers.clear();
    this.listeners.clear();
  }

  private clearTimer(id: string): void {
    const timer = this.expirationTimers.get(id);
    if (timer) {
      clearTimeout(timer);
      this.expirationTimers.delete(id);
    }
  }

  private recompute(): void {
    const now = Date.now();
    const candidateList = Array.from(this.candidates.values());
    const nextCandidate = resolveActiveCandidate(candidateList, now);

    if (
      this.activeCandidate.id !== nextCandidate.id ||
      this.activeCandidate.mode !== nextCandidate.mode ||
      this.activeCandidate.timestamp !== nextCandidate.timestamp
    ) {
      // Track previous meaningful state (stateful, non-temporary candidate)
      if (
        this.activeCandidate.id !== "idle" &&
        this.activeCandidate.expiresAt === undefined
      ) {
        this.previousMeaningfulCandidate = this.activeCandidate;
      }

      this.activeCandidate = nextCandidate;
      this.notify();
    }
  }

  private notify(): void {
    for (const listener of this.listeners) {
      listener(this.activeCandidate);
    }
  }
}

// Global context engine instance
export const contextEngine = new ContextEngine();
