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

<div class="section-container" role="group" aria-label="Mascot Settings">
  <!-- Enable Mascot -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Show Pip</span>
      <span class="setting-desc">Keep Pip in the companion header.</span>
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
      <span class="setting-label">Follow the pointer</span>
      <span class="setting-desc">Let Pip look toward your pointer.</span>
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
      <span class="setting-label">Expressiveness</span>
      <span class="setting-desc">Adjust how much Pip moves and reacts.</span>
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
