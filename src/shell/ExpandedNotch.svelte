<script lang="ts">
  import { onMount } from "svelte";
  import MascotCanvas from "../mascot/MascotCanvas.svelte";
  import ActivitySummary from "./ActivitySummary.svelte";
  import MusicModule from "../modules/music/MusicModule.svelte";
  import AppsPanel from "../modules/launcher/AppsPanel.svelte";
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
  let { shellState, onStateChange, onback, oncollapse }: {
    shellState: NotchShellState;
    onStateChange: (state: NotchShellState) => void;
    onback: () => void;
    oncollapse: () => void;
  } = $props();
  let candidate = $state<ContextCandidate>(contextEngine.current);
  onMount(() => contextEngine.subscribe((value) => { candidate = value; }));
  const isHome = $derived(shellState === "expanded");
  const panelTitle = $derived(shellState === "utilities" ? "More tools" : shellState === "focus" ? "Focus" : shellState.charAt(0).toUpperCase() + shellState.slice(1));
  function toggle(state: NotchShellState) { onStateChange(shellState === state ? "expanded" : state); }
</script>

<div class="expanded-layout">
  <header class="companion-header">
    {#if !isHome}<button type="button" class="icon-button back" onclick={onback} aria-label="Back to home" title="Back to home (Esc)"><IconChevronLeft size={16} /></button>{/if}
    {#if settingsStore.state.mascot.enabled}<MascotCanvas size={40} interactive />{/if}
    <div class="header-summary" aria-live={candidate.priority <= 1 ? "polite" : "off"}><ActivitySummary compact={!isHome} /></div>
    {#if !isHome}<MusicModule mini />{/if}
    {#if isHome}<button type="button" class="icon-button collapse" onclick={oncollapse} aria-label="Collapse notch" title="Collapse (Esc)" data-focus="collapse"><span aria-hidden="true">−</span></button>{/if}
  </header>

  {#if isHome}
    <div class="home-media"><MusicModule /></div>
  {:else}
    <div class="drawer-body" role="region" aria-label={panelTitle} tabindex="-1" data-panel-focus>
      {#if shellState === "apps"}<AppsPanel onclose={onback} />
      {:else if shellState === "focus"}<FocusTimerPanel onclose={onback} />
      {:else if shellState === "clipboard"}<ClipboardPanel onclose={onback} />
      {:else if shellState === "downloads"}<DownloadsPanel onclose={onback} />
      {:else if shellState === "battery"}<BatteryContextView onclose={onback} />
      {:else if shellState === "utilities"}
        <h2>More tools</h2>
        <div class="utility-list">
          <button type="button" onclick={() => onStateChange("clipboard")}><IconClipboard size={16} /><span>Clipboard</span><IconChevronRight size={16} /></button>
          <button type="button" onclick={() => onStateChange("downloads")}><IconDownload size={16} /><span>Downloads</span><IconChevronRight size={16} /></button>
          {#if batteryStore.hasBattery}<button type="button" onclick={() => onStateChange("battery")}><IconBattery size={16} /><span>Battery</span><span class="utility-value">{batteryStore.formattedPercent}</span><IconChevronRight size={16} /></button>{/if}
          <button type="button" onclick={() => onStateChange("settings")}><IconSettings size={16} /><span>Settings</span><IconChevronRight size={16} /></button>
        </div>
      {/if}
    </div>
  {/if}

  <nav class="primary-nav" aria-label="Notch tools">
    {#if settingsStore.state.apps.quickDockEnabled}<button type="button" class:active={shellState === "apps"} aria-pressed={shellState === "apps"} onclick={() => toggle("apps")} data-focus="apps"><IconApp size={16} /><span>Apps</span></button>{/if}
    <button type="button" class:active={shellState === "focus"} aria-pressed={shellState === "focus"} onclick={() => toggle("focus")} data-focus="focus"><IconTimer size={16} /><span>Focus</span></button>
    <button type="button" class:active={!["apps", "focus", "expanded"].includes(shellState)} aria-pressed={!["apps", "focus", "expanded"].includes(shellState)} onclick={() => toggle("utilities")} data-focus="more"><IconMoreHorizontal size={16} /><span>More</span></button>
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
  .drawer-body { flex: 1; min-height: 0; overflow-y: auto; overflow-x: hidden; padding: 0 16px 8px; outline: none; animation: content-in var(--motion-context) var(--ease-natural); }
  .drawer-body:focus-visible { outline: 2px solid var(--accent-focus); outline-offset: -2px; border-radius: 8px; }
  h2 { font-size: 14px; font-weight: 600; margin: 8px 8px 6px; }
  .utility-list { display: grid; }
  .utility-list button { display: flex; align-items: center; gap: 12px; min-height: 40px; padding: 4px 8px; border-radius: 8px; color: var(--text-secondary); text-align: left; }
  .utility-list button:hover { background: var(--surface-hover); color: var(--text-primary); }
  .utility-list button > span:first-of-type { flex: 1; color: var(--text-primary); }
  .utility-value { font-size: 12px; font-variant-numeric: tabular-nums; }
  .primary-nav { display: flex; align-items: center; justify-content: center; gap: 16px; border-top: 1px solid var(--border-subtle); padding: 4px 16px 8px; flex: none; }
  .primary-nav button { display: flex; align-items: center; justify-content: center; gap: 8px; min-height: 32px; padding: 4px 12px; font-size: 12px; font-weight: 500; color: var(--text-secondary); border-radius: 8px; }
  .primary-nav button:hover, .primary-nav button.active { color: var(--text-primary); background: var(--surface-hover); }
  @keyframes content-in { from { opacity: 0; transform: translateY(4px); } to { opacity: 1; transform: translateY(0); } }
  @media (max-width: 380px) { .companion-header { padding-inline: 8px; gap: 4px; } .primary-nav { gap: 4px; padding-inline: 4px; } }
</style>
