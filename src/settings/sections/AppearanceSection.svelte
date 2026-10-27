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

<div class="section-container" role="group" aria-label="Appearance Settings">
  <!-- Accent color -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Accent color</span>
      <span class="setting-desc">Highlight shade used across controls and focus states</span>
    </div>
    <div class="accent-palette" role="radiogroup" aria-label="Accent color Palette">
      {#each ACCENT_COLOR_PRESETS as preset}
        <button
          type="button"
          class="accent-chip"
          role="radio"
          aria-checked={appearance.accentColor.toLowerCase() === preset.value.toLowerCase()}
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

  <!-- Notch Surface opacity -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Surface opacity</span>
      <span class="setting-desc">Adjust how much of your wallpaper shows through.</span>
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
        aria-label="Surface opacity"
      />
      <span class="slider-label">{Math.round(appearance.opacity * 100)}%</span>
    </div>
  </div>

  <!-- Animation intensity -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Animation intensity</span>
      <span class="setting-desc">Adjust how much the notch and Pip move.</span>
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
        aria-label="Animation intensity"
      />
      <span class="slider-label">{Math.round(appearance.animationIntensity * 100)}%</span>
    </div>
  </div>

  <!-- Reduced motion -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Reduced motion</span>
      <span class="setting-desc">Disable motion transitions for comfort & accessibility</span>
    </div>
    <button
      type="button"
      class="win-switch"
      class:checked={appearance.reducedMotion}
      role="switch"
      aria-checked={appearance.reducedMotion}
      aria-label="Toggle Reduced motion"
      onclick={toggleReducedMotion}
    >
      <span class="switch-thumb"></span>
    </button>
  </div>
</div>
