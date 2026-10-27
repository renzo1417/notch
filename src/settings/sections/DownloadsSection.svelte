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

<div class="section-container" role="group" aria-label="Downloads Settings">
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
        <IconPlus size={16} color="currentColor" />
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
            <IconFolder size={16} color="var(--accent-primary)" />
            <span class="folder-path" title={folder.path}>{folder.path}</span>
            <button
              type="button"
              class="remove-folder-btn"
              title="Stop watching this folder"
              aria-label="Remove watched folder"
              onclick={() => handleRemoveFolder(folder.id)}
            >
              <IconTrash size={16} color="var(--text-tertiary)" />
            </button>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>
