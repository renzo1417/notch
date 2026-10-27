<script lang="ts">
  import { tick } from "svelte";
  import { settingsStore } from "./settings-store.svelte";
  import GeneralSection from "./sections/GeneralSection.svelte";
  import AppearanceSection from "./sections/AppearanceSection.svelte";
  import MascotSection from "./sections/MascotSection.svelte";
  import MusicSection from "./sections/MusicSection.svelte";
  import TimerSection from "./sections/TimerSection.svelte";
  import ClipboardSection from "./sections/ClipboardSection.svelte";
  import DownloadsSection from "./sections/DownloadsSection.svelte";
  import AppsSection from "./sections/AppsSection.svelte";
  import PerformanceSection from "./sections/PerformanceSection.svelte";
  import IconChevronLeft from "../icons/IconChevronLeft.svelte";
  import IconRotateCcw from "../icons/IconRotateCcw.svelte";

  let { onclose }: { onclose: () => void } = $props();
  const groups = [
    { id: "appearance", label: "Appearance", sections: [{ label: "Appearance", component: AppearanceSection }, { label: "Pip", component: MascotSection }] },
    { id: "tools", label: "Tools", sections: [{ label: "Music", component: MusicSection }, { label: "Focus", component: TimerSection }, { label: "Clipboard", component: ClipboardSection }, { label: "Downloads", component: DownloadsSection }, { label: "Apps", component: AppsSection }] },
    { id: "system", label: "System", sections: [{ label: "General", component: GeneralSection }, { label: "Performance", component: PerformanceSection }] },
  ];
  let selected = $state(0);
  let dialog: HTMLDivElement;
  let body: HTMLElement;

  function select(index: number) {
    selected = index;
    if (body) body.scrollTop = 0;
  }

  async function categoryKey(event: KeyboardEvent) {
    let index = selected;
    if (event.key === "ArrowRight") index = (selected + 1) % groups.length;
    else if (event.key === "ArrowLeft") index = (selected + groups.length - 1) % groups.length;
    else if (event.key === "Home") index = 0;
    else if (event.key === "End") index = groups.length - 1;
    else return;
    event.preventDefault();
    select(index);
    await tick();
    dialog.querySelector<HTMLElement>(`#settings-tab-${groups[index].id}`)?.focus();
  }

  function trapFocus(event: KeyboardEvent) {
    if (event.key !== "Tab") return;
    const controls = [...dialog.querySelectorAll<HTMLElement>('button:not(:disabled), input:not(:disabled), select:not(:disabled), [tabindex="0"]')].filter((element) => element.offsetParent !== null && element.tabIndex >= 0);
    const first = controls[0], last = controls[controls.length - 1];
    if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus(); }
    else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first?.focus(); }
  }
</script>

<div bind:this={dialog} class="settings-modal" role="dialog" aria-modal="true" aria-labelledby="settings-title" tabindex="-1" onkeydown={trapFocus}>
  <header class="settings-header">
    <button type="button" class="settings-back" onclick={onclose} aria-label="Back to home" title="Back to home (Esc)" data-focus="settings"><IconChevronLeft size={16} /></button>
    <h1 id="settings-title">Settings</h1>
    <span class="save-feedback" role="status">{settingsStore.saveError || (settingsStore.isSaving ? "Saving…" : "")}</span>
    <button type="button" class="reset-settings" onclick={() => settingsStore.resetToDefaults()} title="Reset all settings to defaults"><IconRotateCcw size={16} /><span>Defaults</span></button>
  </header>
  <div class="settings-nav" role="tablist" aria-label="Settings categories">
    {#each groups as group, index}
      <button type="button" role="tab" id={`settings-tab-${group.id}`} aria-controls={`settings-panel-${group.id}`} aria-selected={selected === index} tabindex={selected === index ? 0 : -1} class:active={selected === index} onclick={() => select(index)} onkeydown={categoryKey}>{group.label}</button>
    {/each}
  </div>
  <div bind:this={body} class="settings-body" role="tabpanel" id={`settings-panel-${groups[selected].id}`} aria-labelledby={`settings-tab-${groups[selected].id}`} tabindex="0">
    {#each groups[selected].sections as section (section.label)}
      {@const Section = section.component}
      <section class="settings-section" aria-label={section.label}><h2>{section.label}</h2><Section /></section>
    {/each}
  </div>
</div>

<style>
  .settings-modal { display: flex; flex-direction: column; width: 100%; height: 100%; min-height: 0; outline: none; }
  .settings-header { display: flex; align-items: center; gap: 8px; padding: 12px 16px 8px; flex: none; }
  h1 { font-size: 14px; font-weight: 600; }
  .settings-back { display: flex; align-items: center; justify-content: center; width: 32px; height: 32px; border-radius: 8px; flex: none; }
  .save-feedback { flex: 1; text-align: right; color: var(--text-secondary); font-size: 12px; }
  .reset-settings { display: flex; align-items: center; gap: 8px; padding: 4px 8px; min-height: 32px; color: var(--text-secondary); font-size: 12px; border-radius: 8px; }
  .settings-back:hover, .reset-settings:hover { background: var(--surface-hover); color: var(--text-primary); }
  .settings-nav { display: flex; gap: 8px; padding: 0 20px 8px; border-bottom: 1px solid var(--border-subtle); flex: none; }
  .settings-nav button { flex: 1; min-height: 32px; border-radius: 8px; font-size: 13px; color: var(--text-secondary); }
  .settings-nav button.active, .settings-nav button:hover { background: var(--surface-hover); color: var(--text-primary); }
  .settings-body { flex: 1; min-height: 0; overflow-y: auto; overflow-x: hidden; padding: 8px 24px 20px; outline-offset: -3px; }
  .settings-section + .settings-section { margin-top: 20px; }
  h2 { font-size: 13px; font-weight: 600; padding: 12px 0 4px; color: var(--text-secondary); }
  @media (max-width: 480px) { .settings-body { padding-inline: 16px; } .reset-settings span { display: none; } }
</style>
