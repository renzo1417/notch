<script lang="ts">
  import MascotCanvas from "../mascot/MascotCanvas.svelte";
  import IconMusic from "../icons/IconMusic.svelte";

  interface Props {
    backendConnected: boolean;
  }

  let { backendConnected }: Props = $props();
</script>

<div class="compact-layout" aria-label="Compact Notch Shell: Windows Notch Active">
  <!-- Left: Mini companion avatar -->
  <div class="compact-mascot" aria-hidden="true">
    <MascotCanvas size={26} compact={true} interactive={false} />
  </div>

  <!-- Center: Immediate context summary -->
  <div class="compact-center">
    <div
      class="status-dot"
      class:connected={backendConnected}
      aria-hidden="true"
      title={backendConnected ? "Native backend connected" : "Connecting to backend..."}
    ></div>
    <span class="center-label">Windows Notch</span>
  </div>

  <!-- Right: Subdued media indicator -->
  <div class="compact-media" aria-hidden="true" title="No audio playing">
    <IconMusic size={13} color="var(--text-tertiary)" />
  </div>
</div>

<style>
  .compact-layout {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    height: 100%;
    user-select: none;
    padding: 0 var(--space-12);
    box-sizing: border-box;
  }

  .compact-mascot {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    flex-shrink: 0;
  }

  .compact-center {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-6);
    flex: 1;
    min-width: 0;
    padding: 0 var(--space-8);
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

  .center-label {
    font-size: var(--font-size-label);
    font-weight: 600;
    letter-spacing: 0.2px;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .compact-media {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    flex-shrink: 0;
    opacity: 0.6;
  }
</style>
