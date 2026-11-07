import { invoke } from "@tauri-apps/api/core";
import type { CaptureMode, CaptureResult, CaptureTargets } from "./types";

class CaptureStore {
  targets = $state<CaptureTargets>({ supported: false, monitors: [], windows: [] });
  loading = $state(false);
  busy = $state(false);
  error = $state("");
  message = $state("");
  private initialized = false;
  private sequence = 0;
  private activeRequestId: string | null = null;

  async load(refresh = false): Promise<void> {
    if (this.loading || (this.initialized && !refresh)) return;
    this.loading = true;
    this.error = "";
    try {
      if (!("__TAURI_INTERNALS__" in window)) {
        this.message = "Screenshots are available in the Windows desktop app.";
        return;
      }
      this.targets = await invoke<CaptureTargets>("list_capture_targets");
      this.initialized = true;
      this.message = this.targets.supported ? "" : "Screen capture isn’t supported on this device.";
    } catch {
      this.error = "Couldn’t load capture targets. Refresh to try again.";
    } finally {
      this.loading = false;
    }
  }

  async capture(mode: CaptureMode, targetId: string): Promise<void> {
    if (this.busy || this.loading || !this.targets.supported || !targetId) return;
    this.busy = true;
    const requestId = `${Date.now()}_${++this.sequence}`;
    this.activeRequestId = requestId;
    this.error = "";
    this.message = mode === "area" ? "Select an area. Press Esc to cancel." : "Taking screenshot…";
    try {
      const result = await invoke<CaptureResult>("start_capture", { mode, targetId, requestId });
      this.message = result.status === "copied" ? "Screenshot copied — paste it anywhere" : "Capture cancelled. Your clipboard is unchanged.";
    } catch (error) {
      this.error = String(error).replace(/^Error:\s*/, "");
      this.message = "";
    } finally {
      this.busy = false;
      this.activeRequestId = null;
    }
  }

  async cancel(): Promise<void> {
    if (!this.busy || !this.activeRequestId) return;
    try { await invoke("cancel_capture", { requestId: this.activeRequestId }); }
    catch { this.error = "Couldn’t cancel yet. Press Esc in the selection window."; }
  }
}

export const captureStore = new CaptureStore();
