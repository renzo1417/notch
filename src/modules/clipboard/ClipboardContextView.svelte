<script lang="ts">
  import { clipboardStore } from "./clipboard-store.svelte";
  import type { ClipboardItem } from "./types";
  import IconClipboard from "../../icons/IconClipboard.svelte";
  import IconCopy from "../../icons/IconCopy.svelte";
  import IconCheck from "../../icons/IconCheck.svelte";
  import IconTrash from "../../icons/IconTrash.svelte";
  import IconPause from "../../icons/IconPause.svelte";
  import IconPlay from "../../icons/IconPlay.svelte";
  import IconX from "../../icons/IconX.svelte";

  let selectedIndex = $state(0);

  const items = $derived(clipboardStore.items);
  const activeItem = $derived(items.length > 0 ? items[Math.min(selectedIndex, items.length - 1)] : null);

  function handleCopy(item: ClipboardItem, e: MouseEvent) {
    e.stopPropagation();
    clipboardStore.copyItem(item);
  }

  function handleSelect(index: number, e: MouseEvent) {
    e.stopPropagation();
    selectedIndex = index;
  }
</script>

<div class="clipboard-container" aria-label="Clipboard History Surface">
  <!-- Header row -->
  <div class="clipboard-header">
    <div class="header-left">
      <IconClipboard size={11} color="var(--text-tertiary)" />
      <span class="clipboard-category">
        CLIPBOARD · {clipboardStore.isPaused ? "PAUSED" : `${items.length} ${items.length === 1 ? "ITEM" : "ITEMS"}`}
      </span>
    </div>

    <div class="header-actions">
      <!-- Toggle Pause -->
      <button
        type="button"
        class="icon-btn"
        class:active={clipboardStore.isPaused}
        onclick={(e) => {
          e.stopPropagation();
          clipboardStore.togglePause();
        }}
        title={clipboardStore.isPaused ? "Resume clipboard capture" : "Pause clipboard capture"}
        aria-label={clipboardStore.isPaused ? "Resume clipboard capture" : "Pause clipboard capture"}
      >
        {#if clipboardStore.isPaused}
          <IconPlay size={10} color="var(--text-primary)" />
        {:else}
          <IconPause size={10} color="var(--text-secondary)" />
        {/if}
      </button>

      <!-- Clear History -->
      {#if items.length > 0}
        <button
          type="button"
          class="icon-btn"
          onclick={(e) => {
            e.stopPropagation();
            clipboardStore.clearHistory();
          }}
          title="Clear clipboard history"
          aria-label="Clear clipboard history"
        >
          <IconTrash size={10} color="var(--text-secondary)" />
        </button>
      {/if}

      <!-- Close Surface -->
      <button
        type="button"
        class="icon-btn"
        onclick={(e) => {
          e.stopPropagation();
          clipboardStore.closeClipboardSurface();
        }}
        title="Close clipboard view"
        aria-label="Close clipboard view"
      >
        <IconX size={10} color="var(--text-tertiary)" />
      </button>
    </div>
  </div>

  <!-- Content body -->
  <div class="clipboard-body">
    {#if items.length === 0}
      <div class="empty-state">
        <span class="empty-title">
          {clipboardStore.isPaused ? "Capture paused" : "Clipboard history empty"}
        </span>
        <span class="empty-meta">
          {clipboardStore.isPaused ? "Resume to capture new copies" : "Copy text anywhere to see it here"}
        </span>
      </div>
    {:else}
      <!-- Active item display with 1-click copy -->
      <div class="item-card">
        <div class="item-text-wrapper" title={activeItem?.text}>
          <span class="item-text">{activeItem?.text}</span>
        </div>

        <button
          type="button"
          class="copy-btn"
          class:copied={activeItem && clipboardStore.copiedId === activeItem.id}
          onclick={(e) => {
            if (activeItem) handleCopy(activeItem, e);
          }}
          title="Copy text to clipboard"
          aria-label="Copy text to clipboard"
        >
          {#if activeItem && clipboardStore.copiedId === activeItem.id}
            <IconCheck size={11} color="#22c55e" />
            <span>Copied</span>
          {:else}
            <IconCopy size={11} color="var(--text-primary)" />
            <span>Copy</span>
          {/if}
        </button>
      </div>

      <!-- Quick switcher strip if more than 1 item -->
      {#if items.length > 1}
        <div class="recent-strip" role="tablist" aria-label="Recent clipboard items">
          {#each items.slice(0, 4) as item, i}
            <button
              type="button"
              role="tab"
              aria-selected={i === selectedIndex}
              class="strip-pill"
              class:selected={i === selectedIndex}
              onclick={(e) => handleSelect(i, e)}
              title={item.text}
            >
              <span class="strip-text">{item.text.replace(/\s+/g, " ")}</span>
            </button>
          {/each}
          {#if items.length > 4}
            <span class="more-count" title="{items.length - 4} more items in history">
              +{items.length - 4}
            </span>
          {/if}
        </div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .clipboard-container {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 2px;
    width: 100%;
    min-width: 0;
  }

  .clipboard-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-6);
    line-height: var(--line-height-tight);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
  }

  .clipboard-category {
    font-size: var(--font-size-tiny);
    font-weight: 600;
    letter-spacing: 0.6px;
    color: var(--text-tertiary);
    text-transform: uppercase;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 3px;
    flex-shrink: 0;
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    background: transparent;
    border: none;
    border-radius: 3px;
    cursor: pointer;
    padding: 0;
    transition: background-color var(--motion-micro) var(--ease-natural);
  }

  .icon-btn:hover {
    background-color: var(--surface-hover);
  }

  .icon-btn.active {
    background-color: rgba(239, 68, 68, 0.15);
  }

  .clipboard-body {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    gap: 1px;
    padding: 2px 0;
  }

  .empty-title {
    font-size: var(--font-size-body);
    font-weight: 500;
    color: var(--text-primary);
    line-height: var(--line-height-tight);
  }

  .empty-meta {
    font-size: var(--font-size-meta);
    color: var(--text-tertiary);
    line-height: var(--line-height-tight);
  }

  .item-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-6);
    background: var(--surface-raised);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-inner);
    padding: 3px 6px;
    min-width: 0;
  }

  .item-text-wrapper {
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }

  .item-text {
    display: block;
    font-size: var(--font-size-meta);
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.3;
  }

  .copy-btn {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 1px 6px;
    font-size: var(--font-size-tiny);
    font-weight: 500;
    color: var(--text-primary);
    cursor: pointer;
    flex-shrink: 0;
    transition: all var(--motion-micro) var(--ease-natural);
  }

  .copy-btn:hover {
    background: var(--surface-hover);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .copy-btn.copied {
    border-color: rgba(34, 197, 94, 0.4);
    color: #22c55e;
  }

  .recent-strip {
    display: flex;
    align-items: center;
    gap: 3px;
    min-width: 0;
    overflow: hidden;
  }

  .strip-pill {
    flex: 1;
    min-width: 0;
    max-width: 58px;
    background: transparent;
    border: 1px solid var(--border-subtle);
    border-radius: 3px;
    padding: 1px 4px;
    cursor: pointer;
    text-align: left;
    transition: all var(--motion-micro) var(--ease-natural);
  }

  .strip-pill:hover {
    background: var(--surface-hover);
  }

  .strip-pill.selected {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.25);
  }

  .strip-text {
    display: block;
    font-size: 9px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.2;
  }

  .more-count {
    font-size: 9px;
    color: var(--text-tertiary);
    flex-shrink: 0;
    padding: 0 2px;
  }
</style>
