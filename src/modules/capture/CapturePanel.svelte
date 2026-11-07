<script lang="ts">
  import { onMount, tick } from "svelte";
  import { captureStore } from "./capture-store.svelte";
  import { captureAction, defaultCaptureMonitor } from "./capture-model";
  import { settingsStore } from "../../settings/settings-store.svelte";
  import type { CaptureMode } from "./types";
  import IconCamera from "../../icons/IconCamera.svelte";
  let mode = $state<CaptureMode>("area");
  let monitorId = $state("");
  let windowId = $state("");
  let action: HTMLButtonElement;
  const targetId = $derived(mode === "window" ? windowId : monitorId);
  const choices = $derived(mode === "window" ? captureStore.targets.windows : captureStore.targets.monitors);
  $effect(() => {
    const targets = captureStore.targets;
    if (!targets.monitors.some((monitor) => monitor.id === monitorId)) monitorId = defaultCaptureMonitor(targets.monitors, settingsStore.state.general.selectedMonitor);
    if (!targets.windows.some((target) => target.id === windowId)) windowId = targets.windows[0]?.id ?? "";
  });
  onMount(() => { void captureStore.load(); });
  async function takeScreenshot() {
    await captureStore.capture(mode, targetId);
    await tick();
    action?.focus({ preventScroll: true });
  }
</script>

<section class="tool-panel capture-panel" aria-label="Screenshot capture">
  <div class="panel-header"><h2 class="panel-title">Screenshot</h2><button type="button" class="refresh" disabled={captureStore.busy || captureStore.loading} onclick={() => captureStore.load(true)}>{captureStore.loading ? "Loading…" : "Refresh"}</button></div>
  <div class="mode-options" role="group" aria-label="Capture mode">
    {#each ["area", "window", "screen"] as option}<button type="button" disabled={captureStore.busy} class:active={mode === option} aria-pressed={mode === option} onclick={() => { mode = option as CaptureMode; }}>{option === "area" ? "Area" : option === "window" ? "Window" : "Screen"}</button>{/each}
  </div>
  <div class="target-row"><label class="target-label" for="capture-target">{mode === "window" ? "Application window" : "Monitor"}</label>
  <select id="capture-target" disabled={captureStore.busy || captureStore.loading || !choices.length} value={targetId} onchange={(event) => { if (mode === "window") windowId = event.currentTarget.value; else monitorId = event.currentTarget.value; }}>
    {#if !choices.length}<option value="">{captureStore.loading ? "Loading targets…" : "No available targets"}</option>{/if}
    {#each choices as target}<option value={target.id}>{target.name}</option>{/each}
  </select></div>
  <p class="capture-description">Copies to your clipboard. {mode === "area" ? "Drag or use arrow keys; Copy or Enter confirms. Esc cancels." : "The notch is hidden while capturing."}</p>
  <div class="capture-actions">
    <button bind:this={action} type="button" class="capture-action" data-focus="capture-start" disabled={captureStore.busy || captureStore.loading || !captureStore.targets.supported || !targetId} onclick={takeScreenshot}><IconCamera /><span>{captureStore.busy ? "Capturing…" : captureAction(mode)}</span></button>
    {#if captureStore.busy}<button type="button" class="cancel" onclick={() => captureStore.cancel()}>Cancel</button>{/if}
  </div>
  {#if captureStore.error || captureStore.message}<p class="capture-status" class:error={!!captureStore.error} role={captureStore.error ? "alert" : "status"}>{captureStore.error || captureStore.message}</p>{/if}
</section>

<style>
  .capture-panel { gap: 6px; min-height: 0; }
  .target-row { display: flex; align-items: center; gap: 10px; }
  .target-row select { flex: 1; width: auto; }
  .refresh { padding: 4px 8px; color: var(--text-secondary); font-size: 12px; }
  .mode-options { display: flex; gap: 4px; }
  .mode-options button { flex: 1; color: var(--text-secondary); background: var(--surface-raised); font-size: 12px; }
  .mode-options button.active { color: var(--accent-primary); background: var(--accent-muted); }
  .target-label { font-size: 12px; color: var(--text-secondary); }
  select { flex: none; min-height: 32px; width: 100%; min-width: 0; color: var(--text-primary); background: var(--surface-raised); border: 1px solid var(--border-subtle); border-radius: 8px; padding: 4px 8px; font: inherit; font-size: 12px; }
  .capture-description, .capture-status { flex: none; font-size: 12px; line-height: 1.4; color: var(--text-secondary); }
  .capture-actions { display: flex; gap: 8px; align-items: center; }
  .capture-action { display: inline-flex; align-items: center; gap: 8px; padding: 4px 12px; color: var(--on-accent); background: var(--accent-primary); font-size: 12px; }
  .capture-action:hover:not(:disabled) { background: var(--accent-primary); filter: brightness(.93); }
  .cancel { padding: 4px 12px; }
  .capture-status.error { color: var(--status-error); }
</style>
