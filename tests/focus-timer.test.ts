import test from "node:test";
import assert from "node:assert/strict";
import {
  FocusTimerEngine,
  formatSeconds,
  formatCompactLabel,
} from "../src/modules/timer/timer-engine.ts";
import {
  ContextEngine,
  ContextPriority,
} from "../src/app/state/context-engine.ts";

test("Focus Timer: time formatting helpers produce clean tabular outputs", () => {
  assert.equal(formatSeconds(1500), "25:00");
  assert.equal(formatSeconds(3000), "50:00");
  assert.equal(formatSeconds(542), "09:02");
  assert.equal(formatSeconds(59), "00:59");
  assert.equal(formatSeconds(0), "00:00");
  assert.equal(formatSeconds(-10), "00:00");
});

test("Focus Timer: presets and custom duration management", () => {
  const engine = new FocusTimerEngine();

  // Initial default: 25 minutes
  assert.equal(engine.getState().presetMinutes, 25);
  assert.equal(engine.getState().remainingSeconds, 25 * 60);
  assert.equal(engine.activePreset, "25");

  // 50-minute preset
  engine.setPreset(50);
  assert.equal(engine.getState().presetMinutes, 50);
  assert.equal(engine.getState().remainingSeconds, 50 * 60);
  assert.equal(engine.activePreset, "50");

  // Switch back to 25m
  engine.setPreset(25);
  assert.equal(engine.getState().presetMinutes, 25);
  assert.equal(engine.getState().remainingSeconds, 25 * 60);
  assert.equal(engine.activePreset, "25");

  // Custom duration
  engine.setCustomDuration(42);
  assert.equal(engine.getState().presetMinutes, 42);
  assert.equal(engine.getState().customMinutes, 42);
  assert.equal(engine.getState().remainingSeconds, 42 * 60);
  assert.equal(engine.activePreset, "custom");

  // Clamping custom duration: < 1 clamps to 1
  engine.setCustomDuration(0);
  assert.equal(engine.getState().presetMinutes, 1);

  // Clamping custom duration: > 180 clamps to 180
  engine.setCustomDuration(250);
  assert.equal(engine.getState().presetMinutes, 180);

  engine.destroy();
});

test("Focus Timer: start, pause, resume, reset lifecycle with drift-free timestamps", () => {
  const customContextEngine = new ContextEngine();
  let mascotState = "idle";

  const engine = new FocusTimerEngine({
    contextEngine: customContextEngine,
    callbacks: {
      onMascotFocused: () => {
        mascotState = "focused";
      },
      onMascotReset: () => {
        mascotState = "idle";
      },
    },
  });

  const startTime = 1_000_000;
  engine.setPreset(25);
  engine.start(25, startTime);

  // State after start
  const state1 = engine.getState();
  assert.equal(state1.isRunning, true);
  assert.equal(state1.isPaused, false);
  assert.equal(state1.startTimestampMs, startTime);
  assert.equal(state1.endTimestampMs, startTime + 25 * 60 * 1000);
  assert.equal(state1.remainingSeconds, 1500);
  assert.equal(mascotState, "focused");
  assert.equal(customContextEngine.currentMode, "timer");

  // Advance time by 45 seconds (45,000 ms)
  const timeAfter45s = startTime + 45_000;
  engine.tick(timeAfter45s);
  assert.equal(engine.getState().remainingSeconds, 1500 - 45); // 1455 seconds
  assert.equal(engine.formattedTime, "24:15");
  assert.equal(engine.formattedCompact, "Focus 24:15");

  // Pause at timeAfter45s
  engine.pause(timeAfter45s);
  const statePaused = engine.getState();
  assert.equal(statePaused.isRunning, true);
  assert.equal(statePaused.isPaused, true);
  assert.equal(statePaused.remainingSeconds, 1455);
  assert.equal(statePaused.endTimestampMs, null);
  assert.equal(engine.formattedCompact, "Focus · Paused 24:15");

  // Ticking while paused must not change remainingSeconds
  engine.tick(timeAfter45s + 30_000);
  assert.equal(engine.getState().remainingSeconds, 1455);

  // Resume 10 seconds later
  const resumeTime = timeAfter45s + 10_000;
  engine.resume(resumeTime);
  const stateResumed = engine.getState();
  assert.equal(stateResumed.isRunning, true);
  assert.equal(stateResumed.isPaused, false);
  assert.equal(stateResumed.endTimestampMs, resumeTime + 1455 * 1000);
  assert.equal(mascotState, "focused");

  // Advance time by another 55 seconds
  engine.tick(resumeTime + 55_000);
  assert.equal(engine.getState().remainingSeconds, 1400); // 23:20
  assert.equal(engine.formattedTime, "23:20");

  // Reset / Stop
  engine.reset();
  const stateReset = engine.getState();
  assert.equal(stateReset.isRunning, false);
  assert.equal(stateReset.isPaused, false);
  assert.equal(stateReset.remainingSeconds, 1500);
  assert.equal(stateReset.endTimestampMs, null);
  assert.equal(mascotState, "idle");
  assert.equal(customContextEngine.currentMode, "idle");

  engine.destroy();
  customContextEngine.destroy();
});

test("Focus Timer: timer strictly outranks coding session in Context Engine (P2 > P4)", () => {
  const contextEngine = new ContextEngine();

  // Active coding session registered at P4_CODING
  contextEngine.registerCandidate({
    id: "coding",
    mode: "coding",
    priority: ContextPriority.P4_CODING,
    timestamp: 100,
    data: { appName: "VS Code" },
  });

  // Verify coding context is active
  assert.equal(contextEngine.currentMode, "coding");

  // Initialize and start Focus Timer
  const engine = new FocusTimerEngine({ contextEngine });
  engine.start(25, 200);

  // Timer (P2_FOCUS_TIMER) strictly takes over the center surface!
  assert.equal(contextEngine.currentMode, "timer");
  assert.equal(contextEngine.current.priority, ContextPriority.P2_FOCUS_TIMER);

  // When timer is stopped, center surface smoothly falls back to coding session
  engine.stop();
  assert.equal(contextEngine.currentMode, "coding");

  // Unregister coding session returns to idle
  contextEngine.unregisterCandidate("coding");
  assert.equal(contextEngine.currentMode, "idle");

  engine.destroy();
  contextEngine.destroy();
});

test("Focus Timer: completion triggers celebration, internal notification, and context return", () => {
  const contextEngine = new ContextEngine();
  let celebrated = false;
  let notificationTitle = "";
  let notificationMessage = "";

  const engine = new FocusTimerEngine({
    contextEngine,
    callbacks: {
      onMascotCelebrate: () => {
        celebrated = true;
      },
      onNotification: (title, message, durationMs) => {
        notificationTitle = title;
        notificationMessage = message;
        // Registers completion notification in contextEngine at P1
        contextEngine.pushTemporaryContext({
          id: "timer-completion",
          mode: "notification",
          priority: ContextPriority.P1_COMPLETION,
          durationMs,
          data: { title, message },
        });
      },
    },
  });

  // Background active coding session
  contextEngine.registerCandidate({
    id: "coding",
    mode: "coding",
    priority: ContextPriority.P4_CODING,
    timestamp: 100,
  });

  const startNow = 1_000_000;
  engine.start(25, startNow);
  assert.equal(contextEngine.currentMode, "timer");

  // Advance time to completion (exactly 1500 seconds later)
  const completionTime = startNow + 1500 * 1000;
  engine.tick(completionTime);

  // Timer is finished
  assert.equal(engine.getState().isRunning, false);
  assert.equal(engine.getState().remainingSeconds, 1500); // Ready for next session
  assert.equal(celebrated, true);
  assert.equal(notificationTitle, "Focus Complete");
  assert.equal(notificationMessage, "25m finished · Great focus session!");

  // Completion notification takes precedence over background coding at P1_COMPLETION
  assert.equal(contextEngine.currentMode, "notification");
  assert.equal(contextEngine.current.priority, ContextPriority.P1_COMPLETION);

  // Dismiss notification -> reveals previous coding session!
  contextEngine.dismissTemporaryContext("timer-completion");
  assert.equal(contextEngine.currentMode, "coding");

  engine.destroy();
  contextEngine.destroy();
});
