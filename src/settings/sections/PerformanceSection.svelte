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

<div class="section-container" role="group" aria-label="Performance Settings">
  <!-- Battery Saver Mode -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Battery Saver Mode</span>
      <span class="setting-desc">Reduce animation to use less power.</span>
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
      <span class="setting-desc">Keep Pip still between interactions.</span>
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
