<script lang="ts">
  import { timerStore } from "./timer-store.svelte";
  import IconPlay from "../../icons/IconPlay.svelte";
  import IconPause from "../../icons/IconPause.svelte";
  import IconRotateCcw from "../../icons/IconRotateCcw.svelte";
  import IconX from "../../icons/IconX.svelte";

  interface Props {
    onclose: () => void;
  }

  let { onclose }: Props = $props();

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
    e.preventDefault();
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
</script>

<div class="tool-panel focus-panel" aria-label="Focus timer controls">
  <!-- Header -->
  <div class="panel-header">
    <div class="header-left">
      <span class="panel-title">Focus</span>
      {#if timer.isRunning}
        <div class="status-indicator" class:paused={timer.isPaused}></div>
        <span class="status-label">{timer.isPaused ? "Paused" : "Running"}</span>
      {/if}
    </div>

    <button
      type="button"
      class="close-btn"
      onclick={(e) => {
        e.stopPropagation();
        onclose();
      }}
      title="Close Focus panel (Esc)"
      aria-label="Close Focus panel"
    >
      <IconX size={16} color="var(--text-tertiary)" />
    </button>
  </div>

  <!-- Body -->
  <div class="panel-body">
    <!-- Presets bar -->
    {#if !timer.isRunning}
      <div class="presets-row" role="group" aria-label="Timer presets">
        <button
          type="button"
          class="preset-btn"
          class:active={activePreset === "25" && !isEditingCustom}
          onclick={(e) => handlePresetClick(e, 25)}
        >
          25m
        </button>
        <button
          type="button"
          class="preset-btn"
          class:active={activePreset === "50" && !isEditingCustom}
          onclick={(e) => handlePresetClick(e, 50)}
        >
          50m
        </button>
        <button
          type="button"
          class="preset-btn"
          class:active={activePreset === "custom" || isEditingCustom}
          onclick={handleCustomTabClick}
        >
          {activePreset === "custom" && !isEditingCustom ? `${timer.presetMinutes}m` : "Custom"}
        </button>
      </div>

      {#if isEditingCustom}
        <form class="custom-form" onsubmit={applyCustomMinutes}>
          <input
            type="number"
            class="custom-input"
            bind:value={customInputVal}
            min="1"
            max="180"
            onclick={(e) => e.stopPropagation()}
            aria-label="Minutes"
          />
          <span class="custom-unit">min</span>
          <button type="submit" class="custom-set-btn">Set</button>
        </form>
      {/if}
    {/if}

    <!-- Time Readout -->
    <div class="time-display" aria-label="Time remaining">
      <span class="time-digits">{formattedTime}</span>
    </div>

    <!-- Action Controls -->
    <div class="controls-row">
      {#if !timer.isRunning}
        <button
          type="button"
          class="main-action-btn start"
          onclick={handleStart}
          aria-label="Start Focus"
        >
          <IconPlay size={16} color="var(--on-accent)" />
          <span>Start</span>
        </button>
      {:else}
        {#if timer.isPaused}
          <button
            type="button"
            class="main-action-btn resume"
            onclick={handleResume}
            aria-label="Resume Focus"
          >
            <IconPlay size={16} color="var(--on-accent)" />
            <span>Resume</span>
          </button>
        {:else}
          <button
            type="button"
            class="main-action-btn pause"
            onclick={handlePause}
            aria-label="Pause Focus"
          >
            <IconPause size={16} color="var(--on-accent)" />
            <span>Pause</span>
          </button>
        {/if}

        <button
          type="button"
          class="reset-btn"
          onclick={handleReset}
          title="Reset timer"
          aria-label="Reset timer"
        >
          <IconRotateCcw size={16} color="var(--text-secondary)" />
        </button>
      {/if}
    </div>
  </div>
</div>
