<script lang="ts">
  import { settingsStore } from "../settings-store.svelte";

  const music = $derived(settingsStore.state.music);

  function toggleMusic() {
    settingsStore.update("music", { enabled: !music.enabled });
  }

  function toggleControls() {
    settingsStore.update("music", { showControls: !music.showControls });
  }
</script>

<div class="section-container" role="tabpanel" aria-label="Music Settings">
  <!-- Enable Music Module -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Enable Media Zone</span>
      <span class="setting-desc">Persistent right-zone Windows GSMTC audio controller</span>
    </div>
    <button
      type="button"
      class="win-switch"
      class:checked={music.enabled}
      role="switch"
      aria-checked={music.enabled}
      aria-label="Toggle Media Module"
      onclick={toggleMusic}
    >
      <span class="switch-thumb"></span>
    </button>
  </div>

  <!-- Transport Controls -->
  <div class="setting-row" class:disabled={!music.enabled}>
    <div class="setting-info">
      <span class="setting-label">Show Playback Controls</span>
      <span class="setting-desc">Display previous, play/pause, and next transport buttons</span>
    </div>
    <button
      type="button"
      class="win-switch"
      class:checked={music.showControls}
      disabled={!music.enabled}
      role="switch"
      aria-checked={music.showControls}
      aria-label="Toggle Playback Controls"
      onclick={toggleControls}
    >
      <span class="switch-thumb"></span>
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
