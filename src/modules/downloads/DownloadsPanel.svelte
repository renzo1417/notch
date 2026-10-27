<script lang="ts">
  import { onMount } from "svelte";
  import { downloadsStore } from "./downloads-store.svelte";
  import IconDownload from "../../icons/IconDownload.svelte";
  import IconFolder from "../../icons/IconFolder.svelte";
  import IconPlus from "../../icons/IconPlus.svelte";
  import IconTrash from "../../icons/IconTrash.svelte";
  import IconX from "../../icons/IconX.svelte";

  interface Props {
    onclose: () => void;
  }

  let { onclose }: Props = $props();

  const active = $derived(downloadsStore.activeDownload);
  const completed = $derived(downloadsStore.recentCompleted);
  const folders = $derived(downloadsStore.watchedFolders);
  let showFolders = $state(false);

  onMount(() => {
    downloadsStore.init();
  });
</script>

<div class="tool-panel downloads-panel" aria-label="Downloads panel">
  <!-- Header -->
  <div class="panel-header">
    <div class="header-left">
      <span class="panel-title">{showFolders ? "Watched Folders" : "Downloads"}</span>
    </div>

    <div class="header-actions">
      <button
        type="button"
        class="header-toggle-btn"
        onclick={(e) => {
          e.stopPropagation();
          showFolders = !showFolders;
        }}
        title={showFolders ? "Show Downloads" : "Manage Watched Folders"}
        aria-label="Toggle folders view"
      >
        <IconFolder size={16} color="var(--text-secondary)" />
        <span>{showFolders ? "Status" : "Folders"}</span>
      </button>

      <button
        type="button"
        class="close-btn"
        onclick={(e) => {
          e.stopPropagation();
          onclose();
        }}
        title="Close Downloads (Esc)"
        aria-label="Close Downloads"
      >
        <IconX size={16} color="var(--text-tertiary)" />
      </button>
    </div>
  </div>

  <!-- Body -->
  <div class="panel-body">
    {#if showFolders}
      <div class="folders-list">
        {#each folders as folder (folder.id)}
          <div class="folder-row">
            <span class="folder-path" title={folder.path}>{folder.path}</span>
            {#if folder.isDefault}
              <span class="default-badge">Default</span>
            {:else}
              <button
                type="button"
                class="del-btn"
                onclick={(e) => {
                  e.stopPropagation();
                  downloadsStore.removeFolder(folder.id);
                }}
                title="Remove folder"
                aria-label="Remove folder"
              >
                <IconTrash size={16} color="var(--status-error)" />
              </button>
            {/if}
          </div>
        {/each}

        <button
          type="button"
          class="add-folder-btn"
          onclick={(e) => {
            e.stopPropagation();
            downloadsStore.pickAndAddFolder();
          }}
          title="Add watched folder"
        >
          <IconPlus size={16} color="var(--text-secondary)" />
          <span>Add Folder</span>
        </button>
      </div>
    {:else}
      <div class="status-content">
        {#if active}
          <div class="active-download-card">
            <div class="download-info">
              <IconDownload size={16} color="var(--status-info)" />
              <span class="file-name" title={active.fileName}>{active.fileName}</span>
            </div>
            {#if active.progressPercent !== undefined}
              <div class="progress-bar-wrap">
                <div class="progress-bar-fill" style="width: {active.progressPercent}%"></div>
              </div>
              <span class="progress-meta">{active.progressPercent}% · In progress</span>
            {:else}
              <span class="progress-meta">Downloading…</span>
            {/if}
          </div>
        {:else if completed}
          <div class="completed-download-card">
            <div class="download-info">
              <IconDownload size={16} color="var(--status-success)" />
              <span class="file-name" title={completed.fileName}>{completed.fileName}</span>
            </div>
            <span class="completed-meta">Download complete</span>
            <button
              type="button"
              class="open-folder-btn"
              onclick={(e) => {
                e.stopPropagation();
                downloadsStore.openFolder(completed?.filePath);
              }}
            >
              Open in Explorer
            </button>
          </div>
        {:else}
          <div class="empty-state">
            <span class="empty-title">Watching {folders.length} {folders.length === 1 ? "folder" : "folders"}</span>
            <span class="empty-meta">New downloads will automatically appear here.</span>
            <button
              type="button"
              class="open-folder-btn"
              onclick={(e) => {
                e.stopPropagation();
                downloadsStore.openFolder();
              }}
            >
              Open Downloads Folder
            </button>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
