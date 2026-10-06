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

  async togglePlayPause(): Promise<void> {
    const caps = this.state.capabilities;
    if (!caps.canTogglePlayPause && !caps.canPlay && !caps.canPause) {
      return;
    }

    // Optimistic toggle
    const prevPlaying = this.state.isPlaying;
    this.state.isPlaying = !prevPlaying;

    try {
      await invoke("media_toggle_play_pause");
    } catch (err) {
      console.warn("[MediaStore] media_toggle_play_pause failed:", err);
      this.state.isPlaying = prevPlaying;
    }
  }

  async previous(): Promise<void> {
    if (!this.state.capabilities.canSkipPrevious) return;
    try {
      await invoke("media_previous");
    } catch (err) {
      console.warn("[MediaStore] media_previous failed:", err);
    }
  }

  async next(): Promise<void> {
    if (!this.state.capabilities.canSkipNext) return;
    try {
      await invoke("media_next");
    } catch (err) {
      console.warn("[MediaStore] media_next failed:", err);
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
