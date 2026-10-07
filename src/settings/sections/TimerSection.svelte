<script lang="ts">
  import { settingsStore } from "../settings-store.svelte";

  const timer = $derived(settingsStore.state.timer);

  function setPreset(preset: number) {
    settingsStore.update("timer", { defaultPreset: preset });
  }

  function handleCustomMinutesChange(event: Event) {
    const target = event.target as HTMLInputElement;
    settingsStore.update("timer", { customMinutes: Number(target.value) });
  }

  function toggleAutoStart() {
    settingsStore.update("timer", { autoStartOnPresetSelect: !timer.autoStartOnPresetSelect });
  }
</script>

<div class="section-container" role="tabpanel" aria-label="Timer Settings">
  <!-- Default Preset -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Default Focus Preset</span>
      <span class="setting-desc">Primary session length for Pomodoro / deep work</span>
    </div>
    <div class="preset-buttons" role="group" aria-label="Default Focus Presets">
      <button
        type="button"
        class="preset-btn"
        class:active={timer.defaultPreset === 25}
        onclick={() => setPreset(25)}
      >
        25m
      </button>
      <button
        type="button"
        class="preset-btn"
        class:active={timer.defaultPreset === 50}
        onclick={() => setPreset(50)}
      >
        50m
      </button>
    </div>
  </div>

  <!-- Custom Duration -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Custom Duration</span>
      <span class="setting-desc">Length for personalized custom timer sessions</span>
    </div>
    <div class="slider-group">
      <input
        type="range"
        class="win-slider"
        min="5"
        max="120"
        step="5"
        value={timer.customMinutes}
        oninput={handleCustomMinutesChange}
        aria-label="Custom Duration Minutes"
      />
      <span class="slider-label">{timer.customMinutes}m</span>
    </div>
  </div>

  <!-- Auto-Start on Selection -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Instant Start</span>
      <span class="setting-desc">Commence countdown immediately upon tapping preset</span>
    </div>
    <button
      type="button"
      class="win-switch"
      class:checked={timer.autoStartOnPresetSelect}
      role="switch"
      aria-checked={timer.autoStartOnPresetSelect}
      aria-label="Toggle Instant Start"
      onclick={toggleAutoStart}
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

  .preset-buttons {
    display: flex;
    gap: 4px;
    background: var(--surface-root);
    padding: 2px;
    border-radius: 6px;
    border: 1px solid var(--border-strong);
  }

  .preset-btn {
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

  .preset-btn:hover {
    color: var(--text-primary);
  }

  .preset-btn.active {
    background: var(--accent-primary);
    color: white;
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
