<script lang="ts">
  import { onMount } from "svelte";
  import { launcherStore } from "./launcher-store.svelte";
  import type { LaunchableApp } from "./types";
  import IconPlus from "../../icons/IconPlus.svelte";
  import IconApp from "../../icons/IconApp.svelte";
  import IconAlertCircle from "../../icons/IconAlertCircle.svelte";
  import IconTrash from "../../icons/IconTrash.svelte";
  import IconX from "../../icons/IconX.svelte";
  import { shellActions } from "../../app/state/shell-state";
  import IconMoreHorizontal from "../../icons/IconMoreHorizontal.svelte";
  let activeMenuAppId = $state<string | null>(null);
  const pinned = $derived(launcherStore.pinnedApps);
  const selectedApp = $derived(pinned.find((app) => app.id === activeMenuAppId));
  function launch(app: LaunchableApp) {
    activeMenuAppId = null;
    if (app.exists === false) launcherStore.brokenApp = app;
    else void launcherStore.launch(app);
  }
  function options(app: LaunchableApp, event?: MouseEvent) {
    event?.preventDefault();
    activeMenuAppId = activeMenuAppId === app.id ? null : app.id;
  }
  onMount(() => { void launcherStore.init(); });
</script>

<div class="tool-panel apps-panel">
  <div class="panel-header"><div class="header-left"><h2 class="panel-title">Apps</h2><span class="item-count">{pinned.length}</span></div></div>
  {#if launcherStore.brokenApp}
    <div class="broken-alert" role="alert">
      <div class="broken-text"><IconAlertCircle size={16} /><span title={launcherStore.brokenApp.name}>Missing: {launcherStore.brokenApp.name}</span></div>
      <button type="button" class="alert-action-btn" onclick={() => { if (launcherStore.brokenApp) shellActions.requestAppPicker(launcherStore.brokenApp.id); }}>Replace app</button>
      <button type="button" class="alert-action-btn danger" onclick={() => { if (launcherStore.brokenApp) void launcherStore.remove(launcherStore.brokenApp); }}>Remove</button>
    </div>
  {/if}
  <div class="grid-container">
    {#if pinned.length === 0}<div class="empty-state"><span class="empty-text">{launcherStore.isLoading ? "Loading apps…" : "Add your first app below."}</span></div>
    {:else}
      <div class="apps-grid" role="list">
        {#each pinned as app (app.id)}
          <div class="app-tile" role="listitem">
            <button type="button" class="app-card" class:broken={app.exists === false} class:launching={launcherStore.launchingAppId === app.id} onclick={() => launch(app)} oncontextmenu={(event) => options(app, event)} title={app.exists === false ? `${app.name} (File not found)` : `Launch ${app.name}`}>
              <span class="app-icon-wrap">{#if app.iconDataUrl}<img src={app.iconDataUrl} alt="" class="app-icon" />{:else}<IconApp size={24} />{/if}{#if app.exists === false}<span class="broken-dot" aria-hidden="true"></span>{/if}</span>
              <span class="app-name">{app.name}</span>
            </button>
            <button type="button" class="app-options" aria-label={`Options for ${app.name}`} aria-expanded={activeMenuAppId === app.id} onclick={() => options(app)}><IconMoreHorizontal size={16} /></button>
          </div>
        {/each}
      </div>
    {/if}
  </div>
  {#if selectedApp}
    <div class="app-options-row" role="group" aria-label={`Options for ${selectedApp.name}`}>
      <span class="app-name">{selectedApp.name}</span><button type="button" class="menu-item" onclick={() => { if (selectedApp) shellActions.requestAppPicker(selectedApp.id); activeMenuAppId = null; }}>Replace app</button><button type="button" class="menu-item danger" onclick={() => { if (selectedApp) void launcherStore.remove(selectedApp); activeMenuAppId = null; }}><IconTrash size={16} />Remove</button><button type="button" class="close-btn" aria-label="Close app options" onclick={() => { activeMenuAppId = null; }}><IconX size={16} /></button>
    </div>
  {/if}
  <div class="panel-footer"><button type="button" class="add-app-btn" data-focus="add-app" onclick={() => { activeMenuAppId = null; shellActions.requestAppPicker(); }}><IconPlus size={16} />Add app</button></div>
</div>
