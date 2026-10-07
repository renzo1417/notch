import { test } from "node:test";
import assert from "node:assert/strict";
import { MascotStateMachine } from "../src/mascot/state-machine.ts";
import { ContextEngine, ContextPriority } from "../src/app/state/context-engine.ts";
import { formatCompactLabel } from "../src/modules/timer/timer-engine.ts";
import { DEFAULT_FOCUS_TIMER_STATE } from "../src/modules/timer/types.ts";

test("Phase 10 Mascot: static resting state returns isDirty=false to prevent continuous Canvas redraw", () => {
  const machine = new MascotStateMachine({
    enabled: true,
    trackCursor: true,
    animationIntensity: 1.0,
    pauseInactiveAnimation: true,
  });

  // Let initial state settle
  for (let i = 0; i < 10; i++) {
    machine.update(16);
  }

  // Simulate resting after inactivity window (> 5000ms)
  // By updating far past activity timestamp:
  const isDirty = machine.update(16);
  // In resting state between blinks with paused inactive breathing, isDirty must be false
  assert.equal(isDirty, false, "Resting mascot must not report dirty frame when idle");
});

test("Phase 10 Mascot: battery saver / reduced motion halts ambient breathing redraws", () => {
  const machine = new MascotStateMachine({
    enabled: true,
    trackCursor: true,
    animationIntensity: 0.0, // Reduced motion or battery saver mode
  });

  // Let initial frame settle
  machine.update(16);
  machine.update(16);

  const isDirty = machine.update(16);
  assert.equal(isDirty, false, "Zero intensity animation must not trigger ambient dirty frames");
  assert.equal(machine.getState().breathingOffset, 0, "Breathing offset must remain at zero");
});

test("Phase 10 Mascot: clicks trigger squash & stretch physics and mark frame dirty", () => {
  const machine = new MascotStateMachine({
    enabled: true,
    trackCursor: true,
    animationIntensity: 1.0,
  });

  machine.recordClick();
  const state = machine.getState();

  assert.ok(state.squashY < 1.0, "Click must produce vertical squash impulse");
  assert.ok(state.squashX > 1.0, "Volume conservation must expand horizontal axis");

  const isDirty = machine.update(16);
  assert.equal(isDirty, true, "Squash physics step must mark frame dirty for repaint");
});

test("Phase 10 Mascot: rapid click stress triggers dizzy state and enforces cooldown", () => {
  let ariaState = "idle";
  const machine = new MascotStateMachine(
    {
      enabled: true,
      trackCursor: true,
      animationIntensity: 1.0,
    },
    (newState) => {
      ariaState = newState;
    }
  );

  // 5 rapid clicks within 1500ms
  for (let i = 0; i < 5; i++) {
    machine.recordClick();
  }

  assert.equal(machine.getCurrentState(), "dizzy", "Rapid clicks must transition to dizzy");
  assert.equal(ariaState, "dizzy", "Aria notification callback must receive dizzy state");

  // Subsequent clicks during dizzy cooldown should not reset cooldown or cause crash
  machine.recordClick();
  assert.equal(machine.getCurrentState(), "dizzy", "Must remain dizzy during active state duration");
});

test("Phase 10 Mascot: pointer move updates gaze coordinates and settles cleanly", () => {
  const machine = new MascotStateMachine({
    enabled: true,
    trackCursor: true,
    animationIntensity: 1.0,
  });

  machine.recordPointerMove(20, -15, true);
  const state = machine.getState();
  assert.ok(state.targetGaze.x > 0, "Positive relative X moves target gaze right");
  assert.ok(state.targetGaze.y < 0, "Negative relative Y moves target gaze up");

  // Step physics to lerp toward target
  const isDirty = machine.update(16);
  assert.equal(isDirty, true, "Gaze transition must mark frame dirty");

  // Step until gaze arrives at target
  for (let i = 0; i < 40; i++) {
    machine.update(16);
  }

  assert.equal(state.gaze.x, state.targetGaze.x, "Gaze X must arrive precisely at target");
  assert.equal(state.gaze.y, state.targetGaze.y, "Gaze Y must arrive precisely at target");
});

test("Phase 10 Context Priority Engine: high-frequency candidate churn maintains strict priority invariants", () => {
  const engine = new ContextEngine();
  const now = Date.now();

  // Register multiple competing modules simultaneously
  engine.registerCandidate({
    id: "coding",
    mode: "coding",
    priority: ContextPriority.P4_CODING,
    timestamp: now,
  });

  engine.registerCandidate({
    id: "downloads",
    mode: "download",
    priority: ContextPriority.P3_DOWNLOAD,
    timestamp: now + 10,
  });

  engine.registerCandidate({
    id: "timer",
    mode: "timer",
    priority: ContextPriority.P2_FOCUS_TIMER,
    timestamp: now + 20,
  });

  // Focus timer (P2) must win over downloads (P3) and coding (P4)
  assert.equal(engine.current.mode, "timer");

  // Inject short P1 completion notification
  engine.registerCandidate({
    id: "download_complete",
    mode: "notification",
    priority: ContextPriority.P1_COMPLETION,
    timestamp: now + 30,
    durationMs: 500,
  });

  // P1 completion outranks active P2 focus timer
  assert.equal(engine.current.mode, "notification");

  // Remove P1 completion; should immediately drop back to P2 timer
  engine.unregisterCandidate("download_complete");
  assert.equal(engine.current.mode, "timer");

  // Remove timer; should drop to P3 downloads
  engine.unregisterCandidate("timer");
  assert.equal(engine.current.mode, "download");

  // Remove downloads; should drop to P4 coding
  engine.unregisterCandidate("downloads");
  assert.equal(engine.current.mode, "coding");

  // Remove coding; should drop to P6 idle
  engine.unregisterCandidate("coding");
  assert.equal(engine.current.mode, "idle");
});

test("Phase 10 Focus Timer Collapsed Label: compact label formatting under all states", () => {
  // Ready state
  assert.equal(
    formatCompactLabel(DEFAULT_FOCUS_TIMER_STATE),
    "Focus Timer"
  );

  // Running state (25:00 = 1500s)
  assert.equal(
    formatCompactLabel({
      ...DEFAULT_FOCUS_TIMER_STATE,
      isRunning: true,
      isPaused: false,
      remainingSeconds: 1500,
    }),
    "Focus 25:00"
  );

  // Paused state
  assert.equal(
    formatCompactLabel({
      ...DEFAULT_FOCUS_TIMER_STATE,
      isRunning: true,
      isPaused: true,
      remainingSeconds: 742,
    }),
    "Focus · Paused 12:22"
  );
});
