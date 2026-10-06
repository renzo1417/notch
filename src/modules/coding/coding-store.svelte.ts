import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { contextEngine, ContextPriority } from "../../app/state/context-engine";
import { initialCodingSession, type CodingSession } from "./types";

class CodingStore {
  state = $state<CodingSession>({ ...initialCodingSession });
  private unlistenFn: UnlistenFn | null = null;
  private isInitializing = false;
  private isSubscribed = false;

  async init(): Promise<void> {
    if (this.isSubscribed || this.isInitializing) return;
    this.isInitializing = true;

    try {
      const initial = await invoke<CodingSession>("get_coding_session");
      if (initial) {
        this.applySession(initial);
      }
    } catch (err) {
      console.warn("[CodingStore] Failed to query initial coding session:", err);
    }

    try {
      this.unlistenFn = await listen<CodingSession>("coding.changed", (event) => {
        if (event.payload) {
          this.applySession(event.payload);
        }
      });
      this.isSubscribed = true;
    } catch (err) {
      console.warn("[CodingStore] Failed to listen to coding.changed event:", err);
    } finally {
      this.isInitializing = false;
    }
  }

  private applySession(session: CodingSession): void {
    this.state = session;

    if (session.isActive) {
      contextEngine.registerCandidate({
        id: "coding",
        mode: "coding",
        priority: ContextPriority.P4_CODING,
        timestamp: session.sessionStartMs || Date.now(),
        data: session,
      });
    } else {
      contextEngine.unregisterCandidate("coding");
    }
  }

  async setProjectLabel(label?: string): Promise<void> {
    const trimmed = label?.trim() || undefined;
    this.state.projectLabel = trimmed;
    try {
      await invoke("set_coding_project_label", { label: trimmed ?? null });
    } catch (err) {
      console.warn("[CodingStore] Failed to set project label:", err);
    }
  }

  async resetSession(): Promise<void> {
    try {
      await invoke("reset_coding_session");
    } catch (err) {
      console.warn("[CodingStore] Failed to reset coding session:", err);
    }
  }

  destroy(): void {
    if (this.unlistenFn) {
      this.unlistenFn();
      this.unlistenFn = null;
    }
    contextEngine.unregisterCandidate("coding");
    this.isSubscribed = false;
    this.isInitializing = false;
  }
}

export const codingStore = new CodingStore();
