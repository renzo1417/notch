import { test } from "node:test";
import assert from "node:assert/strict";
import { migrateSettings } from "../src/settings/migrations.ts";
import { GRAPHITE, normalizeHex, accentForeground } from "../src/styles/palette.ts";
import { ShellTransition } from "../src/shell/shell-transition.ts";
import { backState, type NotchShellState } from "../src/app/state/shell-state.ts";

test("Graphite migration changes only the legacy default accent once", () => {
  const old = { schemaVersion: 1, appearance: { accentColor: "#3B82F6", opacity: .82 }, general: { selectedMonitor: "Display 2" } };
  const migrated = migrateSettings(old);
  assert.equal(migrated.appearance.accentColor, GRAPHITE.accent);
  assert.equal(migrated.appearance.opacity, .82);
  assert.equal(migrated.general.selectedMonitor, "Display 2");
  assert.equal(migrated.clipboard.persistHistory, false);
  assert.equal(migrateSettings({ ...old, appearance: { accentColor: "#f08" } }).appearance.accentColor, "#f08");
  assert.equal(migrateSettings({ ...old, schemaVersion: 2 }).appearance.accentColor, "#3B82F6");
  assert.deepEqual(migrateSettings(migrated), migrated);
});

test("Custom accents normalize shorthand and retain readable filled controls", () => {
  assert.equal(normalizeHex("#abc"), "#aabbcc");
  assert.equal(accentForeground(GRAPHITE.accent), "#151618");
  assert.equal(accentForeground("#111"), "#ffffff");
});

test("Shell expands native bounds before showing content and shrinks after visual collapse", async () => {
  const calls: string[] = [];
  const transition = new ShellTransition({ resize: async (state) => { calls.push(`resize:${state}`); }, show: (state) => calls.push(`show:${state}`), duration: () => 0 });
  await transition.request("expanded");
  await transition.request("clipboard");
  await transition.request("compact");
  assert.deepEqual(calls, ["resize:expanded", "show:expanded", "resize:clipboard", "show:clipboard", "show:compact", "resize:compact"]);
  transition.destroy();
});

test("A newer navigation request wins while a native resize is in flight", async () => {
  let release: (() => void) | undefined;
  const shown: NotchShellState[] = [];
  const resized: NotchShellState[] = [];
  const transition = new ShellTransition({
    resize: async (state) => { resized.push(state); if (state === "expanded") await new Promise<void>((resolve) => { release = resolve; }); },
    show: (state) => { shown.push(state); }, duration: () => 0,
  });
  const first = transition.request("expanded");
  await Promise.resolve();
  const latest = transition.request("downloads");
  release?.();
  await Promise.all([first, latest]);
  assert.deepEqual(shown, ["downloads"]);
  assert.deepEqual(resized, ["expanded", "downloads"]);
  transition.destroy();
});

test("Pending collapse is canceled when a drawer is reopened", async () => {
  const resized: NotchShellState[] = [];
  const transition = new ShellTransition({ resize: async (state) => { resized.push(state); }, show: () => {}, duration: () => 10000 });
  await transition.request("clipboard");
  const collapse = transition.request("compact");
  await Promise.resolve();
  const reopen = transition.request("downloads");
  await Promise.all([collapse, reopen]);
  assert.equal(resized.includes("compact"), false);
  assert.equal(resized.at(-1), "downloads");
  transition.destroy();
});

test("Escape closes one level and legacy launcher navigation remains available", async () => {
  assert.equal(backState("clipboard"), "expanded");
  assert.equal(backState("settings"), "expanded");
  assert.equal(backState("expanded"), "compact");
  const shown: NotchShellState[] = [];
  const transition = new ShellTransition({ resize: async () => {}, show: (state) => { shown.push(state); }, duration: () => 0 });
  await transition.request("launcher");
  assert.deepEqual(shown, ["apps"]);
  transition.destroy();
});

test("Failed native expansion keeps the previous surface and can be retried", async () => {
  let fail = true;
  const shown: NotchShellState[] = [];
  const transition = new ShellTransition({
    resize: async () => { if (fail) throw new Error("Window unavailable"); },
    show: (state) => { shown.push(state); }, duration: () => 0,
  });
  await assert.rejects(transition.request("expanded"), /Window unavailable/);
  assert.deepEqual(shown, []);
  fail = false;
  await transition.request("expanded");
  assert.deepEqual(shown, ["expanded"]);
  transition.destroy();
});
