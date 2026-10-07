import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { contextEngine, ContextPriority } from "../../app/state/context-engine.ts";
import { notificationStore } from "../notifications/notification-store.svelte.ts";
import type { ClipboardItem } from "./types.ts";

class ClipboardStore {
  items = $state<ClipboardItem[]>([]);
  isPaused = $state<boolean>(false);
  maxItems = $state<number>(25);
  copiedId = $state<string | null>(null);
  isLoading = $state<boolean>(false);

  private unlistenChanged: UnlistenFn | null = null;
  private copyTimeout: ReturnType<typeof setTimeout> | null = null;

  get count(): number {
    return this.items.length;
  }

  get latest(): ClipboardItem | null {
    return this.items.length > 0 ? this.items[0] : null;
  }

  async init(): Promise<void> {
    this.isLoading = true;
    try {
      const history = await invoke<ClipboardItem[]>("get_clipboard_history");
      this.items = history;
      const paused = await invoke<boolean>("is_clipboard_paused");
      this.isPaused = paused;
    } catch (err) {
      console.error("Failed to initialize clipboard store:", err);
    } finally {
      this.isLoading = false;
    }

    // Subscribe to native Windows WM_CLIPBOARDUPDATE event stream
    if (!this.unlistenChanged) {
      try {
        this.unlistenChanged = await listen<ClipboardItem>("clipboard.changed", (event) => {
          this.handleNativeClipboardChange(event.payload);
        });
      } catch (err) {
        console.error("Failed to listen for clipboard events:", err);
      }
    }
  }

  /**
   * Internal handler for native clipboard change events.
   * Spec Invariant: Clipboard changes do NOT automatically hijack the center context!
   * Dedupes consecutive identical items and enforces max history count.
   */
  private handleNativeClipboardChange(newItem: ClipboardItem): void {
    if (this.isPaused) return;

    // Deduplicate consecutive identical entries
    if (this.items.length > 0 && this.items[0].text === newItem.text) {
      return;
    }

    const updated = [newItem, ...this.items.filter((item) => item.text !== newItem.text)];
    this.items = updated.slice(0, this.maxItems);
  }

  /**
   * Opens the clipboard history surface in the center context.
   * Registers at P5_USER_SURFACE (same priority tier as Launcher).
   */
  openClipboardSurface(): void {
    contextEngine.registerCandidate({
      id: "clipboard",
      mode: "clipboard",
      priority: ContextPriority.P5_USER_SURFACE,
      timestamp: Date.now(),
    });
  }

  /**
   * Closes the clipboard history surface and returns to previous meaningful context.
   */
  closeClipboardSurface(): void {
    contextEngine.unregisterCandidate("clipboard");
  }

  /**
   * Copies an entry back to the native Windows clipboard.
   */
  async copyItem(item: ClipboardItem): Promise<boolean> {
    try {
      await invoke("copy_to_clipboard", { text: item.text });
      this.copiedId = item.id;

      if (this.copyTimeout) {
        clearTimeout(this.copyTimeout);
      }
      this.copyTimeout = setTimeout(() => {
        if (this.copiedId === item.id) {
          this.copiedId = null;
        }
      }, 1800);

      notificationStore.notify({
        title: "Copied to Clipboard",
        message: item.text.length > 40 ? `${item.text.slice(0, 37)}...` : item.text,
        durationMs: 2000,
      });

      return true;
    } catch (err) {
      console.error("Failed to copy item to clipboard:", err);
      return false;
    }
  }

  /**
   * Clears in-memory clipboard history.
   */
  async clearHistory(): Promise<void> {
    try {
      await invoke("clear_clipboard_history");
      this.items = [];
      notificationStore.notify({
        title: "Clipboard Cleared",
        message: "History emptied",
        durationMs: 2000,
      });
    } catch (err) {
      console.error("Failed to clear clipboard history:", err);
    }
  }

  /**
   * Toggles pause/resume of clipboard capture.
   */
  async togglePause(): Promise<void> {
    const next = !this.isPaused;
    try {
      await invoke("set_clipboard_paused", { paused: next });
      this.isPaused = next;
      notificationStore.notify({
        title: next ? "Clipboard Paused" : "Clipboard Resumed",
        message: next ? "Capture temporarily suspended" : "Listening for copy events",
        durationMs: 2200,
      });
    } catch (err) {
      console.error("Failed to toggle clipboard pause:", err);
    }
  }

  /**
   * Sets clipboard paused state explicitly.
   */
  async setPaused(paused: boolean): Promise<void> {
    try {
      await invoke("set_clipboard_paused", { paused });
      this.isPaused = paused;
    } catch (err) {
      console.error("Failed to set clipboard paused:", err);
    }
  }

  /**
   * Sets maximum history items (1..=100).
   */
  async setMaxItems(max: number): Promise<void> {
    const clamped = Math.max(1, Math.min(100, max));
    try {
      await invoke("set_clipboard_max_items", { maxItems: clamped });
      this.maxItems = clamped;
      if (this.items.length > clamped) {
        this.items = this.items.slice(0, clamped);
      }
    } catch (err) {
      console.error("Failed to set max items:", err);
    }
  }

  destroy(): void {
    if (this.unlistenChanged) {
      this.unlistenChanged();
      this.unlistenChanged = null;
    }
    if (this.copyTimeout) {
      clearTimeout(this.copyTimeout);
      this.copyTimeout = null;
    }
  }
}

export const clipboardStore = new ClipboardStore();
