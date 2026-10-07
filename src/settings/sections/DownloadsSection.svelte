<script lang="ts">
  import { settingsStore } from "../settings-store.svelte";
  import { downloadsStore } from "../../modules/downloads/downloads-store.svelte";
  import IconPlus from "../../icons/IconPlus.svelte";
  import IconTrash from "../../icons/IconTrash.svelte";
  import IconFolder from "../../icons/IconFolder.svelte";

  const downloads = $derived(settingsStore.state.downloads);
  const folders = $derived(downloadsStore.watchedFolders);

  function toggleDownloads() {
    settingsStore.update("downloads", { enabled: !downloads.enabled });
  }

  function toggleAutoOpen() {
    settingsStore.update("downloads", { autoOpenCompleted: !downloads.autoOpenCompleted });
  }

  async function handleAddFolder() {
    await downloadsStore.pickAndAddFolder();
  }

  async function handleRemoveFolder(id: string) {
    await downloadsStore.removeFolder(id);
  }
</script>

<div class="section-container" role="tabpanel" aria-label="Downloads Settings">
  <!-- Enable Downloads -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Downloads Awareness</span>
      <span class="setting-desc">Filesystem event monitoring for browser download completion</span>
    </div>
    <button
      type="button"
      class="win-switch"
      class:checked={downloads.enabled}
      role="switch"
      aria-checked={downloads.enabled}
      aria-label="Toggle Downloads Awareness"
      onclick={toggleDownloads}
    >
      <span class="switch-thumb"></span>
    </button>
  </div>

  <!-- Auto Open Completed -->
  <div class="setting-row" class:disabled={!downloads.enabled}>
    <div class="setting-info">
      <span class="setting-label">Auto-Reveal Completed</span>
      <span class="setting-desc">Open File Explorer directory when a download finishes</span>
    </div>
    <button
      type="button"
      class="win-switch"
      class:checked={downloads.autoOpenCompleted}
      disabled={!downloads.enabled}
      role="switch"
      aria-checked={downloads.autoOpenCompleted}
      aria-label="Toggle Auto-Reveal Completed"
      onclick={toggleAutoOpen}
    >
      <span class="switch-thumb"></span>
    </button>
  </div>

  <!-- Watched Folders Header & List -->
  <div class="folder-group" class:disabled={!downloads.enabled}>
    <div class="folder-header">
      <div class="setting-info">
        <span class="setting-label">Watched Folders ({folders.length})</span>
        <span class="setting-desc">Directories actively monitored via Win32 ReadDirectoryChangesW</span>
      </div>
      <button
        type="button"
        class="add-folder-btn"
        disabled={!downloads.enabled}
        onclick={handleAddFolder}
        title="Add folder to watch list"
      >
        <IconPlus size={12} color="currentColor" />
        <span>Add Folder</span>
      </button>
    </div>

    <div class="folder-list">
      {#if folders.length === 0}
        <div class="empty-folders">
          <span>No watched folders configured. Default user Downloads folder will be used.</span>
        </div>
      {:else}
        {#each folders as folder (folder.id)}
          <div class="folder-item">
            <IconFolder size={14} color="var(--accent-primary)" />
            <span class="folder-path" title={folder.path}>{folder.path}</span>
            <button
              type="button"
              class="remove-folder-btn"
              title="Stop watching this folder"
              aria-label="Remove watched folder"
              onclick={() => handleRemoveFolder(folder.id)}
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

  .setting-row.disabled,
  .folder-group.disabled {
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

  .folder-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px 10px;
    background: var(--surface-raised);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-control);
  }

  .folder-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .add-folder-btn {
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

  .add-folder-btn:hover:not(:disabled) {
    background: var(--surface-hover);
  }

  .folder-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 80px;
    overflow-y: auto;
  }

  .folder-item {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--surface-root);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 4px 8px;
  }

  .folder-path {
    flex: 1;
    min-width: 0;
    font-size: var(--font-size-meta);
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .remove-folder-btn {
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 2px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .remove-folder-btn:hover {
    background: rgba(239, 68, 68, 0.2);
  }

  .empty-folders {
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

  .win-switch.checked .switch-thumb {
    transform: translateX(18px);
  }
</style>
