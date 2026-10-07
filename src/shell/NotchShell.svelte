<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import CompactNotch from "./CompactNotch.svelte";
  import ExpandedNotch from "./ExpandedNotch.svelte";
  import SettingsView from "../settings/SettingsView.svelte";
  import type { NotchShellState } from "../app/state/shell-state";
  import { shellActions } from "../app/state/shell-state";
  import { SHELL_MOTION } from "./shell-motion";
  import { settingsStore } from "../settings/settings-store.svelte";

  interface PingResponse {
    message: string;
    version: string;
    timestamp_ms: number;
    backend_ready: boolean;
  }

  let shellState = $state<NotchShellState>("compact");
  let backendConnected = $state(false);
  let collapseTimeout: ReturnType<typeof setTimeout> | null = null;
  let autoCollapseTimer: ReturnType<typeof setTimeout> | null = null;
  let unsubCollapse: (() => void) | null = null;
  let unlistenOpenSettings: UnlistenFn | null = null;

  async function syncWindowState(state: "compact" | "expanded" | "settings") {
    try {
      await invoke("set_notch_state", { notchState: state });
    } catch (err) {
      console.error("Failed to sync notch window state:", err);
    }
  }

  async function checkBackend() {
    try {
      const res = await invoke<PingResponse>("ping", {
        clientTimestampMs: Date.now(),
      });
      backendConnected = res.backend_ready;
    } catch {
      backendConnected = false;
    }
  }

  function clearAutoCollapse() {
    if (autoCollapseTimer) {
      clearTimeout(autoCollapseTimer);
      autoCollapseTimer = null;
    }
  }

  function startAutoCollapse() {
    clearAutoCollapse();
    if (shellState === "expanded") {
      const delay = settingsStore.state.general.autoCollapseDelayMs;
      if (delay > 0) {
        autoCollapseTimer = setTimeout(() => {
          collapseNotch();
        }, delay);
      }
    }
  }

  function collapseNotch() {
    clearAutoCollapse();
    if (collapseTimeout) {
      clearTimeout(collapseTimeout);
      collapseTimeout = null;
    }
    if (shellState !== "compact") {
      shellState = "compact";
      collapseTimeout = setTimeout(() => {
        syncWindowState("compact");
        collapseTimeout = null;
      }, SHELL_MOTION.shellMs);
    }
  }

  async function openSettings() {
    clearAutoCollapse();
    if (collapseTimeout) {
      clearTimeout(collapseTimeout);
      collapseTimeout = null;
    }
    await syncWindowState("settings");
    shellState = "settings";
  }

  async function closeSettings() {
    clearAutoCollapse();
    shellState = "expanded";
    await syncWindowState("expanded");
    startAutoCollapse();
  }

  async function toggleExpansion() {
    if (shellState === "settings") {
      return;
    }

    if (collapseTimeout) {
      clearTimeout(collapseTimeout);
      collapseTimeout = null;
    }

    if (shellState === "compact") {
      await syncWindowState("expanded");
      shellState = "expanded";
      startAutoCollapse();
    } else {
      collapseNotch();
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (shellState === "settings") {
      if (event.key === "Escape") {
        event.preventDefault();
        closeSettings();
      }
      return;
    }

    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      toggleExpansion();
    }
  }

  onMount(async () => {
    settingsStore.init();
    checkBackend();
    syncWindowState("compact");

    unsubCollapse = shellActions.onCollapse(() => {
      collapseNotch();
    });

    if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
      try {
        unlistenOpenSettings = await listen("open-settings", () => {
          openSettings();
        });
      } catch {
        // Ignore outside Tauri runtime
      }
    }
  });

  onDestroy(() => {
    if (collapseTimeout) clearTimeout(collapseTimeout);
    clearAutoCollapse();
    if (unsubCollapse) unsubCollapse();
    if (unlistenOpenSettings) unlistenOpenSettings();
    settingsStore.destroy();
  });
</script>

<div class="notch-wrapper">
  <div
    class="notch-shell"
    class:expanded={shellState === "expanded"}
    class:settings={shellState === "settings"}
    role="button"
    tabindex="0"
    aria-label={shellState === "settings" ? "Settings Panel" : shellState === "expanded" ? "Collapse notch to compact view" : "Expand notch to detailed view"}
    aria-expanded={shellState === "expanded" || shellState === "settings"}
    onclick={toggleExpansion}
    onkeydown={handleKeyDown}
    onmouseenter={clearAutoCollapse}
    onmouseleave={startAutoCollapse}
  >
    {#if shellState === "compact"}
      <CompactNotch {backendConnected} />
    {:else if shellState === "settings"}
      <SettingsView onclose={closeSettings} />
    {:else}
      <ExpandedNotch {backendConnected} onOpenSettings={openSettings} />
    {/if}
  </div>
</div>

<style>
  .notch-wrapper {
    display: flex;
    justify-content: center;
    align-items: flex-start;
    width: 100%;
    height: 100%;
    padding-top: 0;
    pointer-events: none;
  }

  .notch-shell {
    pointer-events: auto;
    background-color: var(--surface-root);
    border: 1px solid var(--border-subtle);
    border-top: none;
    border-bottom-left-radius: var(--radius-shell);
    border-bottom-right-radius: var(--radius-shell);
    box-shadow: var(--shadow-soft);
    cursor: pointer;
    overflow: hidden;
    outline: none;

    /* Resting compact geometry */
    width: 300px;
    height: 38px;

    /* GPU-accelerated smooth transitions */
    transition: 
      width var(--motion-shell) var(--ease-natural),
      height var(--motion-shell) var(--ease-natural),
      border-color var(--motion-micro) var(--ease-natural),
      box-shadow var(--motion-micro) var(--ease-natural);
  }

  .notch-shell:hover {
    border-color: var(--border-strong);
  }

  .notch-shell:focus-visible {
    outline: 2px solid var(--accent-focus);
    outline-offset: 1px;
  }

  /* Expanded geometry */
  .notch-shell.expanded {
    width: 520px;
    height: 76px;
  }

  /* Settings geometry */
  .notch-shell.settings {
    width: 540px;
    height: 360px;
    cursor: default;
    border-radius: var(--radius-shell);
  }
</style>
