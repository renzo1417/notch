<script lang="ts">
  import { onMount } from "svelte";
  import { contextEngine, type ContextCandidate } from "../app/state/context-engine";
  import { timerStore } from "../modules/timer/timer-store.svelte";
  import { codingStore } from "../modules/coding/coding-store.svelte";
  import { downloadsStore } from "../modules/downloads/downloads-store.svelte";
  import { batteryStore } from "../modules/battery/battery-store.svelte";
  import { clipboardStore } from "../modules/clipboard/clipboard-store.svelte";
  import IconTimer from "../icons/IconTimer.svelte";
  import IconDownload from "../icons/IconDownload.svelte";
  import IconClipboard from "../icons/IconClipboard.svelte";
  import IconMonitor from "../icons/IconMonitor.svelte";
  import IconBattery from "../icons/IconBattery.svelte";
  import IconCheck from "../icons/IconCheck.svelte";
  import IconAlertCircle from "../icons/IconAlertCircle.svelte";

  let { compact = false }: { compact?: boolean } = $props();
  let candidate = $state<ContextCandidate>(contextEngine.current);
  onMount(() => contextEngine.subscribe((value) => { candidate = value; }));
  const mode = $derived(candidate.mode);
  const notification = $derived(candidate.data as { title?: string; message?: string } | undefined);
  const label = $derived(
    mode === "timer" ? timerStore.formattedCompact :
    mode === "coding" ? `${codingStore.state.appName || "Coding"}${codingStore.state.elapsedMinutes ? ` · ${codingStore.state.elapsedMinutes}m` : ""}` :
    mode === "download" ? downloadsStore.activeDownload?.fileName || "Downloads" :
    mode === "battery" ? batteryStore.statusSummary :
    mode === "clipboard" ? `Clipboard · ${clipboardStore.count} items` :
    mode === "launcher" ? "Apps" :
    mode === "notification" ? (typeof candidate.data === "string" ? candidate.data : notification?.title || "Complete") : "Ready"
  );
  const detail = $derived(
    mode === "timer" ? (timerStore.state.isPaused ? "Paused" : "Focus session") :
    mode === "coding" ? codingStore.state.projectLabel || "Coding session" :
    mode === "download" ? (downloadsStore.activeDownload?.progressPercent !== undefined ? `${downloadsStore.activeDownload.progressPercent}% downloaded` : "Download in progress") :
    mode === "notification" ? notification?.message || "" : ""
  );
  const ActivityIcon = $derived(candidate.priority === 0 ? IconAlertCircle :
    mode === "timer" ? IconTimer : mode === "download" ? IconDownload :
    mode === "coding" ? IconMonitor : mode === "clipboard" ? IconClipboard :
    mode === "battery" ? IconBattery : IconCheck);
</script>

<div class="activity-summary" class:compact class:warning={candidate.priority === 0} title={[label, detail].filter(Boolean).join(" · ")}>
  {#if mode !== "idle"}<span class="activity-icon" aria-hidden="true"><ActivityIcon size={16} /></span>{/if}
  <div class="activity-text">
    <span class="activity-label">{label}</span>
    {#if !compact && detail}<span class="activity-detail">{detail}</span>{/if}
  </div>
</div>

<style>
  .activity-summary { display: flex; align-items: center; gap: 8px; min-width: 0; color: var(--text-primary); }
  .activity-icon { display: flex; flex: none; color: var(--text-secondary); }
  .activity-text { display: grid; gap: 3px; min-width: 0; }
  .activity-label { font-size: 14px; font-weight: 600; font-variant-numeric: tabular-nums; }
  .activity-detail { font-size: 12px; color: var(--text-secondary); }
  .activity-label, .activity-detail { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .compact .activity-label { font-size: 13px; font-weight: 500; }
  .warning .activity-label, .warning .activity-icon { color: var(--status-warning); }
</style>
