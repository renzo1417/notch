<script lang="ts">
  interface Props {
    backendDetails?: string;
  }

  let { backendDetails = "Ready" }: Props = $props();
</script>

<div class="expanded-placeholder" role="region" aria-label="Expanded Notch Placeholder">
  <div class="zone zone-left">
    <span class="zone-label">Mascot</span>
  </div>
  <div class="zone zone-center">
    <span class="zone-label">Context Surface</span>
    <span class="zone-sub">{backendDetails}</span>
  </div>
  <div class="zone zone-right">
    <span class="zone-label">Music</span>
  </div>
</div>

<style>
  .expanded-placeholder {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    height: 100%;
    padding: var(--space-8) var(--space-12);
    gap: var(--space-12);
  }

  .zone {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    border: 1px dashed var(--border-subtle);
    border-radius: var(--radius-control);
    padding: var(--space-6) var(--space-12);
    background-color: var(--surface-hover);
  }

  .zone-left {
    flex: 1;
  }

  .zone-center {
    flex: 2;
  }

  .zone-right {
    flex: 1.5;
  }

  .zone-label {
    font-size: var(--font-size-label);
    font-weight: 500;
    color: var(--text-secondary);
  }

  .zone-sub {
    font-size: var(--font-size-tiny);
    color: var(--text-tertiary);
  }
</style>
