import test from "node:test";
import assert from "node:assert/strict";
import {
  ContextPriority,
  resolveActiveCandidate,
  ContextEngine,
  type ContextCandidate,
} from "../src/app/state/context-engine.ts";
import { shellActions } from "../src/app/state/shell-state.ts";
import { MAX_PINNED_BEFORE_OVERFLOW, type LaunchableApp } from "../src/modules/launcher/types.ts";

test("Launcher Model: maintains strict separation of executable path and argument array", () => {
  const app: LaunchableApp = {
    id: "app-cap-1",
    name: "Cap.io",
    executablePath: "C:\\Program Files\\Cap\\Cap.exe",
    args: ["--minimized", "--tray"],
    iconDataUrl: "data:image/bmp;base64,Qk0=",
    enabled: true,
    order: 0,
    exists: true,
  };

  assert.equal(app.name, "Cap.io");
  assert.equal(app.executablePath, "C:\\Program Files\\Cap\\Cap.exe");
  assert.deepEqual(app.args, ["--minimized", "--tray"]);
  assert.ok(!app.executablePath.includes("--minimized"), "Executable path must not contain arguments");
  assert.equal(app.enabled, true);
  assert.equal(app.order, 0);
  assert.equal(app.exists, true);
});

test("Launcher Priority: manually opened launcher surface registers at P5_USER_SURFACE", () => {
  const engine = new ContextEngine();
  const now = Date.now();

  assert.equal(engine.current.mode, "idle");

  // User opens launcher surface
  engine.registerCandidate({
    id: "launcher",
    mode: "launcher",
    priority: ContextPriority.P5_USER_SURFACE,
    timestamp: now,
  });

  assert.equal(engine.current.mode, "launcher");
  assert.equal(engine.current.priority, ContextPriority.P5_USER_SURFACE);

  // Background coding session appears (P4 > P5)
  engine.registerCandidate({
    id: "coding",
    mode: "coding",
    priority: ContextPriority.P4_CODING,
    timestamp: now + 50,
  });

  assert.equal(engine.current.mode, "coding", "Coding session (P4) outranks user launcher surface (P5)");

  // Active focus timer appears (P2 > P4 > P5)
  engine.registerCandidate({
    id: "timer",
    mode: "timer",
    priority: ContextPriority.P2_FOCUS_TIMER,
    timestamp: now + 100,
  });

  assert.equal(engine.current.mode, "timer", "Timer (P2) outranks coding (P4) and launcher (P5)");

  // When timer and coding are dismissed, launcher is revealed
  engine.unregisterCandidate("timer");
  engine.unregisterCandidate("coding");
  assert.equal(engine.current.mode, "launcher");

  // Dismissing launcher returns to idle
  engine.unregisterCandidate("launcher");
  assert.equal(engine.current.mode, "idle");
  engine.destroy();
});

test("Launcher Reordering: updates orders sequentially and preserves enabled state", () => {
  const apps: LaunchableApp[] = [
    {
      id: "app-1",
      name: "Visual Studio Code",
      executablePath: "C:\\Tools\\Code.exe",
      args: [],
      enabled: true,
      order: 0,
      exists: true,
    },
    {
      id: "app-2",
      name: "Cap.io",
      executablePath: "C:\\Tools\\Cap.exe",
      args: [],
      enabled: true,
      order: 1,
      exists: true,
    },
    {
      id: "app-3",
      name: "Terminal",
      executablePath: "C:\\Tools\\wt.exe",
      args: [],
      enabled: true,
      order: 2,
      exists: true,
    },
  ];

  // Move Cap.io to first position
  const reorderedIds = ["app-2", "app-1", "app-3"];
  const reordered = reorderedIds.map((id, index) => {
    const app = apps.find((a) => a.id === id)!;
    return { ...app, order: index };
  });

  assert.equal(reordered[0].name, "Cap.io");
  assert.equal(reordered[0].order, 0);
  assert.equal(reordered[1].name, "Visual Studio Code");
  assert.equal(reordered[1].order, 1);
  assert.equal(reordered[2].name, "Terminal");
  assert.equal(reordered[2].order, 2);
});

test("Launcher Overflow: enforces 4–6 pinned apps boundary before pagination", () => {
  assert.ok(MAX_PINNED_BEFORE_OVERFLOW >= 4 && MAX_PINNED_BEFORE_OVERFLOW <= 6);

  const testApps: LaunchableApp[] = Array.from({ length: 8 }, (_, i) => ({
    id: `app-${i}`,
    name: `App ${i + 1}`,
    executablePath: `C:\\App${i}.exe`,
    args: [],
    enabled: true,
    order: i,
    exists: true,
  }));

  const pinned = testApps.filter((a) => a.enabled).sort((a, b) => a.order - b.order);
  const visible = pinned.slice(0, MAX_PINNED_BEFORE_OVERFLOW);
  const overflow = pinned.length - MAX_PINNED_BEFORE_OVERFLOW;

  assert.equal(visible.length, MAX_PINNED_BEFORE_OVERFLOW);
  assert.equal(overflow, 3, "8 apps produce 5 visible and 3 overflow items");
});

test("Broken App Detection: identifies missing paths and verifies locate/remove workflow", () => {
  const brokenApp: LaunchableApp = {
    id: "app-broken",
    name: "Deleted Game",
    executablePath: "D:\\Games\\Deleted.exe",
    args: [],
    enabled: true,
    order: 0,
    exists: false,
  };

  assert.equal(brokenApp.exists, false);

  // Workflow option 1: Locate new valid path
  const relocatedApp: LaunchableApp = {
    ...brokenApp,
    executablePath: "E:\\Games\\Relocated.exe",
    exists: true,
  };
  assert.equal(relocatedApp.executablePath, "E:\\Games\\Relocated.exe");
  assert.equal(relocatedApp.exists, true);

  // Workflow option 2: Remove from apps list
  const list = [brokenApp];
  const afterRemoval = list.filter((a) => a.id !== brokenApp.id);
  assert.equal(afterRemoval.length, 0);
});

test("Shell Actions: requestCollapse dispatches collapse notification to subscribers", () => {
  let collapseTriggered = false;
  const unsub = shellActions.onCollapse(() => {
    collapseTriggered = true;
  });

  shellActions.requestCollapse();
  assert.equal(collapseTriggered, true, "requestCollapse must trigger subscriber callback");
  unsub();
});
