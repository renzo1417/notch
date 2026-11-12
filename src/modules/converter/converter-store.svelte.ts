import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { settingsStore } from "../../settings/settings-store.svelte";
import { notificationStore } from "../notifications/notification-store.svelte";
import type { ConversionFile, ConversionInspection, ConversionSnapshot, ConverterAvailability } from "./types";

class ConverterStore {
  availability = $state<ConverterAvailability>({ status: "", version: null, message: "" });
  snapshot = $state<ConversionSnapshot>({ jobId: "", running: false, files: [], target: "", message: "" });
  files = $state<ConversionFile[]>([]);
  targets = $state<string[]>([]);
  unavailableTargets = $state<string[]>([]);
  target = $state("");
  outputFolder = $state<string | null>(null);
  loading = $state(false);
  starting = $state(false);
  cancelling = $state(false);
  private picking = $state(false);
  error = $state("");
  recent = $state<ConversionFile[]>([]);
  private pendingPaths: string[] = [];
  private unlisten: UnlistenFn | null = null;
  private initialization: Promise<void> | null = null;
  private disposed = false;
  private revision = 0;
  get busy() { return this.starting || this.snapshot.running || this.picking; }
  get native() { return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window; }

  init(): Promise<void> {
    if (this.initialization) return this.initialization;
    this.disposed = false;
    this.initialization = (async () => {
      if (!this.native) return;
      const unlisten = await listen<ConversionSnapshot>("converter.changed", event => {
        this.revision++;
        this.snapshot = event.payload;
      });
      if (this.disposed) { unlisten(); return; }
      this.unlisten = unlisten;
      const revision = this.revision;
      const snapshot = await invoke<ConversionSnapshot>("get_conversion_state");
      if (!this.disposed && revision === this.revision) this.snapshot = snapshot;
    })().catch(error => { this.error = String(error); this.initialization = null; });
    return this.initialization;
  }

  async check(refresh = false): Promise<void> {
    if (this.loading || this.busy) return;
    this.loading = true; this.error = "";
    try {
      await this.init();
      if (!this.native) {
        this.availability = { status: "missing", version: null, message: "File conversion is available in the Windows desktop app." };
        return;
      }
      this.availability = await invoke<ConverterAvailability>("get_converter_status", { refresh });
      const paths = this.pendingPaths.length ? this.pendingPaths : this.files.map(file => file.path);
      if (this.availability.status === "ready" && paths.length) await this.inspect(paths);
    } catch (error) { this.error = String(error); }
    finally { this.loading = false; }
  }

  private async inspect(paths: string[]): Promise<void> {
    const result = await invoke<ConversionInspection>("inspect_conversion_files", { paths });
    this.files = result.files; this.targets = result.targets; this.unavailableTargets = result.unavailableTargets;
    if (!this.targets.includes(this.target)) this.target = this.targets[0] ?? "";
    this.pendingPaths = [];
  }

  async prepare(paths: string[]): Promise<void> {
    if (this.busy || this.loading) { this.error = "A conversion is already in progress. Wait for it to finish."; return; }
    if (!paths.length) return;
    this.archive();
    this.files = []; this.targets = []; this.target = ""; this.unavailableTargets = [];
    this.pendingPaths = paths.slice(0, 21);
    await this.check();
  }

  async remove(path: string): Promise<void> {
    if (this.busy || this.loading) return;
    const paths = this.files.filter(file => file.path !== path).map(file => file.path);
    if (!paths.length) { this.clear(); return; }
    await this.prepare(paths);
  }

  async pickFiles(): Promise<void> {
    await this.perform(async () => {
      if (!this.native) { await this.check(); return; }
      const paths = await this.pick("files");
      if (paths.length) await this.prepare(paths);
    });
  }

  async locate(): Promise<void> {
    if (this.busy || this.loading) return;
    await this.perform(async () => {
      if (!this.native) return;
      const [cliPath] = await this.pick("executable");
      if (!cliPath) return;
      await settingsStore.update("converter", { cliPath });
      // The normal settings save is debounced; persist before probing the new path.
      await invoke("save_settings", { settings: settingsStore.state });
      await this.check(true);
    });
  }

  async chooseFolder(): Promise<void> {
    await this.perform(async () => {
      if (!this.native || this.busy) return;
      const [folder] = await this.pick("folder");
      if (folder) this.outputFolder = folder;
    });
  }

  private async pick(kind: "files" | "folder" | "executable"): Promise<string[]> {
    if (this.busy || this.loading) return [];
    this.picking = true;
    try { return await invoke<string[]>("pick_conversion_path", { kind }); }
    finally { this.picking = false; }
  }

  async useAutomatic(): Promise<void> {
    if (this.busy || this.loading) return;
    await this.perform(async () => {
      await settingsStore.update("converter", { cliPath: null });
      if (this.native) await invoke("save_settings", { settings: settingsStore.state });
      await this.check(true);
    });
  }

  async start(): Promise<void> {
    if (this.busy || this.loading || !this.files.length || !this.targets.includes(this.target)) return;
    this.starting = true; this.error = "";
    try {
      await this.init();
      this.snapshot = await invoke<ConversionSnapshot>("start_conversion", { paths: this.files.map(file => file.path), target: this.target, outputFolder: this.outputFolder });
      notificationStore.notify({ title: "File conversion", message: this.snapshot.message });
    } catch (error) { this.error = String(error); }
    finally { this.starting = false; this.cancelling = false; }
  }

  async cancel(): Promise<void> {
    if (!this.snapshot.running || this.cancelling) return;
    this.cancelling = true;
    await this.perform(async () => invoke("cancel_conversion", { jobId: this.snapshot.jobId }));
    if (this.error) this.cancelling = false;
  }

  async retry(): Promise<void> {
    const paths = this.snapshot.files.filter(file => file.state === "failed" || file.state === "cancelled").map(file => file.path);
    const target = this.snapshot.target;
    if (!paths.length) return;
    await this.prepare(paths);
    if (this.targets.includes(target)) { this.target = target; await this.start(); }
  }

  private archive(): void {
    this.recent = [...this.snapshot.files.filter(file => file.state === "done"), ...this.recent].slice(0, 10);
    this.snapshot = { jobId: "", running: false, files: [], target: "", message: "" };
  }

  clear(): void {
    if (this.busy) return;
    this.archive(); this.files = []; this.pendingPaths = []; this.targets = []; this.unavailableTargets = []; this.error = "";
  }

  async open(file: ConversionFile, folder = false): Promise<void> {
    await this.perform(async () => {
      if (!file.outputs[0]) return;
      await invoke("open_conversion_result", { path: file.outputs[0], folder });
    });
  }

  async install(): Promise<void> { await this.perform(async () => openUrl("https://github.com/KnifeLemon/Filee/releases")); }
  private async perform(action: () => Promise<unknown>): Promise<void> {
    this.error = "";
    try { await action(); } catch (error) { this.error = String(error); }
  }

  destroy(): void { this.disposed = true; this.unlisten?.(); this.unlisten = null; this.initialization = null; }
}

export const converterStore = new ConverterStore();
