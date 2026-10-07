<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { MascotRenderer } from "./renderer";
  import { MascotStateMachine } from "./state-machine";
  import { mascotStore } from "./mascot-store.svelte";
  import { settingsStore } from "../settings/settings-store.svelte";
  import type { MascotState } from "./types";

  interface Props {
    size?: number;
    compact?: boolean;
    interactive?: boolean;
    onStateChange?: (state: MascotState) => void;
  }

  let {
    size = 64,
    compact = false,
    interactive = true,
    onStateChange,
  }: Props = $props();

  let canvasElement: HTMLCanvasElement | null = $state(null);
  let currentStateName = $state<MascotState>("idle");

  let renderer: MascotRenderer | null = null;
  let stateMachine: MascotStateMachine | null = null;
  let animationFrameId: number | null = null;
  let lastFrameTimeMs: number = 0;
  let isWindowVisible: boolean = true;
  let isHovered: boolean = $state(false);
  let unsubMascot: (() => void) | null = null;

  let initialRenderDone = false;

  const mascotConfig = $derived(settingsStore.state.mascot);
  const perfConfig = $derived(settingsStore.state.performance);
  const appearanceConfig = $derived(settingsStore.state.appearance);

  $effect(() => {
    if (stateMachine) {
      const reduced = appearanceConfig.reducedMotion || perfConfig.batterySaverMode;
      const intensity = reduced ? 0 : mascotConfig.animationIntensity;
      stateMachine.setConfig({
        enabled: mascotConfig.enabled,
        trackCursor: mascotConfig.cursorTracking,
        animationIntensity: intensity,
        pauseInactiveAnimation: perfConfig.pauseInactiveAnimations,
      });

      // Completely halt or resume rAF loop based on active enabled state
      if (!mascotConfig.enabled) {
        if (animationFrameId !== null) {
          cancelAnimationFrame(animationFrameId);
          animationFrameId = null;
        }
      } else if (isWindowVisible && animationFrameId === null) {
        lastFrameTimeMs = 0;
        initialRenderDone = false;
        animationFrameId = requestAnimationFrame(renderFrame);
      }
    }
  });

  function updateAriaState(newState: MascotState) {
    currentStateName = newState;
    onStateChange?.(newState);
  }

  function renderFrame(timeMs: number) {
    // If hidden, disabled, or unmounted, stop scheduling animation frames
    if (!canvasElement || !renderer || !stateMachine || !isWindowVisible || !mascotConfig.enabled) {
      if (isWindowVisible && mascotConfig.enabled && canvasElement) {
        animationFrameId = requestAnimationFrame(renderFrame);
      } else {
        animationFrameId = null;
      }
      return;
    }

    if (lastFrameTimeMs === 0) {
      lastFrameTimeMs = timeMs;
    }
    const dtMs = Math.min(64, timeMs - lastFrameTimeMs);
    lastFrameTimeMs = timeMs;

    // Advance simulation state
    const isDirty = stateMachine.update(dtMs);

    // Only repaint if visual elements actively changed or initial resting frame needed
    if (isDirty || !initialRenderDone) {
      const ctx = canvasElement.getContext("2d");
      if (ctx) {
        const dpr = window.devicePixelRatio || 1;
        const renderScale = compact ? size / 64 : 1.0;

        renderer.render({
          ctx,
          state: stateMachine.getState(),
          dimensions: {
            width: size,
            height: size,
            scale: renderScale,
          },
          dpr,
        });
        initialRenderDone = true;
      }
    }

    animationFrameId = requestAnimationFrame(renderFrame);
  }

  function handlePointerMove(event: MouseEvent) {
    if (!interactive || !stateMachine || !canvasElement || !mascotConfig.cursorTracking) return;

    const rect = canvasElement.getBoundingClientRect();
    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;

    const relX = event.clientX - centerX;
    const relY = event.clientY - centerY;

    const isHoveringMascot =
      event.clientX >= rect.left &&
      event.clientX <= rect.right &&
      event.clientY >= rect.top &&
      event.clientY <= rect.bottom;

    isHovered = isHoveringMascot;
    stateMachine.recordPointerMove(relX, relY, isHoveringMascot);
  }

  function handlePointerLeave() {
    if (!interactive || !stateMachine) return;
    isHovered = false;
    stateMachine.handlePointerLeave();
  }

  function handleClick(event: MouseEvent) {
    if (!interactive || !stateMachine) return;
    event.stopPropagation();
    stateMachine.recordClick();
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (!interactive || !stateMachine) return;
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      event.stopPropagation();
      stateMachine.recordClick();
    }
  }

  function handleVisibilityChange() {
    isWindowVisible = !document.hidden;
    if (isWindowVisible) {
      lastFrameTimeMs = 0;
      if (animationFrameId === null && mascotConfig.enabled) {
        initialRenderDone = false;
        animationFrameId = requestAnimationFrame(renderFrame);
      }
    } else {
      if (animationFrameId !== null) {
        cancelAnimationFrame(animationFrameId);
        animationFrameId = null;
      }
    }
  }

  function setupCanvasDpr() {
    if (!canvasElement) return;
    const dpr = window.devicePixelRatio || 1;
    canvasElement.width = Math.round(size * dpr);
    canvasElement.height = Math.round(size * dpr);
    canvasElement.style.width = `${size}px`;
    canvasElement.style.height = `${size}px`;
    initialRenderDone = false;
  }

  onMount(() => {
    renderer = new MascotRenderer();
    stateMachine = new MascotStateMachine({
      enabled: mascotConfig.enabled,
      trackCursor: mascotConfig.cursorTracking,
      animationIntensity: mascotConfig.animationIntensity,
      pauseInactiveAnimation: perfConfig.pauseInactiveAnimations,
    }, updateAriaState);

    unsubMascot = mascotStore.subscribe((state, durationMs) => {
      stateMachine?.setState(state, durationMs);
    });

    setupCanvasDpr();
    window.addEventListener("pointermove", handlePointerMove, { passive: true });
    window.addEventListener("resize", setupCanvasDpr, { passive: true });
    document.addEventListener("visibilitychange", handleVisibilityChange);

    if (mascotConfig.enabled && isWindowVisible) {
      animationFrameId = requestAnimationFrame(renderFrame);
    }
  });

  onDestroy(() => {
    if (animationFrameId !== null) {
      cancelAnimationFrame(animationFrameId);
      animationFrameId = null;
    }
    if (unsubMascot) {
      unsubMascot();
      unsubMascot = null;
    }
    window.removeEventListener("pointermove", handlePointerMove);
    window.removeEventListener("resize", setupCanvasDpr);
    document.removeEventListener("visibilitychange", handleVisibilityChange);
  });

  // Public state manipulation methods accessible via component bindings if needed
  export function triggerState(state: MascotState, durationMs?: number) {
    stateMachine?.setState(state, durationMs);
  }
</script>

{#if mascotConfig.enabled}
  {#if interactive}
    <button
      type="button"
      class="mascot-container"
      class:hovered={isHovered}
      aria-label="Desktop companion Pip. State: {currentStateName}. Click to interact."
      title="Pip · {currentStateName}"
      onclick={handleClick}
      onkeydown={handleKeyDown}
      onmouseleave={handlePointerLeave}
    >
      <canvas bind:this={canvasElement} class="mascot-canvas"></canvas>
    </button>
  {:else}
    <div
      class="mascot-container compact"
      role="img"
      aria-label="Desktop companion Pip mini avatar"
    >
      <canvas bind:this={canvasElement} class="mascot-canvas"></canvas>
    </div>
  {/if}
{:else}
  <div class="mascot-container disabled" aria-label="Companion mascot disabled">
    <div class="mascot-disabled-indicator" title="Mascot disabled in Settings"></div>
  </div>
{/if}

<style>
  .mascot-container {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    position: relative;
    user-select: none;
    cursor: pointer;
    outline: none;
    border-radius: var(--radius-control);
    transition: transform var(--motion-micro) var(--ease-natural);
  }

  .mascot-container:focus-visible {
    outline: 2px solid var(--accent-focus);
    outline-offset: 2px;
  }

  .mascot-container.compact {
    cursor: default;
  }

  .mascot-container.disabled {
    cursor: default;
    opacity: 0.5;
  }

  .mascot-disabled-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: var(--text-tertiary);
  }

  .mascot-canvas {
    display: block;
    pointer-events: none;
  }
</style>
