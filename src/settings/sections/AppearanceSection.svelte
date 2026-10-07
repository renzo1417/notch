<script lang="ts">
  import { settingsStore } from "../settings-store.svelte";
  import { ACCENT_COLOR_PRESETS } from "../types";

  const appearance = $derived(settingsStore.state.appearance);

  function selectAccent(color: string) {
    settingsStore.update("appearance", { accentColor: color });
  }

  function handleCustomColorInput(event: Event) {
    const target = event.target as HTMLInputElement;
    settingsStore.update("appearance", { accentColor: target.value });
  }

  function handleOpacityChange(event: Event) {
    const target = event.target as HTMLInputElement;
    settingsStore.update("appearance", { opacity: Number(target.value) });
  }

  function handleIntensityChange(event: Event) {
    const target = event.target as HTMLInputElement;
    settingsStore.update("appearance", { animationIntensity: Number(target.value) });
  }

  function toggleReducedMotion() {
    settingsStore.update("appearance", { reducedMotion: !appearance.reducedMotion });
  }
</script>

<div class="section-container" role="tabpanel" aria-label="Appearance Settings">
  <!-- Accent Color -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Accent Color</span>
      <span class="setting-desc">Highlight shade used across controls and focus states</span>
    </div>
    <div class="accent-palette" role="radiogroup" aria-label="Accent Color Palette">
      {#each ACCENT_COLOR_PRESETS as preset}
        <button
          type="button"
          class="accent-chip"
          class:selected={appearance.accentColor.toLowerCase() === preset.value.toLowerCase()}
          style="background-color: {preset.value};"
          title={preset.label}
          aria-label={preset.label}
          onclick={() => selectAccent(preset.value)}
        ></button>
      {/each}
      <label class="custom-color-label" title="Custom Hex Color">
        <input
          type="color"
          class="custom-color-input"
          value={appearance.accentColor}
          oninput={handleCustomColorInput}
          aria-label="Pick custom color"
        />
      </label>
    </div>
  </div>

  <!-- Notch Surface Opacity -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Surface Opacity</span>
      <span class="setting-desc">Restrained background translucency</span>
    </div>
    <div class="slider-group">
      <input
        type="range"
        class="win-slider"
        min="0.80"
        max="1.00"
        step="0.02"
        value={appearance.opacity}
        oninput={handleOpacityChange}
        aria-label="Surface Opacity"
      />
      <span class="slider-label">{Math.round(appearance.opacity * 100)}%</span>
    </div>
  </div>

  <!-- Animation Intensity -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Animation Intensity</span>
      <span class="setting-desc">Tactile transitions and mascot motion multiplier</span>
    </div>
    <div class="slider-group">
      <input
        type="range"
        class="win-slider"
        min="0.0"
        max="1.0"
        step="0.1"
        value={appearance.animationIntensity}
        oninput={handleIntensityChange}
        aria-label="Animation Intensity"
      />
      <span class="slider-label">{Math.round(appearance.animationIntensity * 100)}%</span>
    </div>
  </div>

  <!-- Reduced Motion -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Reduced Motion</span>
      <span class="setting-desc">Disable motion transitions for comfort & accessibility</span>
    </div>
    <button
      type="button"
      class="win-switch"
      class:checked={appearance.reducedMotion}
      role="switch"
      aria-checked={appearance.reducedMotion}
      aria-label="Toggle Reduced Motion"
      onclick={toggleReducedMotion}
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

  .accent-palette {
    display: flex;
    align-items: center;
    gap: 5px;
    flex-shrink: 0;
  }

  .accent-chip {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    padding: 0;
    transition: transform var(--motion-micro) var(--ease-natural),
                border-color var(--motion-micro) var(--ease-natural);
  }

  .accent-chip:hover {
    transform: scale(1.15);
  }

  .accent-chip.selected {
    border-color: white;
    transform: scale(1.15);
  }

  .custom-color-label {
    position: relative;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    cursor: pointer;
    overflow: hidden;
    border: 1px solid var(--border-strong);
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .custom-color-input {
    position: absolute;
    top: -5px;
    left: -5px;
    width: 30px;
    height: 30px;
    opacity: 0;
    cursor: pointer;
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
