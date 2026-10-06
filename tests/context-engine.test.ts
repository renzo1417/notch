import test from "node:test";
import assert from "node:assert/strict";
import {
  ContextPriority,
  resolveActiveCandidate,
  resolveActiveContext,
  ContextEngine,
  type ContextCandidate,
} from "../src/app/state/context-engine.ts";

test("Context Engine: returns idle when candidate list is empty", () => {
  const result = resolveActiveCandidate([]);
  assert.equal(result.mode, "idle");
  assert.equal(result.priority, ContextPriority.P6_IDLE);

  const mode = resolveActiveContext([]);
  assert.equal(mode, "idle");
});

test("Context Engine: explicit priority order strictly holds (P0 > P1 > P2 > P3 > P4 > P5 > P6)", () => {
  const now = 1000;

  const idle: ContextCandidate = {
    id: "idle",
    mode: "idle",
    priority: ContextPriority.P6_IDLE,
    timestamp: now,
  };

  const launcher: ContextCandidate = {
    id: "launcher",
    mode: "launcher",
    priority: ContextPriority.P5_USER_SURFACE,
    timestamp: now,
  };

  const coding: ContextCandidate = {
    id: "coding",
    mode: "coding",
    priority: ContextPriority.P4_CODING,
    timestamp: now,
  };

  const download: ContextCandidate = {
    id: "download",
    mode: "download",
    priority: ContextPriority.P3_DOWNLOAD,
    timestamp: now,
  };

  const timer: ContextCandidate = {
    id: "timer",
    mode: "timer",
    priority: ContextPriority.P2_FOCUS_TIMER,
    timestamp: now,
  };

  const notification: ContextCandidate = {
    id: "notification",
    mode: "notification",
    priority: ContextPriority.P1_COMPLETION,
    timestamp: now,
  };

  const battery: ContextCandidate = {
    id: "battery",
    mode: "battery",
    priority: ContextPriority.P0_SYSTEM_WARNING,
    timestamp: now,
  };

  // Test pairwise precedence
  assert.equal(resolveActiveContext([idle, launcher], now), "launcher");
  assert.equal(resolveActiveContext([launcher, coding], now), "coding");
  assert.equal(resolveActiveContext([coding, download], now), "download");
  assert.equal(resolveActiveContext([download, timer], now), "timer");
  assert.equal(resolveActiveContext([timer, notification], now), "notification");
  assert.equal(resolveActiveContext([notification, battery], now), "battery");

  // All candidates simultaneously: P0 battery warning wins
  const all = [idle, launcher, coding, download, timer, notification, battery];
  assert.equal(resolveActiveContext(all, now), "battery");

  // Coding outranks idle and launcher
  assert.equal(resolveActiveContext([idle, launcher, coding], now), "coding");

  // Timer outranks coding
  assert.equal(resolveActiveContext([coding, timer], now), "timer");
});

test("Context Engine: ties resolve to most recent timestamp", () => {
  const older: ContextCandidate = {
    id: "coding-1",
    mode: "coding",
    priority: ContextPriority.P4_CODING,
    timestamp: 100,
  };

  const newer: ContextCandidate = {
    id: "coding-2",
    mode: "coding",
    priority: ContextPriority.P4_CODING,
    timestamp: 200,
  };

  assert.equal(resolveActiveCandidate([older, newer]).id, "coding-2");
  assert.equal(resolveActiveCandidate([newer, older]).id, "coding-2");
});

test("Context Engine: expired candidates are pruned deterministically", () => {
  const now = 500;

  const expiredNotification: ContextCandidate = {
    id: "toast-expired",
    mode: "notification",
    priority: ContextPriority.P1_COMPLETION,
    timestamp: 100,
    expiresAt: 400, // already expired at now=500
  };

  const activeCoding: ContextCandidate = {
    id: "coding",
    mode: "coding",
    priority: ContextPriority.P4_CODING,
    timestamp: 200,
  };

  const winning = resolveActiveCandidate([expiredNotification, activeCoding], now);
  assert.equal(winning.id, "coding");
  assert.equal(winning.mode, "coding");
});

test("Context Engine: temporary context expires and reveals previous meaningful state", async () => {
  const engine = new ContextEngine();

  // Register stateful coding candidate
  engine.registerCandidate({
    id: "coding",
    mode: "coding",
    priority: ContextPriority.P4_CODING,
    timestamp: Date.now(),
  });

  assert.equal(engine.currentMode, "coding");

  // Push short-lived temporary notification (50ms)
  engine.pushTemporaryContext({
    id: "toast",
    mode: "notification",
    priority: ContextPriority.P1_COMPLETION,
    durationMs: 50,
  });

  // Notification immediately takes over
  assert.equal(engine.currentMode, "notification");

  // Wait for temporary context to expire
  await new Promise((resolve) => setTimeout(resolve, 80));

  // Cleans up cleanly and reveals previous coding state!
  assert.equal(engine.currentMode, "coding");

  // When coding unregisters, reveals idle
  engine.unregisterCandidate("coding");
  assert.equal(engine.currentMode, "idle");

  engine.destroy();
});
