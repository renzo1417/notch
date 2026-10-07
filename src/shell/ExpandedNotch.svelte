<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import MascotCanvas from "../mascot/MascotCanvas.svelte";
  import MusicModule from "../modules/music/MusicModule.svelte";
  import CodingContextView from "../modules/coding/CodingContextView.svelte";
  import FocusTimerView from "../modules/timer/FocusTimerView.svelte";
  import NotificationContextView from "../modules/notifications/NotificationContextView.svelte";
  import LauncherDockView from "../modules/launcher/LauncherDockView.svelte";
  import ClipboardContextView from "../modules/clipboard/ClipboardContextView.svelte";
  import BatteryContextView from "../modules/battery/BatteryContextView.svelte";
  import DownloadContextView from "../modules/downloads/DownloadContextView.svelte";
  import { timerStore } from "../modules/timer/timer-store.svelte";
  import { launcherStore } from "../modules/launcher/launcher-store.svelte";
  import { clipboardStore } from "../modules/clipboard/clipboard-store.svelte";
  import { batteryStore } from "../modules/battery/battery-store.svelte";
  import { downloadsStore } from "../modules/downloads/downloads-store.svelte";
  import IconTimer from "../icons/IconTimer.svelte";
  import IconApp from "../icons/IconApp.svelte";
  import IconClipboard from "../icons/IconClipboard.svelte";
  import IconBattery from "../icons/IconBattery.svelte";
  import IconDownload from "../icons/IconDownload.svelte";
  import { contextEngine, type ContextCandidate } from "../app/state/context-engine";
  import type { MascotState } from "../mascot/types";

  interface Props {
    backendConnected: boolean;
  }

  let { backendConnected }: Props = $props();

  let mascotState = $state<MascotState>("idle");
  let activeCandidate = $state<ContextCandidate>(contextEngine.current);
  let unsubContext: (() => void) | null = null;

  function handleMascotStateChange(newState: MascotState) {
    mascotState = newState;
  }

  onMount(() => {
    unsubContext = contextEngine.subscribe((cand) => {
      activeCandidate = cand;
    });
  });

  onDestroy(() => {
    if (unsubContext) {
      unsubContext();
    }
  });
</script>

<div class="expanded-three-zone" aria-label="Expanded Notch View · Context: {activeCandidate.mode} · Companion state: {mascotState}">
  <!-- LEFT ZONE: Procedural Mascot (Pip) -->
  <div class="zone left-zone" aria-label="Companion zone">
    <MascotCanvas
      size={64}
      compact={false}
      interactive={true}
      onStateChange={handleMascotStateChange}
    />
  </div>

  <!-- CENTER ZONE: Context Surface -->
  <div class="zone center-zone" aria-label="Context surface: {activeCandidate.mode}">
    {#if activeCandidate.mode === "notification"}
      <NotificationContextView />
    {:else if activeCandidate.mode === "download"}
      <DownloadContextView />
    {:else if activeCandidate.mode === "launcher"}
      <LauncherDockView />
    {:else if activeCandidate.mode === "clipboard"}
      <ClipboardContextView />
    {:else if activeCandidate.mode === "battery"}
      <BatteryContextView />
    {:else if activeCandidate.mode === "timer"}
      <FocusTimerView />
    {:else if activeCandidate.mode === "coding"}
      <CodingContextView />
    {:else}
      <!-- Default Idle Surface -->
      <div class="context-container">
        <div class="context-header">
          <span class="context-category">DESKTOP COMPANION · {mascotState.toUpperCase()}</span>
          <div
            class="status-dot"
            class:connected={backendConnected}
            title={backendConnected ? "Native backend connected" : "Connecting to backend..."}
            aria-hidden="true"
          ></div>
        </div>
        <div class="context-body">
          <div class="idle-title-row">
            <span class="context-title" title="Windows Notch Active">Windows Notch Active</span>
            <div class="idle-quick-actions">
              <button
                type="button"
                class="quick-action-trigger"
                onclick={(e) => {
                  e.stopPropagation();
                  launcherStore.openLauncherSurface();
                }}
                title="Open Apps Dock"
                aria-label="Open Apps Dock"
              >
                <IconApp size={10} color="var(--text-secondary)" />
                <span>Apps</span>
              </button>
              <button
                type="button"
                class="quick-action-trigger"
                onclick={(e) => {
                  e.stopPropagation();
                  timerStore.openTimerSurface();
                }}
                title="Open Focus Timer"
                aria-label="Open Focus Timer"
              >
                <IconTimer size={10} color="var(--text-secondary)" />
                <span>Focus</span>
              </button>
              <button
                type="button"
                class="quick-action-trigger"
                onclick={(e) => {
                  e.stopPropagation();
                  clipboardStore.openClipboardSurface();
                }}
                title="Open Clipboard History"
                aria-label="Open Clipboard History"
              >
                <IconClipboard size={10} color="var(--text-secondary)" />
                <span>Clip</span>
              </button>
              <button
                type="button"
                class="quick-action-trigger"
                class:low-battery={batteryStore.isLowBattery}
                onclick={(e) => {
                  e.stopPropagation();
                  batteryStore.openBatterySurface();
                }}
                title="Open Battery Details ({batteryStore.statusSummary})"
                aria-label="Open Battery Details"
              >
                <IconBattery
                  size={10}
                  color={batteryStore.isLowBattery ? "#ef4444" : "var(--text-secondary)"}
                  charging={batteryStore.isCharging}
                  level={batteryStore.percentage}
                />
                <span>{batteryStore.formattedPercent}</span>
              </button>
              <button
                type="button"
                class="quick-action-trigger"
                class:active-dl={downloadsStore.hasActiveDownload}
                onclick={(e) => {
                  e.stopPropagation();
                  downloadsStore.openDownloadSurface();
                }}
                title={downloadsStore.hasActiveDownload ? `Downloading: ${downloadsStore.activeDownload?.fileName}` : "Downloads & Watched Folders"}
                aria-label="Open Downloads Surface"
              >
                <IconDownload
                  size={10}
                  color={downloadsStore.hasActiveDownload ? "#38bdf8" : "var(--text-secondary)"}
                />
                <span>DL</span>
              </button>
            </div>
          </div>
          <span class="context-meta" title="Quiet desktop companion layer">Quiet top-center companion layer</span>
        </div>
      </div>
    {/if}
  </div>

  <!-- RIGHT ZONE: Media Module (Anchored ~176px fixed geometry) -->
  <div class="zone right-zone" aria-label="Music player zone">
    <MusicModule />
  </div>

</div>

<style>
  .expanded-three-zone {
    display: flex;
    align-items: center;
    width: 100%;
    height: 100%;
    user-select: none;
    box-sizing: border-box;
  }

  .zone {
    height: 100%;
    display: flex;
    align-items: center;
    box-sizing: border-box;
  }

  /* LEFT: Mascot Zone (18-20% visual balance ~ 96px) */
  .left-zone {
    width: 96px;
    justify-content: center;
    flex-shrink: 0;
    padding: 0 var(--space-8);
  }

  /* CENTER: Context Surface (42-46% visual balance ~ 248px) */
  .center-zone {
    flex: 1;
    min-width: 0; /* Enable text truncation */
    padding: 0 var(--space-12);
    border-left: 1px solid var(--border-subtle);
    border-right: 1px solid var(--border-subtle);
    justify-content: flex-start;
  }

  .context-container {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 2px;
    width: 100%;
    min-width: 0;
  }

  .context-header {
    display: flex;
    align-items: center;
    gap: var(--space-6);
  }

  .context-category {
    font-size: var(--font-size-tiny);
    font-weight: 600;
    letter-spacing: 0.6px;
    color: var(--text-tertiary);
    text-transform: uppercase;
    line-height: var(--line-height-tight);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .status-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background-color: var(--text-tertiary);
    transition: background-color var(--motion-micro) var(--ease-natural);
    flex-shrink: 0;
  }

  .status-dot.connected {
    background-color: #22c55e;
  }

  .context-body {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }

  .idle-title-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-8);
    min-width: 0;
  }

  .context-title {
    font-size: var(--font-size-body);
    font-weight: 600;
    color: var(--text-primary);
    line-height: var(--line-height-tight);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .idle-quick-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .quick-action-trigger {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    background: var(--surface-raised);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 1px 5px;
    font-size: var(--font-size-tiny);
    font-weight: 500;
    color: var(--text-secondary);
    cursor: pointer;
    line-height: 1.2;
    transition: all var(--motion-micro) var(--ease-natural);
    flex-shrink: 0;
  }

  .quick-action-trigger:hover {
    background-color: var(--surface-hover);
    color: var(--text-primary);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .quick-action-trigger.low-battery {
    border-color: rgba(239, 68, 68, 0.4);
    color: #ef4444;
  }

  .quick-action-trigger.active-dl {
    border-color: rgba(56, 189, 248, 0.4);
    color: #38bdf8;
  }

  .context-meta {
    font-size: var(--font-size-meta);
    color: var(--text-secondary);
    line-height: var(--line-height-tight);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* RIGHT: Music Module Zone (~176px stable geometry) */
  .right-zone {
    width: 176px;
    flex-shrink: 0;
    justify-content: flex-end;
  }
</style>
