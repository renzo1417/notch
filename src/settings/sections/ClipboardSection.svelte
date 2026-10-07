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

<div class="section-container" role="tabpanel" aria-label="Clipboard Settings">
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
      <IconTrash size={12} color="currentColor" />
      <span>Clear Now</span>
    </button>
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

  .privacy-note {
    color: #eab308;
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
    min-width: 50px;
    text-align: right;
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

  .action-button {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 8px;
    border-radius: 6px;
    font-size: var(--font-size-meta);
    font-weight: 500;
    border: 1px solid var(--border-subtle);
    background: var(--surface-root);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all var(--motion-micro) var(--ease-natural);
  }

  .action-button:hover:not(:disabled) {
    background: var(--surface-hover);
    color: var(--text-primary);
  }

  .action-button.danger {
    color: #ef4444;
    border-color: rgba(239, 68, 68, 0.3);
  }

  .action-button.danger:hover:not(:disabled) {
    background: rgba(239, 68, 68, 0.15);
  }

  .action-button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
