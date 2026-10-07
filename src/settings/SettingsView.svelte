<script lang="ts">
  import { onMount } from "svelte";
  import { settingsStore } from "./settings-store.svelte";
  import type { SettingsSection } from "./types";
  import GeneralSection from "./sections/GeneralSection.svelte";
  import AppearanceSection from "./sections/AppearanceSection.svelte";
  import MascotSection from "./sections/MascotSection.svelte";
  import MusicSection from "./sections/MusicSection.svelte";
  import TimerSection from "./sections/TimerSection.svelte";
  import ClipboardSection from "./sections/ClipboardSection.svelte";
  import DownloadsSection from "./sections/DownloadsSection.svelte";
  import AppsSection from "./sections/AppsSection.svelte";
  import PerformanceSection from "./sections/PerformanceSection.svelte";

  import IconSettings from "../icons/IconSettings.svelte";
  import IconPalette from "../icons/IconPalette.svelte";
  import IconSmile from "../icons/IconSmile.svelte";
  import IconMusic from "../icons/IconMusic.svelte";
  import IconTimer from "../icons/IconTimer.svelte";
  import IconClipboard from "../icons/IconClipboard.svelte";
  import IconDownload from "../icons/IconDownload.svelte";
  import IconApp from "../icons/IconApp.svelte";
  import IconGauge from "../icons/IconGauge.svelte";
  import IconRotateCcw from "../icons/IconRotateCcw.svelte";
  import IconX from "../icons/IconX.svelte";

  interface Props {
    onclose: () => void;
  }

  let { onclose }: Props = $props();

  const activeSection = $derived(settingsStore.activeSection);
  const isSaving = $derived(settingsStore.isSaving);

  const TABS: Array<{ id: SettingsSection; label: string; component: any; icon: any }> = [
    { id: "general", label: "General", component: GeneralSection, icon: IconSettings },
    { id: "appearance", label: "Look", component: AppearanceSection, icon: IconPalette },
    { id: "mascot", label: "Mascot", component: MascotSection, icon: IconSmile },
    { id: "music", label: "Music", component: MusicSection, icon: IconMusic },
    { id: "timer", label: "Timer", component: TimerSection, icon: IconTimer },
    { id: "clipboard", label: "Clip", component: ClipboardSection, icon: IconClipboard },
    { id: "downloads", label: "Downloads", component: DownloadsSection, icon: IconDownload },
    { id: "apps", label: "Apps", component: AppsSection, icon: IconApp },
    { id: "performance", label: "Perf", component: PerformanceSection, icon: IconGauge },
  ];

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      onclose();
    }
  }

  onMount(() => {
    settingsStore.init();
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  });
</script>

<div
  class="settings-modal"
  role="dialog"
  tabindex="-1"
  aria-modal="true"
  aria-label="Settings configuration panel"
  onclick={(e) => e.stopPropagation()}
  onkeydown={(e) => e.stopPropagation()}
>
  <!-- TOP HEADER -->
  <header class="settings-header">
    <div class="header-left">
      <IconSettings size={14} color="var(--accent-primary)" />
      <span class="header-title">SETTINGS</span>
      <span class="header-separator">·</span>
      <span class="header-section">{activeSection.toUpperCase()}</span>
    </div>

    <div class="header-right">
      {#if isSaving}
        <span class="saving-badge">Saving…</span>
      {/if}
      <button
        type="button"
        class="icon-action-btn"
        title="Reset all settings to defaults"
        aria-label="Reset settings to defaults"
        onclick={() => settingsStore.resetToDefaults()}
      >
        <IconRotateCcw size={12} color="var(--text-secondary)" />
        <span class="btn-text">Defaults</span>
      </button>

      <button
        type="button"
        class="icon-action-btn close-btn"
        title="Close settings (Esc)"
        aria-label="Close settings"
        onclick={onclose}
      >
        <IconX size={13} color="var(--text-primary)" />
        <span class="btn-text">Done</span>
      </button>
    </div>
  </header>

  <!-- NAVIGATION TAB STRIP -->
  <div class="settings-nav" role="tablist" aria-label="Settings categories">
    {#each TABS as tab}
      {@const TabIcon = tab.icon}
      <button
        type="button"
        role="tab"
        class="tab-btn"
        class:active={activeSection === tab.id}
        aria-selected={activeSection === tab.id}
        onclick={() => settingsStore.setSection(tab.id)}
      >
        <TabIcon size={12} color="currentColor" />
        <span class="tab-label">{tab.label}</span>
      </button>
    {/each}
  </div>

  <!-- ACTIVE SECTION CONTENT BODY -->
  <main class="settings-body">
    {#if activeSection === "general"}
      <GeneralSection />
    {:else if activeSection === "appearance"}
      <AppearanceSection />
    {:else if activeSection === "mascot"}
      <MascotSection />
    {:else if activeSection === "music"}
      <MusicSection />
    {:else if activeSection === "timer"}
      <TimerSection />
    {:else if activeSection === "clipboard"}
      <ClipboardSection />
    {:else if activeSection === "downloads"}
      <DownloadsSection />
    {:else if activeSection === "apps"}
      <AppsSection />
    {:else if activeSection === "performance"}
      <PerformanceSection />
    {/if}
  </main>
</div>

<style>
  .settings-modal {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    background-color: var(--surface-root);
    box-sizing: border-box;
    padding: var(--space-8) var(--space-12);
    user-select: none;
    overflow: hidden;
  }

  /* Header */
  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-bottom: 6px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .header-title {
    font-size: var(--font-size-tiny);
    font-weight: 700;
    letter-spacing: 0.6px;
    color: var(--text-tertiary);
  }

  .header-separator {
    color: var(--text-tertiary);
    font-size: var(--font-size-tiny);
  }

  .header-section {
    font-size: var(--font-size-tiny);
    font-weight: 600;
    color: var(--accent-primary);
    letter-spacing: 0.4px;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .saving-badge {
    font-size: var(--font-size-tiny);
    color: var(--text-tertiary);
    font-style: italic;
  }

  .icon-action-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: var(--surface-raised);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 2px 6px;
    color: var(--text-secondary);
    font-size: var(--font-size-tiny);
    font-weight: 500;
    cursor: pointer;
    transition: all var(--motion-micro) var(--ease-natural);
  }

  .icon-action-btn:hover {
    background: var(--surface-hover);
    color: var(--text-primary);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .close-btn {
    background: var(--surface-hover);
    color: var(--text-primary);
  }

  .close-btn:hover {
    background: var(--accent-muted);
    border-color: var(--accent-primary);
  }

  .btn-text {
    line-height: 1.2;
  }

  /* Nav Tabs */
  .settings-nav {
    display: flex;
    align-items: center;
    gap: 3px;
    padding: 6px 0;
    border-bottom: 1px solid var(--border-subtle);
    overflow-x: auto;
    scrollbar-width: none;
  }

  .settings-nav::-webkit-scrollbar {
    display: none;
  }

  .tab-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 6px;
    border-radius: 4px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-secondary);
    font-size: var(--font-size-tiny);
    font-weight: 500;
    cursor: pointer;
    white-space: nowrap;
    transition: all var(--motion-micro) var(--ease-natural);
    flex-shrink: 0;
  }

  .tab-btn:hover {
    color: var(--text-primary);
    background: var(--surface-hover);
  }

  .tab-btn.active {
    background: var(--surface-raised);
    color: var(--accent-primary);
    border-color: var(--border-strong);
    font-weight: 600;
  }

  .tab-label {
    line-height: 1.2;
  }

  /* Content Body */
  .settings-body {
    flex: 1;
    overflow-y: auto;
    padding-top: 8px;
    padding-bottom: 4px;
  }
</style>
