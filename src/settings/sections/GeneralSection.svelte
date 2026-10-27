<script lang="ts">
  import { settingsStore } from "../settings-store.svelte";

  const general = $derived(settingsStore.state.general);
  const monitors = $derived(settingsStore.availableMonitors);

  function toggleStartup() {
    settingsStore.update("general", { launchAtStartup: !general.launchAtStartup });
  }

  function toggleAlwaysOnTop() {
    settingsStore.update("general", { alwaysOnTop: !general.alwaysOnTop });
  }

  function handleMonitorChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    const val = target.value === "" ? null : target.value;
    settingsStore.update("general", { selectedMonitor: val });
  }

  function handleCollapseDelayChange(event: Event) {
    const target = event.target as HTMLInputElement;
    settingsStore.update("general", { autoCollapseDelayMs: Number(target.value) });
  }
</script>

<div class="section-container" role="group" aria-label="General Settings">
  <!-- Launch with Windows -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Launch with Windows</span>
      <span class="setting-desc">Start notch quietly at Windows user login</span>
    </div>
    <button
      type="button"
      class="win-switch"
      class:checked={general.launchAtStartup}
      role="switch"
      aria-checked={general.launchAtStartup}
      aria-label="Toggle Launch with Windows"
      onclick={toggleStartup}
    >
      <span class="switch-thumb"></span>
    </button>
  </div>

  <!-- Selected Monitor -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Selected Monitor</span>
      <span class="setting-desc">Screen edge where the notch is placed</span>
    </div>
    <select
      class="win-select"
      value={general.selectedMonitor ?? ""}
      onchange={handleMonitorChange}
      aria-label="Select display monitor"
    >
      <option value="">Primary Display (Default)</option>
      {#each monitors as mon}
        {#if mon.name}
          <option value={mon.name}>
            {mon.name} {mon.is_primary ? "(Primary)" : ""} · {mon.width}x{mon.height}
          </option>
        {/if}
      {/each}
    </select>
  </div>

  <!-- Always on Top -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Always on Top</span>
      <span class="setting-desc">Keep notch visible above foreground windows</span>
    </div>
    <button
      type="button"
      class="win-switch"
      class:checked={general.alwaysOnTop}
      role="switch"
      aria-checked={general.alwaysOnTop}
      aria-label="Toggle Always on Top"
      onclick={toggleAlwaysOnTop}
    >
      <span class="switch-thumb"></span>
    </button>
  </div>

  <!-- Auto-Collapse Delay -->
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Auto-Collapse Delay</span>
      <span class="setting-desc">Collapse home after the pointer leaves.</span>
    </div>
    <div class="slider-group">
      <input
        type="range"
        class="win-slider"
        min="2000"
        max="15000"
        step="1000"
        value={general.autoCollapseDelayMs}
        oninput={handleCollapseDelayChange}
        aria-label="Auto-collapse delay in seconds"
      />
      <span class="slider-label">{general.autoCollapseDelayMs / 1000}s</span>
    </div>
  </div>
</div>
