<script lang="ts">
  import { settingsStore } from "../settings-store.svelte";

  const mascot = $derived(settingsStore.state.mascot);

  function toggleMascot() {
    settingsStore.update("mascot", { enabled: !mascot.enabled });
  }

  function toggleTracking() {
    settingsStore.update("mascot", { cursorTracking: !mascot.cursorTracking });
  }

  function handleIntensityChange(event: Event) {
    const target = event.target as HTMLInputElement;
    settingsStore.update("mascot", { animationIntensity: Number(target.value) });
  }

  function toggleSound() {
    settingsStore.update("mascot", { soundEnabled: !mascot.soundEnabled });
  }
</script>

<div class="section-container" role="tabpanel" aria-label="Mascot Settings">
  <!-- Enable Mascot -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Enable Companion (Pip)</span>
      <span class="setting-desc">Render procedural mascot in the left notch zone</span>
    </div>
    <button
      type="button"
      class="win-switch"
      class:checked={mascot.enabled}
      role="switch"
      aria-checked={mascot.enabled}
      aria-label="Toggle Companion Mascot"
      onclick={toggleMascot}
    >
      <span class="switch-thumb"></span>
    </button>
  </div>

  <!-- Cursor Tracking -->
  <div class="setting-row" class:disabled={!mascot.enabled}>
    <div class="setting-info">
      <span class="setting-label">Cursor Awareness</span>
      <span class="setting-desc">Pupils smoothly glance toward cursor position</span>
    </div>
    <button
      type="button"
      class="win-switch"
      class:checked={mascot.cursorTracking}
      disabled={!mascot.enabled}
      role="switch"
      aria-checked={mascot.cursorTracking}
      aria-label="Toggle Cursor Tracking"
      onclick={toggleTracking}
    >
      <span class="switch-thumb"></span>
    </button>
  </div>

  <!-- Mascot Animation Intensity -->
  <div class="setting-row" class:disabled={!mascot.enabled}>
    <div class="setting-info">
      <span class="setting-label">Expressiveness Multiplier</span>
      <span class="setting-desc">Physical squash & eye gaze cadence</span>
    </div>
    <div class="slider-group">
      <input
        type="range"
        class="win-slider"
        min="0.0"
        max="1.0"
        step="0.1"
        disabled={!mascot.enabled}
        value={mascot.animationIntensity}
        oninput={handleIntensityChange}
        aria-label="Mascot Animation Intensity"
      />
      <span class="slider-label">{Math.round(mascot.animationIntensity * 100)}%</span>
    </div>
  </div>

  <!-- Sound Toggle -->
  <div class="setting-row" class:disabled={!mascot.enabled}>
    <div class="setting-info">
      <span class="setting-label">Audio Cues</span>
      <span class="setting-desc">Subtle audio chime on celebrate / focus completion</span>
    </div>
    <button
      type="button"
      class="win-switch"
      class:checked={mascot.soundEnabled}
      disabled={!mascot.enabled}
      role="switch"
      aria-checked={mascot.soundEnabled}
      aria-label="Toggle Mascot Audio Cues"
      onclick={toggleSound}
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
    min-width: 32px;
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
</style>
