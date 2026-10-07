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

<div class="section-container" role="tabpanel" aria-label="Apps Settings">
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
        <IconPlus size={12} color="currentColor" />
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
              <IconTrash size={12} color="var(--text-tertiary)" />
            </button>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>

<style>
  .section-container {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-12);
    padding: 6px 10px;
    background: var(--surface-raised);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-control);
    transition: opacity var(--motion-micro) var(--ease-natural);
  }

  .setting-row.disabled {
    opacity: 0.5;
    pointer-events: none;
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .setting-label {
    font-size: var(--font-size-label);
    font-weight: 600;
    color: var(--text-primary);
    line-height: var(--line-height-tight);
  }

  .setting-desc {
    font-size: var(--font-size-tiny);
    color: var(--text-tertiary);
    line-height: var(--line-height-tight);
  }

  .slider-group {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .win-slider {
    width: 90px;
    accent-color: var(--accent-primary);
    cursor: pointer;
  }

  .slider-label {
    font-size: var(--font-size-meta);
    font-weight: 600;
    color: var(--text-secondary);
    min-width: 44px;
    text-align: right;
  }

  .apps-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px 10px;
    background: var(--surface-raised);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-control);
  }

  .apps-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .add-app-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    background: var(--surface-root);
    border: 1px solid var(--border-strong);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: var(--font-size-meta);
    font-weight: 500;
    cursor: pointer;
    transition: background-color var(--motion-micro) var(--ease-natural);
  }

  .add-app-btn:hover {
    background: var(--surface-hover);
  }

  .apps-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 80px;
    overflow-y: auto;
  }

  .app-item {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--surface-root);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 3px 8px;
    transition: opacity var(--motion-micro) var(--ease-natural);
  }

  .app-item.disabled {
    opacity: 0.5;
  }

  .app-icon {
    width: 16px;
    height: 16px;
    border-radius: 3px;
    object-fit: contain;
  }

  .app-name {
    flex: 1;
    min-width: 0;
    font-size: var(--font-size-meta);
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .remove-app-btn {
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 2px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .remove-app-btn:hover {
    background: rgba(239, 68, 68, 0.2);
  }

  .empty-apps {
    font-size: var(--font-size-tiny);
    color: var(--text-tertiary);
    padding: 4px 0;
  }

  .win-switch {
    width: 38px;
    height: 20px;
    border-radius: 9999px;
    background: var(--surface-hover);
    border: 1px solid var(--border-strong);
    padding: 2px;
    cursor: pointer;
    position: relative;
    transition: background-color var(--motion-micro) var(--ease-natural),
                border-color var(--motion-micro) var(--ease-natural);
    flex-shrink: 0;
  }

  .win-switch.mini {
    width: 28px;
    height: 16px;
  }

  .win-switch.checked {
    background: var(--accent-primary);
    border-color: var(--accent-primary);
  }

  .switch-thumb {
    display: block;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: white;
    transform: translateX(0);
    transition: transform var(--motion-micro) var(--ease-natural);
  }

  .switch-thumb.mini {
    width: 10px;
    height: 10px;
  }

  .win-switch.checked .switch-thumb {
    transform: translateX(18px);
  }

  .win-switch.checked .switch-thumb.mini {
    transform: translateX(12px);
  }
</style>
