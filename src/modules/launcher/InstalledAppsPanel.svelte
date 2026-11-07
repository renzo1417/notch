<script lang="ts">
  import { onMount } from "svelte";
  import { launcherStore } from "./launcher-store.svelte";
  import { searchInstalledApps, type InstalledApp } from "./types";
  import InstalledAppIcon from "./InstalledAppIcon.svelte";
  let { replaceId }: { replaceId?: string } = $props();
  let limit = $state(40);
  const matches = $derived(searchInstalledApps(launcherStore.catalog, launcherStore.catalogQuery));
  $effect(() => { launcherStore.catalogQuery; limit = 40; });
  onMount(() => { void launcherStore.init(); void launcherStore.loadCatalog(); });
  async function add(app: InstalledApp) {
    await launcherStore.pinInstalledApp(app, replaceId);
  }
  async function browse() {
    launcherStore.catalogError = "";
    await launcherStore.pickAndAdd(replaceId);
  }
</script>

<section class="installed-apps tool-panel" aria-label={replaceId ? "Replace app" : "Installed apps"}>
  <div class="catalog-heading"><h2>{replaceId ? "Replace app" : "Installed apps"}</h2><button type="button" onclick={() => launcherStore.loadCatalog(true)} disabled={launcherStore.catalogLoading || !!launcherStore.catalogBusyId || launcherStore.browseBusy}>Refresh</button></div>
  <label class="search"><span class="sr-only">Search installed apps</span><input type="search" placeholder="Search installed apps" bind:value={launcherStore.catalogQuery} data-autofocus /></label>
  <div class="catalog-status" role="status">{launcherStore.catalogLoading ? "Finding installed apps…" : launcherStore.catalogNotice}</div>
  {#if launcherStore.catalogError}<p class="catalog-error" role="alert">{launcherStore.catalogError}</p>{/if}
  {#each launcherStore.catalogWarnings as warning}<p class="catalog-warning">{warning}</p>{/each}
  {#if !launcherStore.catalogLoading && matches.length === 0}<p class="empty">{launcherStore.catalogQuery ? "No matching apps. Try another name or Browse below." : "No apps listed. Try Refresh or Browse below."}</p>{/if}
  <ul class="catalog-list" aria-label="Installed applications">
    {#each matches.slice(0, limit) as app (app.id)}
      {@const added = launcherStore.isInstalledAppAdded(app)}
      <li><InstalledAppIcon id={app.id} /><div class="catalog-label"><span title={app.name}>{app.name}</span><small>{app.source}</small></div><button type="button" disabled={added || !!launcherStore.catalogBusyId || launcherStore.browseBusy} onclick={() => add(app)} aria-label={`${added ? "Already added" : replaceId ? "Replace with" : "Add"} ${app.name}`}>{launcherStore.catalogBusyId === app.id ? "Adding…" : added ? "Added" : replaceId ? "Replace" : "Add"}</button></li>
    {/each}
  </ul>
  {#if matches.length > limit}<button type="button" class="show-more" onclick={() => { limit += 40; }}>Show more ({matches.length - limit})</button>{/if}
  <div class="browse-row"><button type="button" onclick={browse} disabled={!!launcherStore.catalogBusyId || launcherStore.browseBusy}>Browse executable or shortcut…</button><p>For portable apps or apps missing from this list.</p></div>
</section>

<style>
  .installed-apps { min-height: 0; }
  .catalog-heading { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  h2 { font-size: 14px; font-weight: 600; margin: 4px 0; }
  button { padding: 4px 8px; min-height: 32px; border-radius: 8px; font-size: 12px; color: var(--accent-primary); flex: none; }
  .search { display: block; padding-block: 4px; }
  input { width: 100%; min-height: 36px; border: 1px solid var(--border-strong); border-radius: 8px; background: var(--surface-raised); padding: 6px 10px; color: var(--text-primary); font: inherit; font-size: 13px; }
  input::placeholder { color: var(--text-secondary); }
  .catalog-status:empty { display: none; }
  .catalog-status, .catalog-error, .catalog-warning, .empty { font-size: 12px; line-height: 1.4; margin: 6px 0; }
  .catalog-status { color: var(--text-secondary); }
  .catalog-error { color: var(--status-error); }
  .catalog-warning { color: var(--status-warning); }
  .empty { color: var(--text-secondary); padding-block: 8px; }
  ul { list-style: none; margin: 0; padding: 0; }
  li { display: flex; align-items: center; gap: 10px; padding: 6px 2px; min-height: 44px; border-bottom: 1px solid var(--border-subtle); }
  .catalog-label { min-width: 0; flex: 1; display: grid; gap: 2px; font-size: 13px; }
  .catalog-label > span { overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
  small, .browse-row p { font-size: 12px; color: var(--text-secondary); }
  .browse-row { padding-block: 8px; }
  .browse-row p { margin: 4px 8px 0; line-height: 1.4; }
</style>
