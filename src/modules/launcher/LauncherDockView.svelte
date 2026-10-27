<script lang="ts">
  import { onMount } from "svelte";
  import { launcherStore } from "./launcher-store.svelte";
  import type { LaunchableApp } from "./types";
  import IconPlus from "../../icons/IconPlus.svelte";
  import IconApp from "../../icons/IconApp.svelte";
  import IconTrash from "../../icons/IconTrash.svelte";
  import IconFolder from "../../icons/IconFolder.svelte";
  import IconAlertCircle from "../../icons/IconAlertCircle.svelte";
  import IconChevronLeft from "../../icons/IconChevronLeft.svelte";
  import IconChevronRight from "../../icons/IconChevronRight.svelte";
  import IconMoreHorizontal from "../../icons/IconMoreHorizontal.svelte";
  import IconPlay from "../../icons/IconPlay.svelte";
  import IconX from "../../icons/IconX.svelte";

  interface Props {
    onclose?: () => void;
  }

  let { onclose }: Props = $props();

  let isReorderMode = $state(false);
  let activeMenuAppId = $state<string | null>(null);

  const pinned = $derived(launcherStore.pinnedApps);

  function handleClose() {
    activeMenuAppId = null;
    launcherStore.closeLauncherSurface();
    onclose?.();
  }

  function handleAppClick(app: LaunchableApp, event: MouseEvent) {
    event.stopPropagation();
    activeMenuAppId = null;
    if (isReorderMode) return;

    if (app.exists === false) {
      launcherStore.brokenApp = app;
    } else {
      launcherStore.launch(app);
    }
  }

  function handleContextMenu(app: LaunchableApp, event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    activeMenuAppId = activeMenuAppId === app.id ? null : app.id;
  }

  function toggleMenu(app: LaunchableApp, event: MouseEvent) {
    event.stopPropagation();
    activeMenuAppId = activeMenuAppId === app.id ? null : app.id;
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      if (activeMenuAppId) {
        activeMenuAppId = null;
      } else if (launcherStore.brokenApp) {
        launcherStore.brokenApp = null;
      } else {
        handleClose();
      }
    }
  }

  onMount(() => {
    launcherStore.init();
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  });
</script>

<div
  class="apps-panel"
  role="presentation"
  onclick={() => {
    activeMenuAppId = null;
  }}
  onkeydown={() => {}}
>
  <!-- Header Row -->
  <div class="panel-header">
    <div class="header-left">
      <span class="panel-title">Quick Apps</span>
      <span class="app-count-badge">
        {pinned.length} {pinned.length === 1 ? "app" : "apps"}
      </span>
    </div>

    <div class="header-actions">
      {#if pinned.length > 1}
        <button
          type="button"
          class="reorder-toggle-btn"
          class:active={isReorderMode}
          onclick={(e) => {
            e.stopPropagation();
            isReorderMode = !isReorderMode;
            activeMenuAppId = null;
          }}
          title={isReorderMode ? "Finish reordering" : "Reorder apps"}
          aria-label={isReorderMode ? "Finish reordering" : "Reorder apps"}
        >
          {isReorderMode ? "Done" : "Reorder"}
        </button>
      {/if}

      <button
        type="button"
        class="close-btn"
        onclick={(e) => {
          e.stopPropagation();
          handleClose();
        }}
        title="Close apps panel (Esc)"
        aria-label="Close apps panel"
      >
        <IconX size={12} color="var(--text-secondary)" />
      </button>
    </div>
  </div>

  <!-- Missing App Resolution Alert Banner (if any) -->
  {#if launcherStore.brokenApp}
    <div class="broken-banner" role="alert" aria-label="Missing app alert">
      <div class="broken-banner-info">
        <IconAlertCircle size={14} color="var(--status-warning)" />
        <span class="broken-banner-text">
          <strong>App not found:</strong> {launcherStore.brokenApp.name}
        </span>
      </div>
      <div class="broken-banner-actions">
        <button
          type="button"
          class="banner-action-btn locate"
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
          class="banner-action-btn remove"
          onclick={(e) => {
            e.stopPropagation();
            if (launcherStore.brokenApp) launcherStore.remove(launcherStore.brokenApp);
          }}
          title="Remove from launcher"
          aria-label="Remove application"
        >
          <IconTrash size={11} color="var(--status-error)" />
          <span>Remove</span>
        </button>
        <button
          type="button"
          class="banner-action-btn dismiss"
          onclick={(e) => {
            e.stopPropagation();
            launcherStore.brokenApp = null;
          }}
          title="Dismiss alert"
          aria-label="Dismiss alert"
        >
          <IconX size={10} color="var(--text-tertiary)" />
        </button>
      </div>
    </div>
  {/if}

  <!-- Main Grid Area -->
  <div class="panel-body">
    {#if pinned.length === 0}
      <div class="empty-state">
        <div class="empty-icon-wrap" aria-hidden="true">
          <IconApp size={24} color="var(--text-tertiary)" />
        </div>
        <span class="empty-title">No applications saved</span>
        <span class="empty-sub">
          Pin frequently used applications for instant top-center access.
        </span>
      </div>
    {:else}
      <div class="apps-grid" role="list">
        {#each pinned as app (app.id)}
          <div
            class="app-card"
            class:broken={app.exists === false}
            class:launching={launcherStore.launchingAppId === app.id}
            class:menu-open={activeMenuAppId === app.id}
            role="button"
            tabindex="0"
            onclick={(e) => handleAppClick(app, e)}
            oncontextmenu={(e) => handleContextMenu(app, e)}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                handleAppClick(app, e as unknown as MouseEvent);
              }
            }}
            title={app.exists === false ? `${app.name} (Executable missing — click to resolve)` : `Launch ${app.name}`}
          >
            <!-- App Icon -->
            <div class="app-icon-container">
              {#if app.iconDataUrl}
                <img src={app.iconDataUrl} alt="" class="app-icon-img" />
              {:else}
                <IconApp size={18} color="var(--text-secondary)" />
              {/if}

              {#if app.exists === false}
                <div class="broken-indicator" title="App not found" aria-hidden="true">
                  <IconAlertCircle size={9} color="var(--status-warning)" />
                </div>
              {/if}
            </div>

            <!-- App Details -->
            <div class="app-details">
              <span class="app-name">{app.name}</span>
              <span class="app-sub">
                {#if app.exists === false}
                  <span class="warn-text">App not found</span>
                {:else if launcherStore.launchingAppId === app.id}
                  <span class="launching-text">Launching…</span>
                {:else}
                  Ready
                {/if}
              </span>
            </div>

            <!-- Overflow / Menu Trigger -->
            <button
              type="button"
              class="card-overflow-btn"
              onclick={(e) => toggleMenu(app, e)}
              title="Application options"
              aria-label="Options for {app.name}"
            >
              <IconMoreHorizontal size={13} color="var(--text-secondary)" />
            </button>

            <!-- Inline Reorder arrows if Reorder mode is active -->
            {#if isReorderMode}
              <div class="reorder-arrows-bar">
                <button
                  type="button"
                  class="arrow-btn"
                  onclick={(e) => {
                    e.stopPropagation();
                    launcherStore.moveLeft(app);
                  }}
                  title="Move earlier"
                  aria-label="Move earlier"
                >
                  <IconChevronLeft size={10} color="var(--text-primary)" />
                </button>
                <button
                  type="button"
                  class="arrow-btn"
                  onclick={(e) => {
                    e.stopPropagation();
                    launcherStore.moveRight(app);
                  }}
                  title="Move later"
                  aria-label="Move later"
                >
                  <IconChevronRight size={10} color="var(--text-primary)" />
                </button>
              </div>
            {/if}

            <!-- Contextual Options Menu -->
            {#if activeMenuAppId === app.id}
              <div
                class="context-menu-popover"
                role="menu"
                tabindex="-1"
                onclick={(e) => e.stopPropagation()}
                onkeydown={(e) => e.stopPropagation()}
              >
                {#if app.exists !== false}
                  <button
                    type="button"
                    class="menu-item"
                    role="menuitem"
                    onclick={(e) => {
                      e.stopPropagation();
                      activeMenuAppId = null;
                      launcherStore.launch(app);
                    }}
                  >
                    <IconPlay size={11} color="var(--text-primary)" />
                    <span>Launch</span>
                  </button>
                {:else}
                  <button
                    type="button"
                    class="menu-item"
                    role="menuitem"
                    onclick={(e) => {
                      e.stopPropagation();
                      activeMenuAppId = null;
                      launcherStore.locate(app);
                    }}
                  >
                    <IconFolder size={11} color="var(--text-primary)" />
                    <span>Locate executable</span>
                  </button>
                {/if}

                <button
                  type="button"
                  class="menu-item"
                  role="menuitem"
                  onclick={(e) => {
                    e.stopPropagation();
                    launcherStore.moveLeft(app);
                  }}
                >
                  <IconChevronLeft size={11} color="var(--text-secondary)" />
                  <span>Move earlier</span>
                </button>

                <button
                  type="button"
                  class="menu-item"
                  role="menuitem"
                  onclick={(e) => {
                    e.stopPropagation();
                    launcherStore.moveRight(app);
                  }}
                >
                  <IconChevronRight size={11} color="var(--text-secondary)" />
                  <span>Move later</span>
                </button>

                <div class="menu-divider"></div>

                <button
                  type="button"
                  class="menu-item danger"
                  role="menuitem"
                  onclick={(e) => {
                    e.stopPropagation();
                    activeMenuAppId = null;
                    launcherStore.remove(app);
                  }}
                >
                  <IconTrash size={11} color="var(--status-error)" />
                  <span>Remove from dock</span>
                </button>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Bottom Action Bar -->
  <div class="panel-footer">
    <span class="footer-hint">Click to launch · Right-click for options</span>

    <button
      type="button"
      class="add-app-primary-btn"
      onclick={(e) => {
        e.stopPropagation();
        launcherStore.pickAndAdd();
      }}
      title="Add application (.exe or .lnk)"
      aria-label="Add Application"
    >
      <IconPlus size={13} color="var(--text-primary)" />
      <span>Add Application</span>
    </button>
  </div>
</div>

<style>
  .apps-panel {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    user-select: none;
    box-sizing: border-box;
    padding: 12px 18px 14px 18px;
    background: transparent;
  }

  /* Header */
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    margin-bottom: 10px;
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: var(--space-8);
  }

  .panel-title {
    font-size: var(--font-size-title);
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: 0.1px;
    line-height: 1;
  }

  .app-count-badge {
    font-size: var(--font-size-tiny);
    font-weight: 500;
    color: var(--text-tertiary);
    background: var(--surface-raised);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 1px 6px;
    line-height: 1.2;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: var(--space-6);
  }

  .reorder-toggle-btn {
    background: transparent;
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 2px 8px;
    font-size: var(--font-size-tiny);
    font-weight: 500;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all var(--motion-micro) var(--ease-natural);
  }

  .reorder-toggle-btn:hover {
    background-color: var(--surface-hover);
    color: var(--text-primary);
  }

  .reorder-toggle-btn.active {
    background-color: var(--surface-raised);
    border-color: var(--border-strong);
    color: var(--text-primary);
    font-weight: 600;
  }

  .close-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    background: transparent;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    padding: 0;
    transition: background-color var(--motion-micro) var(--ease-natural);
  }

  .close-btn:hover {
    background-color: var(--surface-hover);
  }

  /* Missing App Banner */
  .broken-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--surface-hover);
    border: 1px solid var(--surface-hover);
    border-radius: var(--radius-control);
    padding: 6px 10px;
    margin-bottom: 8px;
    flex-shrink: 0;
    gap: var(--space-8);
  }

  .broken-banner-info {
    display: flex;
    align-items: center;
    gap: var(--space-6);
    min-width: 0;
  }

  .broken-banner-text {
    font-size: var(--font-size-meta);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .broken-banner-actions {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    flex-shrink: 0;
  }

  .banner-action-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: var(--surface-raised);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 2px 7px;
    font-size: var(--font-size-tiny);
    font-weight: 500;
    cursor: pointer;
    color: var(--text-primary);
    transition: all var(--motion-micro) var(--ease-natural);
  }

  .banner-action-btn:hover {
    background-color: var(--surface-hover);
  }

  .banner-action-btn.locate:hover {
    border-color: var(--surface-hover);
  }

  .banner-action-btn.remove {
    color: var(--status-error);
  }

  .banner-action-btn.remove:hover {
    border-color: var(--surface-hover);
  }

  .banner-action-btn.dismiss {
    padding: 2px 4px;
    background: transparent;
    border-color: transparent;
  }

  /* Body & Grid */
  .panel-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
    padding-right: 2px;
  }

  .panel-body::-webkit-scrollbar {
    width: 4px;
  }

  .panel-body::-webkit-scrollbar-thumb {
    background: var(--border-strong);
    border-radius: 4px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    gap: 4px;
    padding: 20px 0;
  }

  .empty-icon-wrap {
    margin-bottom: 4px;
    opacity: 0.6;
  }

  .empty-title {
    font-size: var(--font-size-body);
    font-weight: 600;
    color: var(--text-primary);
  }

  .empty-sub {
    font-size: var(--font-size-meta);
    color: var(--text-secondary);
    max-width: 280px;
    line-height: var(--line-height-normal);
  }

  .apps-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
    width: 100%;
  }

  /* Responsive 3 columns if narrower */
  @media (max-width: 640px) {
    .apps-grid {
      grid-template-columns: repeat(3, 1fr);
    }
  }

  .app-card {
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
    background-color: var(--surface-raised);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-control);
    padding: 8px 10px;
    height: 52px;
    box-sizing: border-box;
    cursor: pointer;
    transition: background-color var(--motion-micro) var(--ease-natural),
                border-color var(--motion-micro) var(--ease-natural),
                transform var(--motion-micro) var(--ease-natural);
    outline: none;
  }

  .app-card:hover {
    background-color: var(--surface-hover);
    border-color: rgba(255, 255, 255, 0.2);
    transform: translateY(-1px);
  }

  .app-card:focus-visible {
    border-color: var(--accent-focus);
    outline: 2px solid var(--accent-focus);
    outline-offset: 1px;
  }

  .app-card:active,
  .app-card.launching {
    transform: scale(0.97);
    background-color: rgba(255, 255, 255, 0.12);
  }

  .app-card.broken {
    border-color: var(--surface-hover);
    background-color: var(--surface-hover);
  }

  .app-card.menu-open {
    border-color: var(--accent-primary);
  }

  .app-icon-container {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    flex-shrink: 0;
    border-radius: 5px;
    background: rgba(0, 0, 0, 0.2);
    overflow: visible;
  }

  .app-icon-img {
    width: 28px;
    height: 28px;
    object-fit: contain;
    border-radius: 4px;
    pointer-events: none;
  }

  .broken-indicator {
    position: absolute;
    top: -4px;
    right: -4px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #1c1917;
    border: 1px solid var(--status-warning);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .app-details {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
    gap: 1px;
  }

  .app-name {
    font-size: var(--font-size-label);
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: var(--line-height-tight);
  }

  .app-sub {
    font-size: var(--font-size-tiny);
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: var(--line-height-tight);
  }

  .warn-text {
    color: var(--status-warning);
    font-weight: 500;
  }

  .launching-text {
    color: var(--status-info);
    font-weight: 500;
  }

  .card-overflow-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: 4px;
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 0;
    opacity: 0;
    transition: opacity var(--motion-micro) var(--ease-natural),
                background-color var(--motion-micro) var(--ease-natural);
    flex-shrink: 0;
  }

  .app-card:hover .card-overflow-btn,
  .app-card.menu-open .card-overflow-btn {
    opacity: 0.8;
  }

  .card-overflow-btn:hover {
    opacity: 1;
    background-color: var(--surface-hover);
  }

  /* Inline Reorder Arrows */
  .reorder-arrows-bar {
    position: absolute;
    right: 4px;
    top: 50%;
    transform: translateY(-50%);
    display: flex;
    align-items: center;
    gap: 2px;
    background: var(--surface-root);
    border: 1px solid var(--border-strong);
    border-radius: 4px;
    padding: 1px;
  }

  .arrow-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border-radius: 3px;
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 0;
  }

  .arrow-btn:hover {
    background-color: var(--surface-hover);
  }

  /* Context Menu Popover */
  .context-menu-popover {
    position: absolute;
    top: 100%;
    right: 4px;
    z-index: 50;
    min-width: 140px;
    background: var(--surface-root);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-control);
    box-shadow: var(--shadow-soft);
    padding: 4px;
    display: flex;
    flex-direction: column;
    gap: 1px;
    margin-top: 4px;
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 8px;
    border-radius: 4px;
    background: transparent;
    border: none;
    font-size: var(--font-size-meta);
    font-weight: 500;
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
    transition: background-color var(--motion-micro) var(--ease-natural);
  }

  .menu-item:hover {
    background-color: var(--surface-hover);
  }

  .menu-item.danger {
    color: var(--status-error);
  }

  .menu-item.danger:hover {
    background-color: var(--surface-hover);
  }

  .menu-divider {
    height: 1px;
    background-color: var(--border-subtle);
    margin: 2px 0;
  }

  /* Footer */
  .panel-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    margin-top: 10px;
    padding-top: 8px;
    border-top: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .footer-hint {
    font-size: var(--font-size-tiny);
    color: var(--text-tertiary);
  }

  .add-app-primary-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: var(--surface-raised);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-control);
    padding: 5px 12px;
    font-size: var(--font-size-label);
    font-weight: 600;
    color: var(--text-primary);
    cursor: pointer;
    transition: all var(--motion-micro) var(--ease-natural);
  }

  .add-app-primary-btn:hover {
    background-color: var(--surface-hover);
    border-color: rgba(255, 255, 255, 0.25);
  }

  .add-app-primary-btn:active {
    transform: scale(0.98);
  }
</style>
