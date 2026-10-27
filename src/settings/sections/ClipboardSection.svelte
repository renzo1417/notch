<script lang="ts">
  import { settingsStore } from "../settings-store.svelte";
  import { clipboardStore } from "../../modules/clipboard/clipboard-store.svelte";
  import IconTrash from "../../icons/IconTrash.svelte";

  const clipboard = $derived(settingsStore.state.clipboard);

  function toggleClipboard() {
    const next = !clipboard.enabled;
    settingsStore.update("clipboard", { enabled: next });
    clipboardStore.setPaused(!next);
  }

  function handleLimitChange(event: Event) {
    const target = event.target as HTMLInputElement;
    const val = Number(target.value);
    settingsStore.update("clipboard", { historyLimit: val });
    clipboardStore.setMaxItems(val);
  }

  function togglePersist() {
    settingsStore.update("clipboard", { persistHistory: !clipboard.persistHistory });
  }

  function toggleClearOnExit() {
    settingsStore.update("clipboard", { clearOnExit: !clipboard.clearOnExit });
  }

  async function handleClearHistory() {
    await clipboardStore.clearHistory();
  }
</script>

<div class="section-container" role="group" aria-label="Clipboard Settings">
  <!-- Enable Clipboard -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Clipboard Listener</span>
      <span class="setting-desc">Capture text copies via native Windows WM_CLIPBOARDUPDATE</span>
    </div>
    <button
      type="button"
      class="win-switch"
      class:checked={clipboard.enabled}
      role="switch"
      aria-checked={clipboard.enabled}
      aria-label="Toggle Clipboard Listener"
      onclick={toggleClipboard}
    >
      <span class="switch-thumb"></span>
    </button>
  </div>

  <!-- History Limit -->
  <div class="setting-row" class:disabled={!clipboard.enabled}>
    <div class="setting-info">
      <span class="setting-label">History Capacity</span>
      <span class="setting-desc">Maximum recent items held in memory</span>
    </div>
    <div class="slider-group">
      <input
        type="range"
        class="win-slider"
        min="10"
        max="100"
        step="5"
        disabled={!clipboard.enabled}
        value={clipboard.historyLimit}
        oninput={handleLimitChange}
        aria-label="Clipboard History Limit"
      />
      <span class="slider-label">{clipboard.historyLimit} items</span>
    </div>
  </div>

  <!-- Persistence (Privacy Guarantee) -->
  <div class="setting-row" class:disabled={!clipboard.enabled}>
    <div class="setting-info">
      <span class="setting-label">Persist History to Disk</span>
      <span class="setting-desc privacy-note">OFF by default · Local-only privacy guarantee</span>
    </div>
    <button
      type="button"
      class="win-switch"
      class:checked={clipboard.persistHistory}
      disabled={!clipboard.enabled}
      role="switch"
      aria-checked={clipboard.persistHistory}
      aria-label="Toggle Clipboard Persistence"
      onclick={togglePersist}
    >
      <span class="switch-thumb"></span>
    </button>
  </div>

  <!-- Clear on Exit -->
  <div class="setting-row" class:disabled={!clipboard.enabled}>
    <div class="setting-info">
      <span class="setting-label">Wipe on Exit</span>
      <span class="setting-desc">Cleanly purge memory clipboard buffer when app closes</span>
    </div>
    <button
      type="button"
      class="win-switch"
      class:checked={clipboard.clearOnExit}
      disabled={!clipboard.enabled}
      role="switch"
      aria-checked={clipboard.clearOnExit}
      aria-label="Toggle Wipe on Exit"
      onclick={toggleClearOnExit}
    >
      <span class="switch-thumb"></span>
    </button>
  </div>

  <!-- Clear History Action -->
  <div class="setting-row" class:disabled={!clipboard.enabled}>
    <div class="setting-info">
      <span class="setting-label">Wipe Current Buffer</span>
      <span class="setting-desc">Instantly purge all {clipboardStore.count} copied text items</span>
    </div>
    <button
      type="button"
      class="action-button danger"
      disabled={!clipboard.enabled || clipboardStore.count === 0}
      onclick={handleClearHistory}
      title="Clear all stored clipboard history"
    >
      <IconTrash size={16} color="currentColor" />
      <span>Clear Now</span>
    </button>
  </div>
</div>
