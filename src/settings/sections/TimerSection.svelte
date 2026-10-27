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

<div class="section-container" role="group" aria-label="Timer Settings">
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
