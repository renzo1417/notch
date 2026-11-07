import test from "node:test";
import assert from "node:assert/strict";
import { launchIdentity, searchInstalledApps, type InstalledApp } from "../src/modules/launcher/types.ts";

test("Legacy launcher targets retain quoted argument values as complete array entries", () => {
  const legacy = { executablePath: "C:\\Tools\\Editor.exe", args: ["--profile", "Work profile"] };
  assert.equal(launchIdentity(legacy), launchIdentity({ target: { kind: "executable", path: "c:/tools/EDITOR.exe", args: ["--profile", "Work profile"] } }));
  assert.notEqual(launchIdentity(legacy), launchIdentity({ ...legacy, args: ["--profile", "Personal"] }));
});

test("Shortcut and packaged identities do not depend on display names or protected executable paths", () => {
  assert.equal(launchIdentity({ executablePath: "C:\\Menu\\Browser.lnk", args: [] }), launchIdentity({ target: { kind: "shortcut", path: "c:/MENU/browser.lnk" } }));
  assert.notEqual(launchIdentity({ target: { kind: "shortcut", path: "C:\\Menu\\Work.lnk" } }), launchIdentity({ target: { kind: "shortcut", path: "C:\\Menu\\Personal.lnk" } }));
  assert.equal(launchIdentity({ target: { kind: "packaged", aumid: "Example.Package!App" } }), "app:example.package!app");
});

test("Installed-app search supports multiple terms, case-insensitive names, and empty searches", () => {
  const apps: InstalledApp[] = [
    { id: "one", name: "Windows Calculator", source: "Windows app", target: { kind: "packaged", aumid: "Calculator!App" } },
    { id: "two", name: "Browser Work", source: "Start Menu", target: { kind: "shortcut", path: "C:\\Menu\\Browser Work.lnk" } },
    { id: "three", name: "Browser Personal", source: "Start Menu", target: { kind: "shortcut", path: "C:\\Menu\\Browser Personal.lnk" } },
  ];
  assert.deepEqual(searchInstalledApps(apps, "  WORK browser ").map((app) => app.id), ["two"]);
  assert.deepEqual(searchInstalledApps(apps, "CALC windows").map((app) => app.id), ["one"]);
  assert.deepEqual(searchInstalledApps(apps, " "), apps);
  assert.deepEqual(searchInstalledApps(apps, "missing"), []);
  assert.equal(apps.length, 3);
});
