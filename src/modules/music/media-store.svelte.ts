import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { DEFAULT_MEDIA_STATE, type MediaState } from "./types";

class MediaStore {
  state = $state<MediaState>({ ...DEFAULT_MEDIA_STATE });
  private unlistenFn: UnlistenFn | null = null;
  private isInitializing = false;
  private isSubscribed = false;

  async init(): Promise<void> {
    if (this.isSubscribed || this.isInitializing) return;
    this.isInitializing = true;

    try {
      const initial = await invoke<MediaState>("get_media_state");
      if (initial) {
        this.state = initial;
      }
    } catch (err) {
      console.warn("[MediaStore] Failed to query initial media state:", err);
    }

    try {
      this.unlistenFn = await listen<MediaState>("media-state-changed", (event) => {
        if (event.payload) {
          this.state = event.payload;
        }
      });
      this.isSubscribed = true;
    } catch (err) {
      console.warn("[MediaStore] Failed to subscribe to media-state-changed event:", err);
    } finally {
      this.isInitializing = false;
    }
  }

  async togglePlayPause(): Promise<boolean> {
    const caps = this.state.capabilities;
    if (!caps.canTogglePlayPause && !caps.canPlay && !caps.canPause) {
      return false;
    }

    // Optimistic toggle
    const prevPlaying = this.state.isPlaying;
    this.state.isPlaying = !prevPlaying;

    try {
      await invoke("media_toggle_play_pause");
      return true;
    } catch (err) {
      console.warn("[MediaStore] media_toggle_play_pause failed:", err);
      this.state.isPlaying = prevPlaying;
      return false;
    }
  }

  async previous(): Promise<boolean> {
    if (!this.state.capabilities.canSkipPrevious) return false;
    try {
      await invoke("media_previous");
      return true;
    } catch (err) {
      console.warn("[MediaStore] media_previous failed:", err);
      return false;
    }
  }

  async next(): Promise<boolean> {
    if (!this.state.capabilities.canSkipNext) return false;
    try {
      await invoke("media_next");
      return true;
    } catch (err) {
      console.warn("[MediaStore] media_next failed:", err);
      return false;
    }
  }

  destroy(): void {
    if (this.unlistenFn) {
      this.unlistenFn();
      this.unlistenFn = null;
    }
    this.isSubscribed = false;
    this.isInitializing = false;
  }
}

export const mediaStore = new MediaStore();
