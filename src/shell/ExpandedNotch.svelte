<script lang="ts">
  import { onMount } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import MascotCanvas from "../mascot/MascotCanvas.svelte";
  import ActivitySummary from "./ActivitySummary.svelte";
  import MusicModule from "../modules/music/MusicModule.svelte";
  import AppsPanel from "../modules/launcher/AppsPanel.svelte";
  import InstalledAppsPanel from "../modules/launcher/InstalledAppsPanel.svelte";
  import CapturePanel from "../modules/capture/CapturePanel.svelte";
  import { captureStore } from "../modules/capture/capture-store.svelte";
  import IconCamera from "../icons/IconCamera.svelte";
  import FocusTimerPanel from "../modules/timer/FocusTimerPanel.svelte";
  import ClipboardPanel from "../modules/clipboard/ClipboardPanel.svelte";
  import DownloadsPanel from "../modules/downloads/DownloadsPanel.svelte";
  import BatteryContextView from "../modules/battery/BatteryContextView.svelte";
  import { contextEngine, type ContextCandidate } from "../app/state/context-engine";
  import { batteryStore } from "../modules/battery/battery-store.svelte";
  import { settingsStore } from "../settings/settings-store.svelte";
  import IconTimer from "../icons/IconTimer.svelte";
  import IconApp from "../icons/IconApp.svelte";
  import IconClipboard from "../icons/IconClipboard.svelte";
  import IconDownload from "../icons/IconDownload.svelte";
  import IconBattery from "../icons/IconBattery.svelte";
  import IconSettings from "../icons/IconSettings.svelte";
  import IconMoreHorizontal from "../icons/IconMoreHorizontal.svelte";
  import IconChevronLeft from "../icons/IconChevronLeft.svelte";
  import IconChevronRight from "../icons/IconChevronRight.svelte";
  import type { NotchShellState } from "../app/state/shell-state";
  let { shellState, onStateChange, onback, oncollapse, pickerReplaceId }: {
    shellState: NotchShellState;
    pickerReplaceId?: string;
    onStateChange: (state: NotchShellState) => void;
    onback: () => void;
    oncollapse: () => void;
  } = $props();
  let candidate = $state<ContextCandidate>(contextEngine.current);
  onMount(() => contextEngine.subscribe((value) => { candidate = value; }));
  const isHome = $derived(shellState === "expanded");
  const pages: NotchShellState[] = ["apps", "appPicker", "focus", "capture", "clipboard", "downloads", "battery", "utilities"];
  const visited = new SvelteSet<NotchShellState>();
  const titles: Record<string, string> = { apps: "Apps", appPicker: "Installed apps", focus: "Focus", capture: "Screenshot", clipboard: "Clipboard", downloads: "Downloads", battery: "Battery", utilities: "More tools" };
  $effect(() => { if (pages.includes(shellState)) visited.add(shellState); });
  function toggle(state: NotchShellState) { onStateChange(shellState === state ? "expanded" : state); }
</script>

<div class="expanded-layout">
  <header class="companion-header">
    {#if !isHome}<button type="button" class="icon-button back" onclick={onback} aria-label="Back" title="Back (Esc)" data-focus="back"><IconChevronLeft size={16} /></button>{/if}
    {#if settingsStore.state.mascot.enabled && !["compact", "settings"].includes(shellState)}<MascotCanvas size={40} interactive />{/if}
    <div class="header-summary" aria-live={candidate.priority <= 1 ? "polite" : "off"}><ActivitySummary compact={!isHome} /></div>
    {#if !isHome}<MusicModule mini />{/if}
    <button type="button" disabled={captureStore.busy} class="icon-button collapse" onclick={oncollapse} aria-label="Collapse notch" title="Collapse (Esc)" data-focus="collapse"><span aria-hidden="true">−</span></button>
  </header>

  <div class="home-media" hidden={!isHome}><MusicModule /></div>
  <div class="drawer-body" hidden={isHome}>
    {#each pages as page}
      <div class="drawer-page" hidden={shellState !== page} role="region" aria-label={titles[page]} tabindex="-1" data-panel-focus={shellState === page ? "" : undefined}>
        {#if visited.has(page)}
          {#if page === "apps"}<AppsPanel />
          {:else if page === "appPicker"}<InstalledAppsPanel replaceId={pickerReplaceId} />
          {:else if page === "focus"}<FocusTimerPanel />
          {:else if page === "capture"}<CapturePanel />
          {:else if page === "clipboard"}<ClipboardPanel />
          {:else if page === "downloads"}<DownloadsPanel />
          {:else if page === "battery"}<BatteryContextView />
          {:else if page === "utilities"}
            <h2>More tools</h2>
            <div class="utility-list">
              <button type="button" data-focus="clipboard" onclick={() => onStateChange("clipboard")}><IconClipboard size={16} /><span>Clipboard</span><IconChevronRight size={16} /></button>
              <button type="button" data-focus="downloads" onclick={() => onStateChange("downloads")}><IconDownload size={16} /><span>Downloads</span><IconChevronRight size={16} /></button>
              {#if batteryStore.hasBattery}<button type="button" data-focus="battery" onclick={() => onStateChange("battery")}><IconBattery size={16} /><span>Battery</span><span class="utility-value">{batteryStore.formattedPercent}</span><IconChevronRight size={16} /></button>{/if}
              <button type="button" data-focus="open-settings" onclick={() => onStateChange("settings")}><IconSettings size={16} /><span>Settings</span><IconChevronRight size={16} /></button>
            </div>
          {/if}
        {/if}
      </div>
    {/each}
  </div>

  <nav class="primary-nav" aria-label="Notch tools">
    {#if settingsStore.state.apps.quickDockEnabled}<button type="button" disabled={captureStore.busy} class:active={["apps", "appPicker"].includes(shellState)} aria-pressed={["apps", "appPicker"].includes(shellState)} onclick={() => toggle("apps")} data-focus="apps"><IconApp size={16} /><span>Apps</span></button>{/if}
    <button type="button" disabled={captureStore.busy} class:active={shellState === "focus"} aria-pressed={shellState === "focus"} onclick={() => toggle("focus")} data-focus="focus"><IconTimer size={16} /><span>Focus</span></button>
    <button type="button" disabled={captureStore.busy} class:active={shellState === "capture"} aria-pressed={shellState === "capture"} onclick={() => toggle("capture")} data-focus="capture"><IconCamera size={16} /><span>Capture</span></button>
    <button type="button" disabled={captureStore.busy} class:active={!["apps", "appPicker", "capture", "focus", "expanded"].includes(shellState)} aria-pressed={!["apps", "appPicker", "capture", "focus", "expanded"].includes(shellState)} onclick={() => toggle("utilities")} data-focus="more"><IconMoreHorizontal size={16} /><span>More</span></button>
  </nav>
</div>

<style>
  .expanded-layout { display: flex; flex-direction: column; width: 100%; height: 100%; min-height: 0; }
  .companion-header { display: flex; align-items: center; gap: 8px; padding: 8px 16px; min-height: 56px; flex: none; }
  .header-summary { min-width: 0; flex: 1; }
  .icon-button { display: flex; align-items: center; justify-content: center; width: 32px; height: 32px; flex: none; border-radius: 8px; color: var(--text-secondary); }
  .icon-button:hover { background: var(--surface-hover); color: var(--text-primary); }
  .collapse span { font-size: 20px; line-height: 1; }
  .home-media { padding: 0 20px 8px; display: flex; align-items: center; min-height: 0; flex: 1; }
  .drawer-body { flex: 1; min-height: 0; overflow: hidden; padding: 0 16px 8px; }
  .drawer-page { height: 100%; overflow-y: auto; overflow-x: hidden; outline: none; }
  [hidden] { display: none !important; }
  .drawer-page:focus-visible { outline: 2px solid var(--accent-focus); outline-offset: -2px; border-radius: 8px; }
  h2 { font-size: 14px; font-weight: 600; margin: 8px 8px 6px; }
  .utility-list { display: grid; }
  .utility-list button { display: flex; align-items: center; gap: 12px; min-height: 40px; padding: 4px 8px; border-radius: 8px; color: var(--text-secondary); text-align: left; }
  .utility-list button:hover { background: var(--surface-hover); color: var(--text-primary); }
  .utility-list button > span:first-of-type { flex: 1; color: var(--text-primary); }
  .utility-value { font-size: 12px; font-variant-numeric: tabular-nums; }
  .primary-nav { display: flex; align-items: center; justify-content: center; gap: 4px; border-top: 1px solid var(--border-subtle); padding: 4px 16px 8px; flex: none; }
  .primary-nav button { display: flex; align-items: center; justify-content: center; gap: 8px; min-height: 32px; padding: 4px 12px; font-size: 12px; font-weight: 500; color: var(--text-secondary); border-radius: 8px; }
  .primary-nav button:hover, .primary-nav button.active { color: var(--accent-primary); background: var(--accent-muted); }
  @media (max-width: 380px) { .companion-header { padding-inline: 8px; gap: 4px; } .primary-nav { gap: 4px; padding-inline: 4px; } .primary-nav button { gap: 4px; padding-inline: 6px; } }
</style>
