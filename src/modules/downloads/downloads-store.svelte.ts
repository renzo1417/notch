import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { contextEngine, ContextPriority } from "../../app/state/context-engine";
import { notificationStore } from "../notifications/notification-store.svelte";
import { mascotStore } from "../../mascot/mascot-store.svelte";
import type { DownloadState, WatchedFolder } from "./types";

class DownloadsStore {
  activeDownload = $state<DownloadState | null>(null);
  recentCompleted = $state<DownloadState | null>(null);
  watchedFolders = $state<WatchedFolder[]>([]);
  isLoading = $state<boolean>(false);
  showFolderManager = $state<boolean>(false);

  private unlistenChanged: UnlistenFn | null = null;
  private unlistenCompleted: UnlistenFn | null = null;
  private isInitialized = false;

  get hasActiveDownload(): boolean {
    return this.activeDownload !== null;
  }

  async init(): Promise<void> {
    if (this.isInitialized) return;
    this.isInitialized = true;
    this.isLoading = true;

    try {
      const folders = await invoke<WatchedFolder[]>("get_watched_folders");
      this.watchedFolders = folders;

      const state = await invoke<DownloadState | null>("get_download_state");
      if (state && !state.isCompleted) {
        this.activeDownload = state;
        contextEngine.registerCandidate({
          id: "download",
          mode: "download",
          priority: ContextPriority.P3_DOWNLOAD,
          timestamp: Date.now(),
          data: state,
        });
      }
    } catch (err) {
      console.error("Failed to initialize downloads store:", err);
    } finally {
      this.isLoading = false;
    }

    // Subscribe to native download.changed events
    if (!this.unlistenChanged) {
      try {
        this.unlistenChanged = await listen<DownloadState>("download.changed", (event) => {
          this.handleDownloadChanged(event.payload);
        });
      } catch (err) {
        console.error("Failed to listen for download.changed events:", err);
      }
    }

    // Subscribe to native download.completed events
    if (!this.unlistenCompleted) {
      try {
        this.unlistenCompleted = await listen<DownloadState>("download.completed", (event) => {
          this.handleDownloadCompleted(event.payload);
        });
      } catch (err) {
        console.error("Failed to listen for download.completed events:", err);
      }
    }
  }

  /**
   * Handles incoming download in progress events.
   * Registers at P3_DOWNLOAD in the context priority engine.
   */
  private handleDownloadChanged(state: DownloadState): void {
    this.activeDownload = state;

    contextEngine.registerCandidate({
      id: "download",
      mode: "download",
      priority: ContextPriority.P3_DOWNLOAD,
      timestamp: Date.now(),
      data: state,
    });
  }

  /**
   * Handles download completion events.
   * Requirement 7:
   * 1. Short center notification (P1_COMPLETION)
   * 2. Mascot celebration
   * 3. Open Folder action
   */
  private handleDownloadCompleted(state: DownloadState): void {
    this.activeDownload = null;
    this.recentCompleted = state;

    // Unregister active downloading candidate
    contextEngine.unregisterCandidate("download");

    // 1. Trigger short center notification at P1_COMPLETION
    notificationStore.notify({
      title: "Download complete",
      message: state.fileName,
      durationMs: 4500,
    });

    // 2. Trigger mascot celebration animation
    mascotStore.celebrate(3500);
  }

  /**
   * Opens the download folder in Windows File Explorer or selects the file.
   */
  async openFolder(path?: string): Promise<void> {
    const targetPath = path || this.recentCompleted?.filePath || this.activeDownload?.filePath;
    try {
      await invoke("open_download_folder", { path: targetPath ?? null });
    } catch (err) {
      console.error("Failed to open download folder:", err);
    }
  }

  /**
   * Displays native folder picker dialog and adds the selected folder to watched folders.
   */
  async pickAndAddFolder(): Promise<void> {
    try {
      const selected = await invoke<string | null>("pick_download_folder");
      if (selected) {
        const added = await invoke<WatchedFolder>("add_watched_folder", { path: selected });
        this.watchedFolders = [...this.watchedFolders.filter((f) => f.id !== added.id), added];

        notificationStore.notify({
          title: "Watched Folder Added",
          message: added.path,
          durationMs: 2500,
        });
      }
    } catch (err) {
      console.error("Failed to add watched folder:", err);
      notificationStore.notify({
        title: "Could Not Add Folder",
        message: String(err),
        durationMs: 3000,
      });
    }
  }

  /**
   * Removes a watched folder and stops its filesystem watcher.
   */
  async removeFolder(id: string): Promise<void> {
    try {
      await invoke("remove_watched_folder", { id });
      this.watchedFolders = this.watchedFolders.filter((f) => f.id !== id);

      notificationStore.notify({
        title: "Watched Folder Removed",
        message: "Watcher stopped",
        durationMs: 2000,
      });
    } catch (err) {
      console.error("Failed to remove watched folder:", err);
    }
  }

  /**
   * Manually opens the download surface in the center context (P5_USER_SURFACE).
   */
  openDownloadSurface(): void {
    contextEngine.registerCandidate({
      id: "download_surface",
      mode: "download",
      priority: ContextPriority.P5_USER_SURFACE,
      timestamp: Date.now(),
    });
  }

  /**
   * Closes the manually opened download surface.
   */
  closeDownloadSurface(): void {
    contextEngine.unregisterCandidate("download_surface");
    this.showFolderManager = false;
  }

  toggleFolderManager(): void {
    this.showFolderManager = !this.showFolderManager;
  }

  destroy(): void {
    if (this.unlistenChanged) {
      this.unlistenChanged();
      this.unlistenChanged = null;
    }
    if (this.unlistenCompleted) {
      this.unlistenCompleted();
      this.unlistenCompleted = null;
    }
    this.isInitialized = false;
  }
}

export const downloadsStore = new DownloadsStore();
