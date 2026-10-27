<script lang="ts">
  import { downloadsStore } from "./downloads-store.svelte";
  import IconDownload from "../../icons/IconDownload.svelte";
  import IconFolder from "../../icons/IconFolder.svelte";
  import IconPlus from "../../icons/IconPlus.svelte";
  import IconTrash from "../../icons/IconTrash.svelte";
  import IconX from "../../icons/IconX.svelte";

  const active = $derived(downloadsStore.activeDownload);
  const completed = $derived(downloadsStore.recentCompleted);
  const folders = $derived(downloadsStore.watchedFolders);
  const showManager = $derived(downloadsStore.showFolderManager);

  const displayFileName = $derived(
    active?.fileName || completed?.fileName || "Download Awareness"
  );

  const progressLabel = $derived(
    active
      ? active.progressPercent !== undefined
        ? `Downloading · ${active.progressPercent}%`
        : "Downloading…"
      : completed
      ? "Download complete"
      : `${folders.length} watched ${folders.length === 1 ? "folder" : "folders"}`
  );
</script>

<div class="download-context" aria-label="Downloads Surface: {displayFileName} · {progressLabel}">
  <div class="context-header">
    <div class="header-left">
      <span class="context-category">
        {showManager
          ? "WATCHED FOLDERS"
          : active
          ? "DOWNLOADING"
          : completed
          ? "DOWNLOAD COMPLETE"
          : "DOWNLOADS"}
      </span>
      <div
        class="status-indicator"
        class:active={!!active}
        class:complete={!active && !!completed}
        aria-hidden="true"
      >
        <IconDownload
          size={10}
          color={active ? "var(--status-info)" : completed ? "var(--status-success)" : "var(--text-tertiary)"}
        />
      </div>
    </div>

    <div class="header-actions">
      <button
        type="button"
        class="header-btn"
        onclick={(e) => {
          e.stopPropagation();
          downloadsStore.toggleFolderManager();
        }}
        title={showManager ? "Back to download status" : "Manage watched folders"}
        aria-label={showManager ? "Back to download status" : "Manage watched folders"}
      >
        <IconFolder size={11} color="var(--text-secondary)" />
        <span>{showManager ? "Status" : "Folders"}</span>
      </button>

      <button
        type="button"
        class="header-btn icon-only"
        onclick={(e) => {
          e.stopPropagation();
          downloadsStore.closeDownloadSurface();
        }}
        title="Dismiss surface"
        aria-label="Dismiss surface"
      >
        <IconX size={10} color="var(--text-secondary)" />
      </button>
    </div>
  </div>

  <div class="context-body">
    {#if showManager}
      <!-- Watched Folders Management View -->
      <div class="folders-view">
        <div class="folders-list">
          {#each folders as folder (folder.id)}
            <div class="folder-row" title={folder.path}>
              <span class="folder-path">{folder.path}</span>
              <div class="folder-meta-wrap">
                {#if folder.isDefault}
                  <span class="default-badge">Default</span>
                {:else}
                  <button
                    type="button"
                    class="folder-del-btn"
                    onclick={(e) => {
                      e.stopPropagation();
                      downloadsStore.removeFolder(folder.id);
                    }}
                    title="Remove watched folder"
                    aria-label="Remove watched folder"
                  >
                    <IconTrash size={10} color="var(--text-tertiary)" />
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        </div>

        <button
          type="button"
          class="add-folder-btn"
          onclick={(e) => {
            e.stopPropagation();
            downloadsStore.pickAndAddFolder();
          }}
          title="Select and add a folder to watch"
        >
          <IconPlus size={10} color="var(--text-primary)" />
          <span>Add Folder</span>
        </button>
      </div>
    {:else}
      <!-- Standard Download / Completion Status View -->
      <div class="status-view">
        <div class="status-info">
          <span class="context-title" title={displayFileName}>{displayFileName}</span>
          <span class="context-meta" title={progressLabel}>{progressLabel}</span>
        </div>

        <div class="action-buttons">
          <button
            type="button"
            class="open-folder-btn"
            onclick={(e) => {
              e.stopPropagation();
              downloadsStore.openFolder();
            }}
            title="Open folder in File Explorer"
            aria-label="Open download folder"
          >
            <IconFolder size={11} color="var(--text-primary)" />
            <span>Open folder</span>
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .download-context {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 2px;
    width: 100%;
    min-width: 0;
    user-select: none;
    box-sizing: border-box;
  }

  .context-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-6);
    min-width: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 5px;
    min-width: 0;
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

  .status-indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    flex-shrink: 0;
  }

  .status-indicator.active {
    animation: download-pulse 1.8s infinite ease-in-out;
  }

  @keyframes download-pulse {
    0%, 100% {
      opacity: 0.6;
      transform: translateY(0);
    }
    50% {
      opacity: 1;
      transform: translateY(1px);
    }
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 3px;
    flex-shrink: 0;
  }

  .header-btn {
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

  .header-btn:hover {
    background-color: var(--surface-hover);
    color: var(--text-primary);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .header-btn.icon-only {
    padding: 1px 3px;
  }

  .context-body {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .status-view {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-8);
    min-width: 0;
  }

  .status-info {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
    flex: 1;
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

  .context-meta {
    font-size: var(--font-size-meta);
    color: var(--text-secondary);
    line-height: var(--line-height-tight);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .action-buttons {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .open-folder-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: var(--surface-raised);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-control);
    padding: 2px 7px;
    font-size: var(--font-size-tiny);
    font-weight: 500;
    color: var(--text-primary);
    cursor: pointer;
    line-height: 1.3;
    transition: all var(--motion-micro) var(--ease-natural);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .open-folder-btn:hover {
    background: var(--surface-hover);
    border-color: rgba(255, 255, 255, 0.22);
  }

  /* Folders list view */
  .folders-view {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    min-width: 0;
  }

  .folders-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 38px;
    overflow-y: auto;
    flex: 1;
    min-width: 0;
  }

  .folder-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 4px;
    padding: 1px 4px;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 3px;
    font-size: 10px;
    color: var(--text-secondary);
  }

  .folder-path {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    direction: rtl;
    text-align: left;
    max-width: 140px;
  }

  .folder-meta-wrap {
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  .default-badge {
    font-size: 9px;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .folder-del-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 1px;
    color: var(--text-tertiary);
  }

  .folder-del-btn:hover {
    color: var(--status-error);
  }

  .add-folder-btn {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    background: var(--surface-raised);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-control);
    padding: 2px 6px;
    font-size: var(--font-size-tiny);
    font-weight: 500;
    color: var(--text-primary);
    cursor: pointer;
    flex-shrink: 0;
    transition: all var(--motion-micro) var(--ease-natural);
  }

  .add-folder-btn:hover {
    background: var(--surface-hover);
    border-color: rgba(255, 255, 255, 0.22);
  }
</style>
