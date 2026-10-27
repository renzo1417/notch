<script lang="ts">
  import { settingsStore } from "../settings-store.svelte";
  import { launcherStore } from "../../modules/launcher/launcher-store.svelte";
  import IconPlus from "../../icons/IconPlus.svelte";
  import IconTrash from "../../icons/IconTrash.svelte";
  import IconApp from "../../icons/IconApp.svelte";

  const apps = $derived(settingsStore.state.apps);
  const launcherApps = $derived(launcherStore.apps);

  function toggleQuickDock() {
    settingsStore.update("apps", { quickDockEnabled: !apps.quickDockEnabled });
  }

  function handleMaxPinnedChange(event: Event) {
    const target = event.target as HTMLInputElement;
    settingsStore.update("apps", { maxQuickPinned: Number(target.value) });
  }

  async function handleAddApp() {
    await launcherStore.pickAndAdd();
  }

  async function handleToggleApp(id: string, enabled: boolean) {
    await launcherStore.toggleAppEnabled(id, !enabled);
  }

  async function handleRemoveApp(id: string) {
    await launcherStore.removeById(id);
  }
</script>

<div class="section-container" role="group" aria-label="Apps Settings">
  <!-- Enable Quick Dock -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Expanded Apps Dock</span>
      <span class="setting-desc">Show quick application launcher row inside expanded notch</span>
    </div>
    <button
      type="button"
      class="win-switch"
      class:checked={apps.quickDockEnabled}
      role="switch"
      aria-checked={apps.quickDockEnabled}
      aria-label="Toggle Quick Dock"
      onclick={toggleQuickDock}
    >
      <span class="switch-thumb"></span>
    </button>
  </div>

  <!-- Max Pinned Before Overflow -->
  <div class="setting-row" class:disabled={!apps.quickDockEnabled}>
    <div class="setting-info">
      <span class="setting-label">Dock Pinned Limit</span>
      <span class="setting-desc">4–6 applications maximum before compact pagination</span>
    </div>
    <div class="slider-group">
      <input
        type="range"
        class="win-slider"
        min="4"
        max="6"
        step="1"
        disabled={!apps.quickDockEnabled}
        value={apps.maxQuickPinned}
        oninput={handleMaxPinnedChange}
        aria-label="Max pinned apps before overflow"
      />
      <span class="slider-label">{apps.maxQuickPinned} apps</span>
    </div>
  </div>

  <!-- Manage Apps Group -->
  <div class="apps-group">
    <div class="apps-header">
      <div class="setting-info">
        <span class="setting-label">Configured Applications ({launcherApps.length})</span>
        <span class="setting-desc">Native Windows executables or shortcuts</span>
      </div>
      <button
        type="button"
        class="add-app-btn"
        onclick={handleAddApp}
        title="Add application executable"
      >
        <IconPlus size={16} color="currentColor" />
        <span>Add App</span>
      </button>
    </div>

    <div class="apps-list">
      {#if launcherApps.length === 0}
        <div class="empty-apps">
          <span>No apps configured. Tap "Add App" to select an executable or shortcut.</span>
        </div>
      {:else}
        {#each launcherApps as app (app.id)}
          <div class="app-item" class:disabled={!app.enabled}>
            {#if app.iconDataUrl}
              <img src={app.iconDataUrl} alt="" class="app-icon" />
            {:else}
              <IconApp size={16} color="var(--accent-primary)" />
            {/if}
            <span class="app-name" title={app.name}>{app.name}</span>
            <button
              type="button"
              class="win-switch mini"
              class:checked={app.enabled}
              title={app.enabled ? "Disable app in dock" : "Enable app in dock"}
              aria-label="Toggle application enabled state"
              onclick={() => handleToggleApp(app.id, app.enabled)}
            >
              <span class="switch-thumb mini"></span>
            </button>
            <button
              type="button"
              class="remove-app-btn"
              title="Remove app"
              aria-label="Remove application"
              onclick={() => handleRemoveApp(app.id)}
            >
              <IconTrash size={16} color="var(--text-tertiary)" />
            </button>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>
