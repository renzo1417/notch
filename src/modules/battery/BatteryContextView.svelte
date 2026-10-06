<script lang="ts">
  import { batteryStore } from "./battery-store.svelte";
  import IconBattery from "../../icons/IconBattery.svelte";
  import IconAlertCircle from "../../icons/IconAlertCircle.svelte";
  import IconX from "../../icons/IconX.svelte";

  const battery = $derived(batteryStore.state);
  const remaining = $derived(batteryStore.formattedRemainingTime);
</script>

<div class="battery-container" aria-label="System Battery Status Surface">
  <!-- Header row -->
  <div class="battery-header">
    <div class="header-left">
      <IconBattery
        size={12}
        color={batteryStore.isLowBattery ? "#ef4444" : "var(--text-tertiary)"}
        charging={battery.isCharging}
        level={battery.percentage}
      />
      <span class="battery-category">
        SYSTEM POWER · {battery.hasBattery
          ? battery.isCharging
            ? "CHARGING"
            : battery.isAcConnected
            ? "AC POWER"
            : "BATTERY"
          : "DESKTOP"}
      </span>
    </div>

    <button
      type="button"
      class="icon-btn"
      onclick={(e) => {
        e.stopPropagation();
        batteryStore.closeBatterySurface();
      }}
      title="Close battery view"
      aria-label="Close battery view"
    >
      <IconX size={10} color="var(--text-tertiary)" />
    </button>
  </div>

  <!-- Content body -->
  <div class="battery-body">
    <div class="primary-row">
      <span class="percent-label">{battery.percentage}%</span>

      {#if battery.isCharging}
        <span class="status-badge charging">Charging</span>
      {:else if batteryStore.isLowBattery}
        <span class="status-badge warning">
          <IconAlertCircle size={9} color="#ef4444" />
          <span>Low</span>
        </span>
      {:else if battery.isAcConnected}
        <span class="status-badge ac">AC Online</span>
      {:else if battery.hasBattery}
        <span class="status-badge discharge">Battery</span>
      {/if}
    </div>

    <!-- Metadata row: ONLY real Windows data, never fabricated -->
    <div class="meta-row">
      {#if remaining}
        <span class="meta-text" title="{remaining} estimated remaining">
          {remaining} remaining
        </span>
      {:else if battery.isCharging}
        <span class="meta-text">Power adapter connected</span>
      {:else if battery.isAcConnected}
        <span class="meta-text">Running on external AC power</span>
      {:else if battery.hasBattery}
        <span class="meta-text">Discharging · Time unknown</span>
      {:else}
        <span class="meta-text">Desktop power system</span>
      {/if}
    </div>
  </div>
</div>

<style>
  .battery-container {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 2px;
    width: 100%;
    min-width: 0;
  }

  .battery-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-6);
    line-height: var(--line-height-tight);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
  }

  .battery-category {
    font-size: var(--font-size-tiny);
    font-weight: 600;
    letter-spacing: 0.6px;
    color: var(--text-tertiary);
    text-transform: uppercase;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    background: transparent;
    border: none;
    border-radius: 3px;
    cursor: pointer;
    padding: 0;
    transition: background-color var(--motion-micro) var(--ease-natural);
  }

  .icon-btn:hover {
    background-color: var(--surface-hover);
  }

  .battery-body {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }

  .primary-row {
    display: flex;
    align-items: center;
    gap: var(--space-6);
  }

  .percent-label {
    font-size: var(--font-size-body);
    font-weight: 700;
    color: var(--text-primary);
    line-height: var(--line-height-tight);
    letter-spacing: -0.2px;
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: var(--font-size-tiny);
    font-weight: 600;
    padding: 1px 5px;
    border-radius: 3px;
    line-height: 1.2;
  }

  .status-badge.charging {
    background: rgba(34, 197, 94, 0.15);
    color: #22c55e;
    border: 1px solid rgba(34, 197, 94, 0.3);
  }

  .status-badge.warning {
    background: rgba(239, 68, 68, 0.15);
    color: #ef4444;
    border: 1px solid rgba(239, 68, 68, 0.3);
  }

  .status-badge.ac {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-secondary);
    border: 1px solid var(--border-subtle);
  }

  .status-badge.discharge {
    background: rgba(56, 189, 248, 0.12);
    color: #38bdf8;
    border: 1px solid rgba(56, 189, 248, 0.25);
  }

  .meta-row {
    display: flex;
    align-items: center;
  }

  .meta-text {
    font-size: var(--font-size-meta);
    color: var(--text-secondary);
    line-height: var(--line-height-tight);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
