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

<div class="section-container" role="group" aria-label="Music Settings">
  <!-- Enable Music Module -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Music player</span>
      <span class="setting-desc">Show the current track and available playback controls.</span>
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
      <span class="setting-label">Playback controls</span>
      <span class="setting-desc">Show previous, play/pause, and next buttons.</span>
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
