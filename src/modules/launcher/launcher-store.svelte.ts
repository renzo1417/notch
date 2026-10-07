import { invoke } from "@tauri-apps/api/core";
import { contextEngine, ContextPriority } from "../../app/state/context-engine.ts";
import { notificationStore } from "../notifications/notification-store.svelte.ts";
import { mascotStore } from "../../mascot/mascot-store.svelte.ts";
import { shellActions } from "../../app/state/shell-state.ts";
import { MAX_PINNED_BEFORE_OVERFLOW, type LaunchableApp, type InspectedApp } from "./types.ts";

export { MAX_PINNED_BEFORE_OVERFLOW };

class LauncherStore {
  apps = $state<LaunchableApp[]>([]);
  isLoading = $state(false);
  launchingAppId = $state<string | null>(null);
  brokenApp = $state<LaunchableApp | null>(null);
  editingApp = $state<LaunchableApp | null>(null);

  get pinnedApps(): LaunchableApp[] {
    return this.apps
      .filter((app) => app.enabled)
      .sort((a, b) => a.order - b.order);
  }

  get visiblePinnedApps(): LaunchableApp[] {
    return this.pinnedApps.slice(0, MAX_PINNED_BEFORE_OVERFLOW);
  }

  get overflowCount(): number {
    const total = this.pinnedApps.length;
    return total > MAX_PINNED_BEFORE_OVERFLOW ? total - MAX_PINNED_BEFORE_OVERFLOW : 0;
  }

  async init(): Promise<void> {
    this.isLoading = true;
    try {
      const result = await invoke<LaunchableApp[]>("get_launcher_apps");
      this.apps = result;
    } catch (err) {
      console.error("Failed to load launcher apps:", err);
    } finally {
      this.isLoading = false;
    }
  }

  openLauncherSurface(): void {
    contextEngine.registerCandidate({
      id: "launcher",
      mode: "launcher",
      priority: ContextPriority.P5_USER_SURFACE,
      timestamp: Date.now(),
    });
  }

  closeLauncherSurface(): void {
    contextEngine.unregisterCandidate("launcher");
    this.brokenApp = null;
    this.editingApp = null;
  }

  async pickAndAdd(): Promise<boolean> {
    try {
      const selectedPath = await invoke<string | null>("pick_app_file");
      if (!selectedPath) {
        return false;
      }

      const inspected = await invoke<InspectedApp>("inspect_app_file", {
        path: selectedPath,
      });

      const newApp: LaunchableApp = {
        id: `app_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`,
        name: inspected.name,
        executablePath: inspected.executablePath,
        args: inspected.args,
        iconDataUrl: inspected.iconDataUrl,
        enabled: true,
        order: this.apps.length,
        exists: true,
      };

      const added = await invoke<LaunchableApp>("add_launcher_app", {
        app: newApp,
      });

      this.apps = [...this.apps, added];
      mascotStore.setMood("happy", 1800);
      notificationStore.notify({
        title: "App Added",
        message: added.name,
        durationMs: 2500,
      });
      return true;
    } catch (err) {
      console.error("Failed to pick and add app:", err);
      notificationStore.notify({
        title: "Add App Failed",
        message: String(err),
        durationMs: 3000,
      });
      return false;
    }
  }

  async launch(app: LaunchableApp): Promise<boolean> {
    if (app.exists === false) {
      this.brokenApp = app;
      mascotStore.setMood("warning", 2500);
      notificationStore.notify({
        title: "App Not Found",
        message: `${app.name} is missing or moved`,
        durationMs: 3500,
      });
      return false;
    }

    this.launchingAppId = app.id;
    try {
      await invoke("launch_app", { id: app.id });

      // Tactile internal launch feedback
      mascotStore.setMood("happy", 2000);
      notificationStore.notify({
        title: "Launched",
        message: app.name,
        durationMs: 2200,
      });

      // Collapse the notch after successful launch so user can use the opened application
      setTimeout(() => {
        shellActions.requestCollapse();
      }, 350);

      return true;
    } catch (err) {
      console.error("Failed to launch app:", err);
      const errMsg = String(err);
      if (errMsg.includes("AppNotFound")) {
        // Mark app as broken locally
        this.apps = this.apps.map((a) =>
          a.id === app.id ? { ...a, exists: false } : a
        );
        this.brokenApp = { ...app, exists: false };
      }

      mascotStore.setMood("warning", 3000);
      notificationStore.notify({
        title: "Launch Failed",
        message: `Could not launch ${app.name}`,
        durationMs: 3500,
      });
      return false;
    } finally {
      setTimeout(() => {
        if (this.launchingAppId === app.id) {
          this.launchingAppId = null;
        }
      }, 500);
    }
  }

  async locate(app: LaunchableApp): Promise<boolean> {
    try {
      const selectedPath = await invoke<string | null>("pick_app_file");
      if (!selectedPath) {
        return false;
      }

      const updated = await invoke<LaunchableApp>("locate_app", {
        id: app.id,
        newPath: selectedPath,
      });

      this.apps = this.apps.map((a) => (a.id === app.id ? updated : a));
      this.brokenApp = null;
      mascotStore.setMood("happy", 2000);
      notificationStore.notify({
        title: "App Relocated",
        message: `${updated.name} updated`,
        durationMs: 2500,
      });
      return true;
    } catch (err) {
      console.error("Failed to relocate app:", err);
      notificationStore.notify({
        title: "Relocate Failed",
        message: String(err),
        durationMs: 3000,
      });
      return false;
    }
  }

  async remove(app: LaunchableApp): Promise<boolean> {
    try {
      await invoke("remove_launcher_app", { id: app.id });
      this.apps = this.apps.filter((a) => a.id !== app.id);
      if (this.brokenApp?.id === app.id) {
        this.brokenApp = null;
      }
      if (this.editingApp?.id === app.id) {
        this.editingApp = null;
      }
      notificationStore.notify({
        title: "App Removed",
        message: `${app.name} removed from dock`,
        durationMs: 2000,
      });
      return true;
    } catch (err) {
      console.error("Failed to remove app:", err);
      return false;
    }
  }

  async reorder(orderedIds: string[]): Promise<void> {
    try {
      const reordered = await invoke<LaunchableApp[]>("reorder_launcher_apps", {
        ids: orderedIds,
      });
      this.apps = reordered;
    } catch (err) {
      console.error("Failed to reorder apps:", err);
    }
  }

  async moveLeft(app: LaunchableApp): Promise<void> {
    const list = [...this.pinnedApps];
    const index = list.findIndex((a) => a.id === app.id);
    if (index > 0) {
      const temp = list[index - 1];
      list[index - 1] = list[index];
      list[index] = temp;
      await this.reorder(list.map((a) => a.id));
    }
  }

  async moveRight(app: LaunchableApp): Promise<void> {
    const list = [...this.pinnedApps];
    const index = list.findIndex((a) => a.id === app.id);
    if (index >= 0 && index < list.length - 1) {
      const temp = list[index + 1];
      list[index + 1] = list[index];
      list[index] = temp;
      await this.reorder(list.map((a) => a.id));
    }
  }

  async saveEdit(app: LaunchableApp): Promise<boolean> {
    try {
      const updated = await invoke<LaunchableApp>("update_launcher_app", {
        app,
      });
      this.apps = this.apps.map((a) => (a.id === app.id ? updated : a));
      this.editingApp = null;
      notificationStore.notify({
        title: "App Updated",
        message: updated.name,
        durationMs: 2000,
      });
      return true;
    } catch (err) {
      console.error("Failed to update app:", err);
      return false;
    }
  }

  async toggleAppEnabled(id: string, enabled: boolean): Promise<void> {
    const app = this.apps.find((a) => a.id === id);
    if (app) {
      await this.saveEdit({ ...app, enabled });
    }
  }

  async removeById(id: string): Promise<boolean> {
    const app = this.apps.find((a) => a.id === id);
    if (app) {
      return this.remove(app);
    }
    return false;
  }
}

export const launcherStore = new LauncherStore();
