<script lang="ts">
  import { onMount } from "svelte";
  import { clipboardStore } from "./clipboard-store.svelte";
  import type { ClipboardItem } from "./types";
  import IconCopy from "../../icons/IconCopy.svelte";
  import IconCheck from "../../icons/IconCheck.svelte";
  import IconTrash from "../../icons/IconTrash.svelte";
  import IconPause from "../../icons/IconPause.svelte";
  import IconPlay from "../../icons/IconPlay.svelte";
  import IconX from "../../icons/IconX.svelte";

  interface Props {
    onclose: () => void;
  }

  let { onclose }: Props = $props();

  const items = $derived(clipboardStore.items);

  function handleCopy(item: ClipboardItem, e: MouseEvent) {
    e.stopPropagation();
    clipboardStore.copyItem(item);
  }

  onMount(() => {
    clipboardStore.init();
  });
</script>

<div class="tool-panel clipboard-panel" aria-label="Clipboard history">
  <!-- Header -->
  <div class="panel-header">
    <div class="header-left">
      <span class="panel-title">Clipboard</span>
      <span class="item-count">{items.length}</span>
    </div>

    <div class="header-actions">
      <button
        type="button"
        class="action-btn"
        class:active={clipboardStore.isPaused}
        onclick={(e) => {
          e.stopPropagation();
          clipboardStore.togglePause();
        }}
        title={clipboardStore.isPaused ? "Resume clipboard capture" : "Pause clipboard capture"}
        aria-label="Toggle pause"
      >
        {#if clipboardStore.isPaused}
          <IconPlay size={16} color="var(--status-warning)" />
        {:else}
          <IconPause size={16} color="var(--text-secondary)" />
        {/if}
      </button>

      {#if items.length > 0}
        <button
          type="button"
          class="action-btn"
          onclick={(e) => {
            e.stopPropagation();
            clipboardStore.clearHistory();
          }}
          title="Clear history"
          aria-label="Clear history"
        >
          <IconTrash size={16} color="var(--text-secondary)" />
        </button>
      {/if}

      <button
        type="button"
        class="close-btn"
        onclick={(e) => {
          e.stopPropagation();
          onclose();
        }}
        title="Close Clipboard (Esc)"
        aria-label="Close Clipboard"
      >
        <IconX size={16} color="var(--text-tertiary)" />
      </button>
    </div>
  </div>

  <!-- Body: list of items -->
  <div class="items-list">
    {#if items.length === 0}
      <div class="empty-state">
        <span class="empty-text">
          {clipboardStore.isPaused ? "Capture paused" : "Clipboard empty · Copy text anywhere to see it here"}
        </span>
      </div>
    {:else}
      {#each items as item (item.id)}
        <div
          class="item-card"
          role="button"
          tabindex="0"
          onclick={(e) => handleCopy(item, e)}
          onkeydown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              handleCopy(item, e as unknown as MouseEvent);
            }
          }}
          title="Click to copy"
        >
          <span class="item-text">{item.text.trim()}</span>

          <span class="copy-btn" aria-hidden="true">
            {#if clipboardStore.copiedId === item.id}<IconCheck size={16} />{:else}<IconCopy size={16} />{/if}
          </span>
        </div>
      {/each}
    {/if}
  </div>
</div>
