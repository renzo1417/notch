<script lang="ts">
  import { settingsStore } from "../settings-store.svelte";

  const performance = $derived(settingsStore.state.performance);

  function toggleBatterySaver() {
    settingsStore.update("performance", { batterySaverMode: !performance.batterySaverMode });
  }

  function togglePauseInactive() {
    settingsStore.update("performance", { pauseInactiveAnimations: !performance.pauseInactiveAnimations });
  }

  function setFps(fps: number) {
    settingsStore.update("performance", { targetFps: fps });
  }
</script>

<div class="section-container" role="tabpanel" aria-label="Performance Settings">
  <!-- Battery Saver Mode -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Battery Saver Mode</span>
      <span class="setting-desc">Cap render loop to 30 FPS and suspend non-essential micro-motion</span>
    </div>
    <button
      type="button"
      class="win-switch"
      class:checked={performance.batterySaverMode}
      role="switch"
      aria-checked={performance.batterySaverMode}
      aria-label="Toggle Battery Saver Mode"
      onclick={toggleBatterySaver}
    >
      <span class="switch-thumb"></span>
    </button>
  </div>

  <!-- Pause Inactive Animations -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Pause Dormant Animations</span>
      <span class="setting-desc">Zero CPU redraw cycles when companion is idle or collapsed</span>
    </div>
    <button
      type="button"
      class="win-switch"
      class:checked={performance.pauseInactiveAnimations}
      role="switch"
      aria-checked={performance.pauseInactiveAnimations}
      aria-label="Toggle Pause Inactive Animations"
      onclick={togglePauseInactive}
    >
      <span class="switch-thumb"></span>
    </button>
  </div>

  <!-- Target Frame Rate -->
  <div class="setting-row" class:disabled={performance.batterySaverMode}>
    <div class="setting-info">
      <span class="setting-label">Target Frame Rate</span>
      <span class="setting-desc">Maximum repaint cadence for mascot and shell transitions</span>
    </div>
    <div class="fps-buttons" role="group" aria-label="Target Frame Rate">
      <button
        type="button"
        class="fps-btn"
        class:active={performance.targetFps === 30 || performance.batterySaverMode}
        onclick={() => setFps(30)}
      >
        30 FPS
      </button>
      <button
        type="button"
        class="fps-btn"
        class:active={performance.targetFps === 60 && !performance.batterySaverMode}
        disabled={performance.batterySaverMode}
        onclick={() => setFps(60)}
      >
        60 FPS
      </button>
    </div>
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

  .fps-buttons {
    display: flex;
    gap: 4px;
    background: var(--surface-root);
    padding: 2px;
    border-radius: 6px;
    border: 1px solid var(--border-strong);
  }

  .fps-btn {
    padding: 3px 8px;
    font-size: var(--font-size-meta);
    font-weight: 600;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all var(--motion-micro) var(--ease-natural);
  }

  .fps-btn:hover:not(:disabled) {
    color: var(--text-primary);
  }

  .fps-btn.active {
    background: var(--accent-primary);
    color: white;
  }

  .fps-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
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
