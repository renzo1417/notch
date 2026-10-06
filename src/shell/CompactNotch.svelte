<script lang="ts">
  interface Props {
    backendConnected: boolean;
  }

  let { backendConnected }: Props = $props();
</script>

<div class="compact-layout" aria-label="Compact Notch Shell">
  <div 
    class="status-dot" 
    class:connected={backendConnected} 
    aria-hidden="true"
    title={backendConnected ? "Backend connected" : "Connecting..."}
  ></div>
  <span class="center-label">PROJECT_NOTCH · Compact</span>
  <span class="sub-label">Click to expand</span>
</div>

<style>
  .compact-layout {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-8);
    width: 100%;
    height: 100%;
    user-select: none;
    padding: 0 var(--space-12);
  }

  .status-dot {
    width: 6px;
    height: 6px;
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
    letter-spacing: 0.3px;
    color: var(--text-primary);
    white-space: nowrap;
  }

  .sub-label {
    font-size: var(--font-size-tiny);
    color: var(--text-tertiary);
    white-space: nowrap;
  }
</style>
