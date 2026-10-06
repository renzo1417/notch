<script lang="ts">
  import { timerStore } from "./timer-store.svelte";
  import IconPlay from "../../icons/IconPlay.svelte";
  import IconPause from "../../icons/IconPause.svelte";
  import IconRotateCcw from "../../icons/IconRotateCcw.svelte";

  const timer = $derived(timerStore.state);
  const formattedTime = $derived(timerStore.formattedTime);
  const activePreset = $derived(timerStore.activePreset);

  let customInputVal = $state(15);
  let isEditingCustom = $state(false);

  $effect(() => {
    if (!isEditingCustom) {
      customInputVal = timer.customMinutes;
    }
  });

  function handlePresetClick(e: MouseEvent, preset: 25 | 50) {
    e.stopPropagation();
    isEditingCustom = false;
    timerStore.setPreset(preset);
  }

  function handleCustomTabClick(e: MouseEvent) {
    e.stopPropagation();
    isEditingCustom = !isEditingCustom;
    if (isEditingCustom) {
      customInputVal = timer.customMinutes;
    }
  }

  function applyCustomMinutes(e: Event) {
    e.stopPropagation();
    const parsed = parseInt(String(customInputVal), 10);
    if (!isNaN(parsed) && parsed > 0) {
      timerStore.setCustomDuration(parsed);
    }
    isEditingCustom = false;
  }

  function handleStart(e: MouseEvent) {
    e.stopPropagation();
    isEditingCustom = false;
    timerStore.start();
  }

  function handlePause(e: MouseEvent) {
    e.stopPropagation();
    timerStore.pause();
  }

  function handleResume(e: MouseEvent) {
    e.stopPropagation();
    timerStore.resume();
  }

  function handleReset(e: MouseEvent) {
    e.stopPropagation();
    isEditingCustom = false;
    timerStore.reset();
  }

  function handleCloseSurface(e: MouseEvent) {
    e.stopPropagation();
    timerStore.closeTimerSurface();
  }
</script>

<div class="timer-context" aria-label="Focus timer: {formattedTime} remaining. Status: {timer.isRunning ? (timer.isPaused ? 'Paused' : 'Running') : 'Ready'}">
  <!-- Top header row: Category, preset chips, and status dot -->
  <div class="timer-header">
    <div class="header-left">
      <span class="context-category">
        FOCUS {timer.isPaused ? "· PAUSED" : timer.isRunning ? "· ACTIVE" : "· READY"}
      </span>
      <div
        class="focus-dot"
        class:running={timer.isRunning && !timer.isPaused}
        class:paused={timer.isPaused}
        aria-hidden="true"
      ></div>
    </div>

    <!-- Presets (25m / 50m / Custom) -->
    {#if !timer.isRunning}
      <div class="presets-row" role="group" aria-label="Timer presets">
        <button
          type="button"
          class="preset-pill"
          class:active={activePreset === "25" && !isEditingCustom}
          onclick={(e) => handlePresetClick(e, 25)}
          title="25 minute preset"
          aria-label="25 minutes"
        >
          25m
        </button>
        <button
          type="button"
          class="preset-pill"
          class:active={activePreset === "50" && !isEditingCustom}
          onclick={(e) => handlePresetClick(e, 50)}
          title="50 minute preset"
          aria-label="50 minutes"
        >
          50m
        </button>
        <button
          type="button"
          class="preset-pill"
          class:active={activePreset === "custom" || isEditingCustom}
          onclick={handleCustomTabClick}
          title="Custom duration"
          aria-label="Custom duration"
        >
          {activePreset === "custom" && !isEditingCustom ? `${timer.presetMinutes}m` : "Custom"}
        </button>
        <button
          type="button"
          class="close-btn"
          onclick={handleCloseSurface}
          title="Close timer"
          aria-label="Close timer"
        >
          ×
        </button>
      </div>
    {:else}
      <span class="active-badge" title="Session duration: {timer.presetMinutes} minutes">
        {timer.presetMinutes}M
      </span>
    {/if}
  </div>

  <!-- Bottom row: Tabular Time Readout and Tactile Controls -->
  <div class="timer-body">
    {#if isEditingCustom && !timer.isRunning}
      <!-- Inline Custom Duration Configuration -->
      <form class="custom-form" onsubmit={applyCustomMinutes}>
        <input
          type="number"
          class="custom-input"
          bind:value={customInputVal}
          min="1"
          max="180"
          onclick={(e) => e.stopPropagation()}
          aria-label="Custom minutes (1-180)"
        />
        <span class="custom-unit">min</span>
        <button
          type="submit"
          class="ctrl-btn set-btn"
          title="Set duration"
          aria-label="Set duration"
        >
          Set
        </button>
      </form>
    {:else}
      <!-- Main Tabular Countdown Readout -->
      <span
        class="timer-readout"
        class:paused={timer.isPaused}
        title="Time remaining: {formattedTime}"
      >
        {formattedTime}
      </span>
    {/if}

    <!-- Action Buttons -->
    <div class="controls-row">
      {#if !timer.isRunning}
        <button
          type="button"
          class="ctrl-btn primary-btn"
          onclick={handleStart}
          title="Start focus countdown ({timer.presetMinutes}m)"
          aria-label="Start timer"
        >
          <IconPlay size={10} color="#0c0c0d" />
          <span>Start</span>
        </button>
      {:else if timer.isPaused}
        <button
          type="button"
          class="ctrl-btn primary-btn"
          onclick={handleResume}
          title="Resume timer"
          aria-label="Resume timer"
        >
          <IconPlay size={10} color="#0c0c0d" />
          <span>Resume</span>
        </button>
        <button
          type="button"
          class="ctrl-btn reset-btn"
          onclick={handleReset}
          title="Reset timer"
          aria-label="Reset timer"
        >
          <IconRotateCcw size={11} color="var(--text-secondary)" />
        </button>
      {:else}
        <button
          type="button"
          class="ctrl-btn secondary-btn"
          onclick={handlePause}
          title="Pause timer"
          aria-label="Pause timer"
        >
          <IconPause size={10} color="var(--text-primary)" />
          <span>Pause</span>
        </button>
        <button
          type="button"
          class="ctrl-btn reset-btn"
          onclick={handleReset}
          title="Stop and reset timer"
          aria-label="Stop timer"
        >
          <IconRotateCcw size={11} color="var(--text-secondary)" />
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  .timer-context {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 3px;
    width: 100%;
    min-width: 0;
    user-select: none;
    box-sizing: border-box;
  }

  .timer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-width: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: var(--space-6);
  }

  .context-category {
    font-size: var(--font-size-tiny);
    font-weight: 600;
    letter-spacing: 0.6px;
    color: var(--text-tertiary);
    text-transform: uppercase;
    line-height: var(--line-height-tight);
    white-space: nowrap;
  }

  .focus-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background-color: var(--text-tertiary);
    transition: background-color var(--motion-micro) var(--ease-natural);
    flex-shrink: 0;
  }

  .focus-dot.running {
    background-color: #2dd4bf; /* Ice teal matching Pip's focused visor */
  }

  .focus-dot.paused {
    background-color: #f59e0b; /* Amber */
  }

  .presets-row {
    display: flex;
    align-items: center;
    gap: 3px;
  }

  .preset-pill {
    background: transparent;
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 1px 5px;
    font-size: var(--font-size-tiny);
    font-weight: 500;
    color: var(--text-secondary);
    cursor: pointer;
    line-height: 1.2;
    transition: all var(--motion-micro) var(--ease-natural);
  }

  .preset-pill:hover {
    background-color: var(--surface-hover);
    color: var(--text-primary);
  }

  .preset-pill.active {
    background-color: var(--surface-raised);
    border-color: rgba(45, 212, 191, 0.4);
    color: #2dd4bf;
    font-weight: 600;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    font-size: 13px;
    line-height: 1;
    padding: 0 3px;
    cursor: pointer;
    border-radius: 3px;
    transition: color var(--motion-micro) var(--ease-natural);
  }

  .close-btn:hover {
    color: var(--text-primary);
  }

  .active-badge {
    font-size: var(--font-size-tiny);
    font-weight: 600;
    letter-spacing: 0.4px;
    color: var(--text-tertiary);
    padding: 1px 4px;
    background: var(--surface-raised);
    border-radius: 3px;
    border: 1px solid var(--border-subtle);
  }

  .timer-body {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-width: 0;
    height: 28px;
  }

  .timer-readout {
    font-size: 18px;
    font-weight: 700;
    letter-spacing: 0.5px;
    color: var(--text-primary);
    line-height: 1;
    font-variant-numeric: tabular-nums;
    font-family: var(--font-family);
  }

  .timer-readout.paused {
    color: #f59e0b;
    opacity: 0.9;
  }

  .custom-form {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .custom-input {
    width: 44px;
    height: 22px;
    background: var(--surface-raised);
    border: 1px solid var(--border-strong);
    border-radius: 4px;
    color: var(--text-primary);
    font-size: var(--font-size-meta);
    font-weight: 600;
    text-align: center;
    outline: none;
    padding: 0 2px;
    box-sizing: border-box;
  }

  .custom-input:focus {
    border-color: #2dd4bf;
  }

  .custom-unit {
    font-size: var(--font-size-tiny);
    color: var(--text-secondary);
  }

  .controls-row {
    display: flex;
    align-items: center;
    gap: var(--space-6);
  }

  .ctrl-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    height: 22px;
    padding: 0 8px;
    border-radius: 5px;
    font-size: var(--font-size-meta);
    font-weight: 600;
    cursor: pointer;
    border: none;
    transition: background-color var(--motion-micro) var(--ease-natural),
                transform var(--motion-micro) var(--ease-natural),
                opacity var(--motion-micro) var(--ease-natural);
  }

  .ctrl-btn:active {
    transform: scale(0.96);
  }

  .primary-btn {
    background-color: #2dd4bf;
    color: #0c0c0d;
  }

  .primary-btn:hover {
    background-color: #5eead4;
  }

  .secondary-btn {
    background-color: var(--surface-raised);
    border: 1px solid var(--border-subtle);
    color: var(--text-primary);
  }

  .secondary-btn:hover {
    background-color: var(--surface-hover);
  }

  .reset-btn {
    width: 22px;
    height: 22px;
    padding: 0;
    background-color: transparent;
    border: 1px solid var(--border-subtle);
    border-radius: 5px;
  }

  .reset-btn:hover {
    background-color: var(--surface-hover);
  }

  .set-btn {
    height: 22px;
    padding: 0 6px;
    font-size: var(--font-size-tiny);
    background-color: var(--surface-raised);
    border: 1px solid var(--border-subtle);
    color: var(--text-primary);
  }
</style>
