import { test } from "node:test";
import assert from "node:assert/strict";
import { captureAction, defaultCaptureMonitor } from "../src/modules/capture/capture-model.ts";
import { backState, SHELL_STATE_DIMENSIONS, shellActions } from "../src/app/state/shell-state.ts";

test("capture follows the configured monitor, then primary, then the first available display", () => {
  const monitors = [
    { id: "secondary", name: "Display 2", primary: false, width: 1920, height: 1080 },
    { id: "primary", name: "Display 1", primary: true, width: 2560, height: 1440 },
  ];
  assert.equal(defaultCaptureMonitor(monitors, "Display 2"), "secondary");
  assert.equal(defaultCaptureMonitor(monitors, "Disconnected"), "primary");
  assert.equal(defaultCaptureMonitor(monitors.slice(0, 1), null), "secondary");
  assert.equal(defaultCaptureMonitor([], null), "");
});

test("capture actions describe the actual selection step", () => {
  assert.equal(captureAction("area"), "Select area");
  assert.equal(captureAction("window"), "Capture window");
  assert.equal(captureAction("screen"), "Capture screen");
});

test("nested utilities and the installed-app picker close one level at a time", () => {
  for (const state of ["clipboard", "downloads", "battery", "settings"] as const) {
    assert.equal(backState(state), "utilities");
    assert.equal(backState(backState(state)), "expanded");
  }
  assert.equal(backState("appPicker", "settings"), "settings");
  assert.equal(backState("appPicker", "apps"), "apps");
  assert.equal(backState("capture"), "expanded");
  assert.equal(backState("expanded"), "compact");
  assert.deepEqual(SHELL_STATE_DIMENSIONS.capture, SHELL_STATE_DIMENSIONS.appPicker);
  assert.equal(SHELL_STATE_DIMENSIONS.capture.windowHeight, 344);
});

test("app picker requests retain repair identity and unsubscribe cleanly", () => {
  const requests: (string | undefined)[] = [];
  const unsubscribe = shellActions.onAppPicker((replaceId) => requests.push(replaceId));
  shellActions.requestAppPicker();
  shellActions.requestAppPicker("broken-pin");
  unsubscribe();
  shellActions.requestAppPicker("ignored");
  assert.deepEqual(requests, [undefined, "broken-pin"]);
});
