<script lang="ts">
  import { batteryStore } from "./battery-store.svelte";
  import IconBattery from "../../icons/IconBattery.svelte";
  import IconX from "../../icons/IconX.svelte";
  let { onclose = () => batteryStore.closeBatterySurface() }: { onclose?: () => void } = $props();
  const battery = $derived(batteryStore.state);
</script>

<div class="tool-panel battery-panel">
  <div class="panel-header"><h2 class="panel-title">Battery</h2><button type="button" class="close-btn" onclick={onclose} aria-label="Close battery view"><IconX size={16} /></button></div>
  <div class="battery-reading" class:low={batteryStore.isLowBattery}>
    <IconBattery size={32} charging={battery.isCharging} level={battery.percentage} />
    <span class="battery-percent">{battery.hasBattery ? batteryStore.formattedPercent : "AC power"}</span>
    <span class="battery-status">{battery.isCharging ? "Charging" : batteryStore.isLowBattery ? "Low battery · Connect power" : battery.isAcConnected ? "Plugged in" : "On battery"}</span>
    {#if batteryStore.formattedRemainingTime}<span class="battery-time">{batteryStore.formattedRemainingTime} remaining</span>{/if}
  </div>
</div>

<style>
  .battery-reading { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; flex: 1; padding: 12px; }
  .battery-percent { font-size: 32px; font-weight: 600; font-variant-numeric: tabular-nums; }
  .battery-status, .battery-time { font-size: 12px; color: var(--text-secondary); }
  .low .battery-status { color: var(--status-warning); }
</style>
