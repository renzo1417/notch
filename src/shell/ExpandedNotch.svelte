<script lang="ts">
  interface Props {
    backendConnected: boolean;
  }

  let { backendConnected }: Props = $props();
</script>

<div class="expanded-layout" aria-label="Expanded Notch Shell">
  <div class="center-content">
    <div class="header-row">
      <div 
        class="status-dot" 
        class:connected={backendConnected} 
        aria-hidden="true"
        title={backendConnected ? "Backend connected" : "Connecting..."}
      ></div>
      <span class="center-label">PROJECT_NOTCH · Expanded</span>
    </div>
    <span class="sub-label">Top-center layout verification · Click to collapse</span>
  </div>
</div>

<style>
  .expanded-layout {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    padding: 0 var(--space-16);
    user-select: none;
  }

  .center-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-4);
  }

  .header-row {
    display: flex;
    align-items: center;
    gap: var(--space-8);
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
    font-size: var(--font-size-body);
    font-weight: 600;
    letter-spacing: 0.3px;
    color: var(--text-primary);
    white-space: nowrap;
  }

  .sub-label {
    font-size: var(--font-size-meta);
    color: var(--text-secondary);
    white-space: nowrap;
  }
</style>
