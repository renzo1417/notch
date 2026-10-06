<script lang="ts">
  import { onMount } from "svelte";
  import { launcherStore, MAX_PINNED_BEFORE_OVERFLOW } from "./launcher-store.svelte";
  import type { LaunchableApp } from "./types";
  import IconPlus from "../../icons/IconPlus.svelte";
  import IconApp from "../../icons/IconApp.svelte";
  import IconTrash from "../../icons/IconTrash.svelte";
  import IconFolder from "../../icons/IconFolder.svelte";
  import IconAlertCircle from "../../icons/IconAlertCircle.svelte";
  import IconChevronLeft from "../../icons/IconChevronLeft.svelte";
  import IconChevronRight from "../../icons/IconChevronRight.svelte";
  import IconX from "../../icons/IconX.svelte";

  let pageIndex = $state(0);
  let isReorderMode = $state(false);

  const pinned = $derived(launcherStore.pinnedApps);
  const totalPages = $derived(Math.max(1, Math.ceil(pinned.length / MAX_PINNED_BEFORE_OVERFLOW)));
  const currentPageApps = $derived(
    pinned.slice(
      pageIndex * MAX_PINNED_BEFORE_OVERFLOW,
      (pageIndex + 1) * MAX_PINNED_BEFORE_OVERFLOW
    )
  );

  function handleAppClick(app: LaunchableApp, event: MouseEvent) {
    event.stopPropagation();
    if (isReorderMode) return;

    if (app.exists === false) {
      launcherStore.brokenApp = app;
    } else {
      launcherStore.launch(app);
    }
  }

  function nextPage(e: MouseEvent) {
    e.stopPropagation();
    if (pageIndex < totalPages - 1) {
      pageIndex += 1;
    }
  }

  function prevPage(e: MouseEvent) {
    e.stopPropagation();
    if (pageIndex > 0) {
      pageIndex -= 1;
    }
  }

  onMount(() => {
    launcherStore.init();
  });
</script>

<div class="dock-container" aria-label="Quick App Dock">
  {#if launcherStore.brokenApp}
    <!-- Broken App Resolution Surface (Locate or Remove) -->
    <div class="broken-view" role="alert" aria-label="Broken application options for {launcherStore.brokenApp.name}">
      <div class="context-header">
        <span class="context-category broken-title">APP NOT FOUND · {launcherStore.brokenApp.name}</span>
        <button
          type="button"
          class="icon-action-btn"
          onclick={(e) => {
            e.stopPropagation();
            launcherStore.brokenApp = null;
          }}
          title="Back to dock"
          aria-label="Back to dock"
        >
          <IconX size={10} color="var(--text-tertiary)" />
        </button>
      </div>

      <div class="broken-body">
        <span class="broken-path-label" title={launcherStore.brokenApp.executablePath}>
          Path missing or moved
        </span>
        <div class="broken-actions-row">
          <button
            type="button"
            class="action-pill primary"
            onclick={(e) => {
              e.stopPropagation();
              if (launcherStore.brokenApp) launcherStore.locate(launcherStore.brokenApp);
            }}
            title="Locate new application path"
            aria-label="Locate application"
          >
            <IconFolder size={11} color="var(--text-primary)" />
            <span>Locate</span>
          </button>

          <button
            type="button"
            class="action-pill danger"
            onclick={(e) => {
              e.stopPropagation();
              if (launcherStore.brokenApp) launcherStore.remove(launcherStore.brokenApp);
            }}
            title="Remove from dock"
            aria-label="Remove application"
          >
            <IconTrash size={11} color="#f87171" />
            <span>Remove</span>
          </button>
        </div>
      </div>
    </div>
  {:else}
    <!-- Normal Dock Surface -->
    <div class="context-header">
      <div class="header-left">
        <span class="context-category">APPS DOCK</span>
        {#if totalPages > 1}
          <span class="page-indicator">{pageIndex + 1}/{totalPages}</span>
        {/if}
      </div>

      <div class="header-right-actions">
        {#if pinned.length > 1}
          <button
            type="button"
            class="text-mode-btn"
            class:active={isReorderMode}
            onclick={(e) => {
              e.stopPropagation();
              isReorderMode = !isReorderMode;
            }}
            title={isReorderMode ? "Done reordering" : "Reorder apps"}
            aria-label={isReorderMode ? "Done reordering" : "Reorder apps"}
          >
            {isReorderMode ? "Done" : "Reorder"}
          </button>
        {/if}

        <button
          type="button"
          class="icon-action-btn add-btn"
          onclick={(e) => {
            e.stopPropagation();
            launcherStore.pickAndAdd();
          }}
          title="Add application (.exe or .lnk)"
          aria-label="Add application"
        >
          <IconPlus size={11} color="var(--text-primary)" />
        </button>

        <button
          type="button"
          class="icon-action-btn"
          onclick={(e) => {
            e.stopPropagation();
            launcherStore.closeLauncherSurface();
          }}
          title="Close launcher"
          aria-label="Close launcher"
        >
          <IconX size={10} color="var(--text-tertiary)" />
        </button>
      </div>
    </div>

    <!-- Dock App Tiles -->
    <div class="dock-body">
      {#if pinned.length === 0}
        <div class="empty-dock">
          <span class="empty-label">No pinned apps</span>
          <button
            type="button"
            class="action-pill primary empty-add"
            onclick={(e) => {
              e.stopPropagation();
              launcherStore.pickAndAdd();
            }}
            aria-label="Add first application"
          >
            <IconPlus size={10} color="var(--text-primary)" />
            <span>Add app</span>
          </button>
        </div>
      {:else}
        <div class="dock-row">
          {#if pageIndex > 0}
            <button
              type="button"
              class="page-arrow-btn"
              onclick={prevPage}
              title="Previous apps"
              aria-label="Previous apps"
            >
              <IconChevronLeft size={10} color="var(--text-secondary)" />
            </button>
          {/if}

          <div class="app-tiles-wrapper">
            {#each currentPageApps as app (app.id)}
              <div class="app-item-container">
                <button
                  type="button"
                  class="app-tile"
                  class:broken={app.exists === false}
                  class:launching={launcherStore.launchingAppId === app.id}
                  onclick={(e) => handleAppClick(app, e)}
                  title={app.exists === false ? `${app.name} (Broken: click to locate or remove)` : `Launch ${app.name}`}
                  aria-label={app.exists === false ? `${app.name} broken application` : `Launch ${app.name}`}
                >
                  {#if app.iconDataUrl}
                    <img src={app.iconDataUrl} alt="" class="app-icon-img" />
                  {:else}
                    <IconApp size={16} color="var(--text-secondary)" />
                  {/if}

                  {#if app.exists === false}
                    <div class="broken-badge" title="Executable not found" aria-hidden="true">
                      <IconAlertCircle size={8} color="#fbbf24" />
                    </div>
                  {/if}
                </button>

                {#if isReorderMode}
                  <div class="reorder-controls">
                    <button
                      type="button"
                      class="mini-reorder-btn"
                      onclick={(e) => {
                        e.stopPropagation();
                        launcherStore.moveLeft(app);
                      }}
                      title="Move left"
                      aria-label="Move left"
                    >
                      <IconChevronLeft size={8} color="var(--text-secondary)" />
                    </button>
                    <button
                      type="button"
                      class="mini-reorder-btn"
                      onclick={(e) => {
                        e.stopPropagation();
                        launcherStore.moveRight(app);
                      }}
                      title="Move right"
                      aria-label="Move right"
                    >
                      <IconChevronRight size={8} color="var(--text-secondary)" />
                    </button>
                  </div>
                {/if}
              </div>
            {/each}
          </div>

          {#if pageIndex < totalPages - 1}
            <button
              type="button"
              class="page-arrow-btn"
              onclick={nextPage}
              title="Next apps"
              aria-label="Next apps"
            >
              <IconChevronRight size={10} color="var(--text-secondary)" />
            </button>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .dock-container {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 3px;
    width: 100%;
    min-width: 0;
    user-select: none;
    box-sizing: border-box;
  }

  .context-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    min-width: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: var(--space-6);
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

  .broken-title {
    color: #f59e0b;
  }

  .page-indicator {
    font-size: var(--font-size-tiny);
    font-weight: 500;
    color: var(--text-tertiary);
    line-height: var(--line-height-tight);
  }

  .header-right-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .icon-action-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 4px;
    cursor: pointer;
    padding: 0;
    color: var(--text-tertiary);
    transition: all var(--motion-micro) var(--ease-natural);
  }

  .icon-action-btn:hover {
    background-color: var(--surface-hover);
    color: var(--text-primary);
  }

  .add-btn {
    background: var(--surface-raised);
    border-color: var(--border-subtle);
  }

  .add-btn:hover {
    border-color: rgba(255, 255, 255, 0.2);
    background-color: var(--surface-hover);
  }

  .text-mode-btn {
    background: transparent;
    border: 1px solid transparent;
    border-radius: 4px;
    padding: 1px 5px;
    font-size: var(--font-size-tiny);
    font-weight: 500;
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all var(--motion-micro) var(--ease-natural);
    line-height: 1.2;
  }

  .text-mode-btn:hover {
    color: var(--text-primary);
    background-color: var(--surface-hover);
  }

  .text-mode-btn.active {
    background-color: var(--surface-raised);
    color: var(--text-primary);
    border-color: var(--border-subtle);
  }

  .dock-body {
    display: flex;
    align-items: center;
    min-width: 0;
    height: 38px;
  }

  .empty-dock {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 0 4px;
  }

  .empty-label {
    font-size: var(--font-size-meta);
    color: var(--text-secondary);
  }

  .dock-row {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 100%;
    min-width: 0;
  }

  .app-tiles-wrapper {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
    min-width: 0;
  }

  .app-item-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    position: relative;
  }

  .app-tile {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 6px;
    background-color: var(--surface-raised);
    border: 1px solid var(--border-subtle);
    padding: 0;
    cursor: pointer;
    flex-shrink: 0;
    transition: 
      background-color var(--motion-micro) var(--ease-natural),
      border-color var(--motion-micro) var(--ease-natural),
      transform var(--motion-micro) var(--ease-natural);
  }

  .app-tile:hover {
    background-color: var(--surface-hover);
    border-color: rgba(255, 255, 255, 0.2);
    transform: translateY(-1px);
  }

  .app-tile:active,
  .app-tile.launching {
    transform: scale(0.92);
    background-color: rgba(255, 255, 255, 0.12);
  }

  .app-tile.broken {
    border-color: rgba(245, 158, 11, 0.4);
    background-color: rgba(245, 158, 11, 0.08);
  }

  .app-icon-img {
    width: 20px;
    height: 20px;
    object-fit: contain;
    border-radius: 2px;
    pointer-events: none;
  }

  .broken-badge {
    position: absolute;
    top: -3px;
    right: -3px;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background-color: #1c1917;
    border: 1px solid #fbbf24;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .reorder-controls {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .mini-reorder-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 12px;
    padding: 0;
    background: var(--surface-raised);
    border: 1px solid var(--border-subtle);
    border-radius: 2px;
    cursor: pointer;
  }

  .mini-reorder-btn:hover {
    background-color: var(--surface-hover);
  }

  .page-arrow-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 28px;
    background: transparent;
    border: none;
    border-radius: 3px;
    cursor: pointer;
    padding: 0;
    color: var(--text-tertiary);
    transition: background-color var(--motion-micro) var(--ease-natural);
  }

  .page-arrow-btn:hover {
    background-color: var(--surface-hover);
    color: var(--text-primary);
  }

  /* Broken application resolution layout */
  .broken-view {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    min-width: 0;
  }

  .broken-body {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-8);
    width: 100%;
    min-width: 0;
  }

  .broken-path-label {
    font-size: var(--font-size-meta);
    color: #fbbf24;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
    flex: 1;
  }

  .broken-actions-row {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .action-pill {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: var(--surface-raised);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 2px 7px;
    font-size: var(--font-size-tiny);
    font-weight: 500;
    color: var(--text-secondary);
    cursor: pointer;
    line-height: 1.2;
    transition: all var(--motion-micro) var(--ease-natural);
  }

  .action-pill:hover {
    background-color: var(--surface-hover);
    color: var(--text-primary);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .action-pill.primary:hover {
    border-color: rgba(45, 212, 191, 0.4);
    color: var(--text-primary);
  }

  .action-pill.danger:hover {
    border-color: rgba(239, 68, 68, 0.4);
    color: #f87171;
  }
</style>
