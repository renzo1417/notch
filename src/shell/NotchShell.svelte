<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import CompactNotch from "./CompactNotch.svelte";
  import type { NotchShellState } from "../app/state/shell-state";

  interface PingResponse {
    message: string;
    version: string;
    timestamp_ms: number;
    backend_ready: boolean;
  }

  let shellState = $state<NotchShellState>("compact");
  let backendStatus = $state("Connecting...");
  let pinging = $state(false);
  let lastLatencyMs = $state<number | null>(null);

  async function pingBackend() {
    pinging = true;
    const start = performance.now();
    try {
      const res = await invoke<PingResponse>("ping", {
        clientTimestampMs: Date.now(),
      });
      const end = performance.now();
      lastLatencyMs = Math.round(end - start);
      if (res.backend_ready) {
        backendStatus = `Connected (v${res.version}, ${lastLatencyMs}ms)`;
      } else {
        backendStatus = "Backend reported unready";
      }
    } catch (err) {
      console.error("IPC ping failed:", err);
      backendStatus = "Bridge Offline";
    } finally {
      pinging = false;
    }
  }

  onMount(() => {
    pingBackend();
  });
</script>

<div class="notch-wrapper">
  <div class="notch-shell" data-state={shellState}>
    <CompactNotch 
      {backendStatus} 
      {pinging} 
      onPing={pingBackend} 
    />
  </div>
</div>

<style>
  .notch-wrapper {
    display: flex;
    justify-content: center;
    width: 100%;
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
    height: 40px;
    display: flex;
    align-items: center;
    padding: 0 var(--space-4);
    transition: 
      width var(--motion-shell) var(--ease-natural),
      height var(--motion-shell) var(--ease-natural),
      background-color var(--motion-micro) var(--ease-natural);
  }

  .notch-shell:hover {
    border-color: var(--border-strong);
  }
</style>
