<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import CompactNotch from "./CompactNotch.svelte";
  import ExpandedNotch from "./ExpandedNotch.svelte";
  import type { NotchShellState } from "../app/state/shell-state";
  import { SHELL_MOTION } from "./shell-motion";

  interface PingResponse {
    message: string;
    version: string;
    timestamp_ms: number;
    backend_ready: boolean;
  }

  let shellState = $state<NotchShellState>("compact");
  let backendConnected = $state(false);
  let collapseTimeout: ReturnType<typeof setTimeout> | null = null;

  async function syncWindowState(state: "compact" | "expanded") {
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

  async function toggleExpansion() {
    if (collapseTimeout) {
      clearTimeout(collapseTimeout);
      collapseTimeout = null;
    }

    if (shellState === "compact") {
      // 1. Expand the native window first so the shell has room to animate
      await syncWindowState("expanded");
      // 2. Trigger CSS expansion animation
      shellState = "expanded";
    } else {
      // 1. Trigger CSS collapse animation
      shellState = "compact";
      // 2. After CSS animation completes, shrink the native window bounds to keep hit area tight
      collapseTimeout = setTimeout(() => {
        syncWindowState("compact");
        collapseTimeout = null;
      }, SHELL_MOTION.shellMs);
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      toggleExpansion();
    }
  }

  onMount(() => {
    checkBackend();
    // Ensure initial compact placement
    syncWindowState("compact");
  });

  onDestroy(() => {
    if (collapseTimeout) {
      clearTimeout(collapseTimeout);
    }
  });
</script>

<div class="notch-wrapper">
  <div
    class="notch-shell"
    class:expanded={shellState === "expanded"}
    role="button"
    tabindex="0"
    aria-label={shellState === "expanded" ? "Collapse notch to compact view" : "Expand notch to detailed view"}
    aria-expanded={shellState === "expanded"}
    onclick={toggleExpansion}
    onkeydown={handleKeyDown}
  >
    {#if shellState === "compact"}
      <CompactNotch {backendConnected} />
    {:else}
      <ExpandedNotch {backendConnected} />
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
</style>
