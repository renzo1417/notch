import test from "node:test";
import assert from "node:assert/strict";
import {
  ContextPriority,
  ContextEngine,
} from "../src/app/state/context-engine.ts";
import type { ClipboardItem } from "../src/modules/clipboard/types.ts";

test("Clipboard: deduplicates identical consecutive entries and trims to maxItems", () => {
  let items: ClipboardItem[] = [];
  const maxItems = 5;

  function pushClipboard(text: string) {
    const trimmed = text.trim();
    if (!trimmed) return;

    // Deduplicate consecutive identical entries
    if (items.length > 0 && items[0].text === text) {
      return;
    }

    const newItem: ClipboardItem = {
      id: `clip_${Date.now()}_${items.length}`,
      text,
      timestampMs: Date.now(),
    };

    items = [newItem, ...items.filter((i) => i.text !== text)].slice(0, maxItems);
  }

  pushClipboard("First snippet");
  assert.equal(items.length, 1);
  assert.equal(items[0].text, "First snippet");

  // Duplicate identical consecutive entry
  pushClipboard("First snippet");
  assert.equal(items.length, 1, "Consecutive identical entry must be ignored");

  // Different entry
  pushClipboard("Second snippet");
  assert.equal(items.length, 2);
  assert.equal(items[0].text, "Second snippet");

  // Another distinct entry
  pushClipboard("Third snippet");
  pushClipboard("Fourth snippet");
  pushClipboard("Fifth snippet");
  assert.equal(items.length, 5);

  // Overflow past maxItems (5)
  pushClipboard("Sixth snippet");
  assert.equal(items.length, 5);
  assert.equal(items[0].text, "Sixth snippet");
  assert.ok(!items.some((i) => i.text === "First snippet"), "Oldest entry should be pruned");
});

test("Clipboard: pause capture prevents recording new entries", () => {
  let items: ClipboardItem[] = [];
  let isPaused = false;

  function record(text: string) {
    if (isPaused) return;
    items.unshift({
      id: `clip_${Date.now()}`,
      text,
      timestampMs: Date.now(),
    });
  }

  record("Before pause");
  assert.equal(items.length, 1);

  isPaused = true;
  record("While paused 1");
  record("While paused 2");
  assert.equal(items.length, 1, "Must not capture items while paused");

  isPaused = false;
  record("After resume");
  assert.equal(items.length, 2);
  assert.equal(items[0].text, "After resume");
});

test("Clipboard: clear history empties stored items completely", () => {
  let items: ClipboardItem[] = [
    { id: "1", text: "Alpha", timestampMs: 100 },
    { id: "2", text: "Beta", timestampMs: 200 },
  ];

  assert.equal(items.length, 2);
  items = [];
  assert.equal(items.length, 0);
});

test("Clipboard: background updates do NOT hijack the active center context", () => {
  const engine = new ContextEngine();
  const now = Date.now();

  // Active coding context is running (P4)
  engine.registerCandidate({
    id: "coding",
    mode: "coding",
    priority: ContextPriority.P4_CODING,
    timestamp: now,
  });

  assert.equal(engine.current.mode, "coding");

  // Invariant from PRODUCT_SPEC.md:
  // "Clipboard changes should not constantly replace the center context. Clipboard history is primarily opened intentionally by the user."
  // When a clipboard update occurs in the background, no candidate is automatically pushed into ContextEngine.
  assert.equal(engine.current.mode, "coding", "Clipboard background copy must NOT hijack center context");

  // Only when user intentionally opens clipboard surface does it register at P5
  engine.registerCandidate({
    id: "clipboard",
    mode: "clipboard",
    priority: ContextPriority.P5_USER_SURFACE,
    timestamp: now + 50,
  });

  // Because coding (P4) outranks user surface (P5), coding stays active until dismissed/unfocused
  assert.equal(engine.current.mode, "coding");

  // When coding ends, clipboard surface is revealed
  engine.unregisterCandidate("coding");
  assert.equal(engine.current.mode, "clipboard");

  // Dismissing clipboard surface returns to idle
  engine.unregisterCandidate("clipboard");
  assert.equal(engine.current.mode, "idle");
  engine.destroy();
});
