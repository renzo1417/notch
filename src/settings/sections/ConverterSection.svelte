<script lang="ts">
  import { settingsStore } from "../settings-store.svelte";
  import { converterStore as converter } from "../../modules/converter/converter-store.svelte";
</script>
<div class="section-container">
  <div class="setting-row"><div class="setting-info"><span class="setting-label">File conversion</span><span class="setting-desc">Drop files onto the Notch. Filee runs only when needed.</span></div><button type="button" class="win-switch" role="switch" disabled={converter.busy} class:checked={settingsStore.state.converter.enabled} aria-checked={settingsStore.state.converter.enabled} aria-label="Enable file conversion" onclick={() => settingsStore.update("converter", { enabled: !settingsStore.state.converter.enabled })}><span class="switch-thumb"></span></button></div>
  <div class="setting-row"><div class="setting-info"><span class="setting-label">Filee CLI</span><span class="setting-desc" title={settingsStore.state.converter.cliPath ?? "Automatic discovery"}>{converter.availability.message || "Find automatically, or locate filee-cli.exe."}</span></div><button type="button" class="action-button" disabled={converter.busy || converter.loading} onclick={() => converter.locate()}>Locate</button><button type="button" class="action-button" disabled={converter.busy || converter.loading} onclick={() => converter.check(true)}>Check again</button></div>
  {#if settingsStore.state.converter.cliPath}<button type="button" class="action-button" disabled={converter.busy || converter.loading} onclick={() => converter.useAutomatic()}>Use automatic discovery</button>{/if}
  <p class="setting-desc">One file at a time · 512 MiB converter limit · originals preserved.</p>
  {#if converter.error}<p role="alert" class="converter-setting-error">{converter.error}</p>{/if}
</div>
<style>.converter-setting-error { color: var(--status-error); font-size: 12px; }</style>
