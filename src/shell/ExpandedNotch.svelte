<script lang="ts">
  import MascotCanvas from "../mascot/MascotCanvas.svelte";
  import MusicPlaceholder from "../modules/music/MusicPlaceholder.svelte";
  import type { MascotState } from "../mascot/types";

  interface Props {
    backendConnected: boolean;
  }

  let { backendConnected }: Props = $props();

  let mascotState = $state<MascotState>("idle");

  function handleMascotStateChange(newState: MascotState) {
    mascotState = newState;
  }
</script>

<div class="expanded-three-zone" aria-label="Expanded Notch View · Companion state: {mascotState}">
  <!-- LEFT ZONE: Procedural Mascot (Pip) -->
  <div class="zone left-zone" aria-label="Companion zone">
    <MascotCanvas
      size={64}
      compact={false}
      interactive={true}
      onStateChange={handleMascotStateChange}
    />
  </div>

  <!-- CENTER ZONE: Context Surface -->
  <div class="zone center-zone" aria-label="Context surface">
    <div class="context-container">
      <div class="context-header">
        <span class="context-category">DESKTOP COMPANION · {mascotState.toUpperCase()}</span>
        <div
          class="status-dot"
          class:connected={backendConnected}
          title={backendConnected ? "Native backend connected" : "Connecting to backend..."}
          aria-hidden="true"
        ></div>
      </div>
      <div class="context-body">
        <span class="context-title" title="Windows Notch Active">Windows Notch Active</span>
        <span class="context-meta" title="Quiet desktop companion layer">Quiet top-center companion layer</span>
      </div>
    </div>
  </div>

  <!-- RIGHT ZONE: Media Module Placeholder -->
  <div class="zone right-zone" aria-label="Music player zone">
    <MusicPlaceholder />
  </div>
</div>

<style>
  .expanded-three-zone {
    display: flex;
    align-items: center;
    width: 100%;
    height: 100%;
    user-select: none;
    box-sizing: border-box;
  }

  .zone {
    height: 100%;
    display: flex;
    align-items: center;
    box-sizing: border-box;
  }

  /* LEFT: Mascot Zone (18-20% visual balance ~ 96px) */
  .left-zone {
    width: 96px;
    justify-content: center;
    flex-shrink: 0;
    padding: 0 var(--space-8);
  }

  /* CENTER: Context Surface (42-46% visual balance ~ 248px) */
  .center-zone {
    flex: 1;
    min-width: 0; /* Enable text truncation */
    padding: 0 var(--space-12);
    border-left: 1px solid var(--border-subtle);
    border-right: 1px solid var(--border-subtle);
    justify-content: flex-start;
  }

  .context-container {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 2px;
    width: 100%;
    min-width: 0;
  }

  .context-header {
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
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .status-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background-color: var(--text-tertiary);
    transition: background-color var(--motion-micro) var(--ease-natural);
    flex-shrink: 0;
  }

  .status-dot.connected {
    background-color: #22c55e;
  }

  .context-body {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }

  .context-title {
    font-size: var(--font-size-body);
    font-weight: 600;
    color: var(--text-primary);
    line-height: var(--line-height-tight);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .context-meta {
    font-size: var(--font-size-meta);
    color: var(--text-secondary);
    line-height: var(--line-height-tight);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* RIGHT: Music Module Zone (~176px stable geometry) */
  .right-zone {
    width: 176px;
    flex-shrink: 0;
    justify-content: flex-end;
  }
</style>
