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

<div class="section-container" role="tabpanel" aria-label="General Settings">
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
      <span class="setting-desc">Return to compact view after inactivity</span>
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

  /* Compact Windows Switch */
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

  /* Select */
  .win-select {
    background: var(--surface-root);
    color: var(--text-primary);
    border: 1px solid var(--border-strong);
    border-radius: 6px;
    padding: 4px 8px;
    font-size: var(--font-size-meta);
    font-family: inherit;
    cursor: pointer;
    outline: none;
    max-width: 200px;
  }

  .win-select:focus {
    border-color: var(--accent-primary);
  }

  /* Slider */
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
    min-width: 28px;
    text-align: right;
  }
</style>
