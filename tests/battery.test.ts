import test from "node:test";
import assert from "node:assert/strict";
import {
  ContextPriority,
  ContextEngine,
} from "../src/app/state/context-engine.ts";
import type { BatteryState } from "../src/modules/battery/types.ts";

function formatRemainingTime(state: BatteryState): string | null {
  if (!state.hasBattery || state.isAcConnected || !state.remainingSeconds) {
    return null;
  }
  const secs = state.remainingSeconds;
  if (secs <= 0 || secs >= 86400) {
    return null;
  }
  const hours = Math.floor(secs / 3600);
  const mins = Math.floor((secs % 3600) / 60);
  if (hours > 0) {
    return `${hours}h ${mins}m`;
  }
  return `${mins}m`;
}

test("Battery: remaining time formatting handles valid seconds and never fabricates unknown values", () => {
  // Scenario 1: Windows provides valid remaining time on battery (3h 42m = 13320s)
  const laptopOnBattery: BatteryState = {
    hasBattery: true,
    percentage: 72,
    isCharging: false,
    isAcConnected: false,
    remainingSeconds: 13320,
  };
  assert.equal(formatRemainingTime(laptopOnBattery), "3h 42m");

  // Scenario 2: Minutes only (45m = 2700s)
  const shortBattery: BatteryState = {
    hasBattery: true,
    percentage: 25,
    isCharging: false,
    isAcConnected: false,
    remainingSeconds: 2700,
  };
  assert.equal(formatRemainingTime(shortBattery), "45m");

  // Scenario 3: Plugged into AC power (remainingSeconds must not be displayed)
  const pluggedIn: BatteryState = {
    hasBattery: true,
    percentage: 80,
    isCharging: true,
    isAcConnected: true,
    remainingSeconds: 3600, // Even if driver reports something, AC power means no remaining drain time
  };
  assert.equal(formatRemainingTime(pluggedIn), null);

  // Scenario 4: Unknown / invalid sentinel value from Windows (e.g. undefined or 0)
  const unknownTime: BatteryState = {
    hasBattery: true,
    percentage: 60,
    isCharging: false,
    isAcConnected: false,
    remainingSeconds: undefined,
  };
  assert.equal(formatRemainingTime(unknownTime), null, "Must never fabricate time remaining when undefined");

  // Scenario 5: Desktop system without battery
  const desktop: BatteryState = {
    hasBattery: false,
    percentage: 100,
    isCharging: false,
    isAcConnected: true,
    remainingSeconds: undefined,
  };
  assert.equal(formatRemainingTime(desktop), null);
});

test("Battery: conservative low-battery alert triggers once and latches until recharged", () => {
  let notificationsTriggered = 0;
  let hasNotified = false;

  function evaluate(state: BatteryState) {
    const isLow = state.hasBattery && !state.isAcConnected && !state.isCharging && state.percentage <= 20;

    if (isLow && !hasNotified) {
      hasNotified = true;
      notificationsTriggered += 1;
    } else if (!isLow && (state.isAcConnected || state.isCharging || state.percentage > 25)) {
      hasNotified = false;
    }
  }

  // 1. Normal battery at 40%
  evaluate({
    hasBattery: true,
    percentage: 40,
    isCharging: false,
    isAcConnected: false,
  });
  assert.equal(notificationsTriggered, 0);

  // 2. Drops to 20% (low threshold) -> triggers 1 notification
  evaluate({
    hasBattery: true,
    percentage: 20,
    isCharging: false,
    isAcConnected: false,
  });
  assert.equal(notificationsTriggered, 1);

  // 3. Next tick drops to 19% -> does NOT repeat notification
  evaluate({
    hasBattery: true,
    percentage: 19,
    isCharging: false,
    isAcConnected: false,
  });
  assert.equal(notificationsTriggered, 1, "Must not spam notifications on subsequent ticks");

  // 4. Charger is plugged in -> resets latch
  evaluate({
    hasBattery: true,
    percentage: 20,
    isCharging: true,
    isAcConnected: true,
  });
  assert.equal(notificationsTriggered, 1);
  assert.equal(hasNotified, false);

  // 5. Unplugged again at 18% -> triggers conservative alert again
  evaluate({
    hasBattery: true,
    percentage: 18,
    isCharging: false,
    isAcConnected: false,
  });
  assert.equal(notificationsTriggered, 2);
});

test("Battery: Context Engine integration registers surface at P5_USER_SURFACE", () => {
  const engine = new ContextEngine();
  const now = Date.now();

  assert.equal(engine.current.mode, "idle");

  // User manually opens battery surface
  engine.registerCandidate({
    id: "battery",
    mode: "battery",
    priority: ContextPriority.P5_USER_SURFACE,
    timestamp: now,
  });

  assert.equal(engine.current.mode, "battery");
  assert.equal(engine.current.priority, ContextPriority.P5_USER_SURFACE);

  // Focus timer outranks battery surface (P2 > P5)
  engine.registerCandidate({
    id: "timer",
    mode: "timer",
    priority: ContextPriority.P2_FOCUS_TIMER,
    timestamp: now + 50,
  });

  assert.equal(engine.current.mode, "timer");

  engine.unregisterCandidate("timer");
  assert.equal(engine.current.mode, "battery");

  engine.unregisterCandidate("battery");
  assert.equal(engine.current.mode, "idle");
  engine.destroy();
});
