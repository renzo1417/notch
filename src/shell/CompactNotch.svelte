<script lang="ts">
  interface Props {
    backendStatus: string;
    onPing: () => void;
    pinging: boolean;
  }

  let { backendStatus, onPing, pinging }: Props = $props();
</script>

<div class="compact-container">
  <div class="status-indicator" class:connected={backendStatus === "Connected"}>
    <span class="status-dot"></span>
    <span class="app-title">PROJECT_NOTCH</span>
  </div>

  <div class="divider"></div>

  <div class="backend-info">
    <span class="status-text">{backendStatus}</span>
  </div>

  <button 
    type="button" 
    class="ping-btn" 
    onclick={onPing} 
    disabled={pinging}
    aria-label="Ping Rust backend"
  >
    {pinging ? "Pinging..." : "Ping Backend"}
  </button>
</div>

<style>
  .compact-container {
    display: flex;
    align-items: center;
    gap: var(--space-8);
    height: 100%;
    padding: 0 var(--space-12);
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: var(--space-6);
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background-color: var(--text-tertiary);
    transition: background-color var(--motion-micro) var(--ease-natural);
  }

  .status-indicator.connected .status-dot {
    background-color: #22c55e;
  }

  .app-title {
    font-size: var(--font-size-label);
    font-weight: 600;
    letter-spacing: 0.5px;
    color: var(--text-primary);
  }

  .divider {
    width: 1px;
    height: 14px;
    background-color: var(--border-subtle);
  }

  .backend-info {
    font-size: var(--font-size-meta);
    color: var(--text-secondary);
  }

  .status-text {
    font-family: inherit;
  }

  .ping-btn {
    padding: var(--space-4) var(--space-8);
    border-radius: var(--radius-control);
    background-color: var(--surface-hover);
    color: var(--text-primary);
    font-size: var(--font-size-tiny);
    font-weight: 500;
    transition: background-color var(--motion-micro) var(--ease-natural);
  }

  .ping-btn:hover:not(:disabled) {
    background-color: var(--surface-active);
  }

  .ping-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
