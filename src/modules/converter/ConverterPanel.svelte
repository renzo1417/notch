<script lang="ts">
  import { onMount } from "svelte";
  import { converterStore as converter } from "./converter-store.svelte";
  import { FORMAT_NAMES, fileStatus, formatBytes } from "./types";
  import { settingsStore } from "../../settings/settings-store.svelte";
  import IconConvert from "../../icons/IconConvert.svelte";
  const results = $derived(converter.snapshot.files.length > 0);
  const files = $derived(results ? converter.snapshot.files : converter.files);
  onMount(() => { if (!converter.availability.status) void converter.check(); });
</script>

<section class="converter-panel" aria-label="Convert files">
  <div class="converter-heading"><div class="converter-title"><IconConvert /><h2>Convert files</h2></div><div class="heading-actions">{#if converter.availability.status === "ready" && settingsStore.state.converter.enabled}<button type="button" disabled={converter.busy || converter.loading} onclick={() => converter.pickFiles()}>Choose files</button>{#if files.length}<button type="button" disabled={converter.busy || converter.loading} onclick={() => converter.clear()}>Clear</button>{/if}{:else}<span class="local-label">On your PC</span>{/if}</div></div>
  {#if !settingsStore.state.converter.enabled}
    <div class="setup"><h3>File conversion is paused</h3><p>Enable it in Settings → Tools to drop or choose files.</p></div>
  {:else if converter.availability.status !== "ready"}
    <div class="setup">
      <span class="setup-icon"><IconConvert size={24} /></span>
      <h3>{converter.loading ? "Checking Filee…" : "Connect Filee once"}</h3>
      <p>{converter.availability.message || "Use Filee for offline image, PDF, document, and text conversion."}</p>
      <div class="setup-actions"><button type="button" class="filled" disabled={converter.loading} onclick={() => converter.locate()}>Locate Filee</button><button type="button" disabled={converter.loading} onclick={() => converter.check(true)}>Check again</button><button type="button" onclick={() => converter.install()}>Get Filee</button></div>
      <small>Filee runs only while needed. Original files stay untouched.</small>
    </div>
  {:else}
    <div class="selection-list">
      {#if files.length}
        {#each files as file (file.path)}
          <div class="file-row" class:failed={file.state === "failed"}>
            <span class="file-kind">{file.extension.slice(0, 4).toUpperCase()}</span>
            <div class="file-info"><span class="file-name" title={file.path}>{file.name}</span><span class="file-meta">{file.error || `${formatBytes(file.bytes)}${results ? ` · ${fileStatus(file)}` : ""}`}</span></div>
            {#if file.state === "done"}<button type="button" onclick={() => converter.open(file)}>Open</button><button type="button" title="Show in folder" onclick={() => converter.open(file, true)}>Folder</button>
            {:else if !results}<button type="button" aria-label={`Remove ${file.name}`} disabled={converter.loading} onclick={() => converter.remove(file.path)}>Remove</button>{/if}
          </div>
          {#if file.state === "running" && file.progress !== null}<progress aria-label={`Converting ${file.name}`} max="1" value={file.progress}></progress>{/if}
        {/each}
      {:else}
        <div class="drop-empty"><span class="empty-icon"><IconConvert size={26} /></span><h3>Drop a file. Pick its next format.</h3><p>Drag up to 20 files onto the Notch, or choose files above.</p></div>
      {/if}
    </div>
    {#if !results && files.length}
      <div class="format-row"><label for="conversion-format">Convert to</label><select id="conversion-format" class="win-select" bind:value={converter.target} disabled={converter.loading || converter.busy || !converter.targets.length}>{#each converter.targets as target}<option value={target}>{FORMAT_NAMES[target] ?? target.toUpperCase()}</option>{/each}</select><button type="button" class="filled" disabled={converter.loading || converter.busy || !converter.target} onclick={() => converter.start()}>{converter.starting ? "Starting…" : "Convert"}</button></div>
      {#if !converter.targets.length}<p class="hint">No shared output format. Remove a file to convert a smaller selection.</p>{/if}
      <div class="destination"><span title={converter.outputFolder ?? "Converted folder next to each original"}>Save to: {converter.outputFolder?.split(/[\\/]/).pop() ?? "Converted, next to originals"}</span><button type="button" disabled={converter.loading || converter.busy} onclick={() => converter.chooseFolder()}>Change</button>{#if converter.outputFolder}<button type="button" onclick={() => converter.outputFolder = null}>Reset</button>{/if}</div>
      {#if converter.unavailableTargets.length}<details class="hint"><summary>More formats need additional engines</summary><p>Install engines in Filee for {converter.unavailableTargets.map(format => FORMAT_NAMES[format] ?? format).join(", ")}, then <button type="button" onclick={() => converter.check(true)} disabled={converter.loading}>check again</button>.</p></details>{/if}
    {:else if results}
      <div class="result-footer"><span role="status">{converter.snapshot.message}</span>{#if converter.snapshot.running}<button type="button" disabled={converter.cancelling} onclick={() => converter.cancel()}>{converter.cancelling ? "Cancelling…" : "Cancel"}</button>{:else if files.some(file => ["failed", "cancelled"].includes(file.state))}<button type="button" onclick={() => converter.retry()}>Retry failed</button>{/if}</div>
    {/if}
    {#if !files.length && converter.recent.length}<div class="recent"><span>Recent results</span>{#each converter.recent as file, index (`${file.path}-${index}`)}<button type="button" onclick={() => converter.open(file)} title={file.outputs[0]}>{file.name}</button>{/each}</div>{/if}
  {/if}
  {#if converter.loading && converter.availability.status === "ready"}<p class="hint" role="status">Checking compatible formats…</p>{/if}
  {#if converter.error}<p class="conversion-error" role="alert">{converter.error}</p>{/if}
</section>

<style>
  .converter-panel { display: flex; flex-direction: column; gap: 6px; min-height: 100%; padding: 4px; }
  .converter-heading, .converter-title, .heading-actions, .format-row, .destination, .result-footer { display: flex; align-items: center; gap: 8px; }
  .converter-heading { min-height: 28px; justify-content: space-between; }
  .converter-title { color: var(--tool-teal); }
  h2 { font-size: 14px; color: var(--text-primary); font-weight: 600; }
  h3 { font-size: 13px; font-weight: 600; }
  .local-label { color: var(--tool-teal); font-size: 11px; }
  button { min-height: 32px; padding: 4px 8px; border-radius: 8px; font-size: 12px; color: var(--accent-primary); }
  button:hover:not(:disabled) { background: var(--surface-hover); }
  button.filled { background: var(--accent-primary); color: var(--on-accent); padding-inline: 14px; font-weight: 600; }
  .selection-list { flex: 1; min-height: 48px; }
  .drop-empty, .setup { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; text-align: center; padding: 12px 8px; }
  .empty-icon, .setup-icon { display: grid; place-items: center; color: var(--tool-teal); background: var(--tool-teal-muted); border-radius: 12px; width: 44px; height: 44px; }
  p, small { font-size: 12px; color: var(--text-secondary); max-width: 46ch; }
  small { font-size: 11px; }
  .setup-actions { display: flex; flex-wrap: wrap; justify-content: center; gap: 4px; }
  .file-row { display: flex; align-items: center; gap: 8px; min-height: 48px; border-bottom: 1px solid var(--border-subtle); opacity: 1; transform: translateY(0); transition: opacity var(--motion-context) var(--ease-natural), transform var(--motion-context) var(--ease-natural); }
  @starting-style { .file-row { opacity: 0; transform: translateY(4px); } }
  .file-kind { display: grid; place-items: center; width: 34px; height: 32px; flex: none; border-radius: 8px; background: var(--tool-teal-muted); color: var(--tool-teal); font-size: 9px; font-weight: 600; }
  .file-info { min-width: 0; flex: 1; display: flex; flex-direction: column; gap: 2px; }
  .file-name { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-size: 12px; }
  .file-meta { color: var(--text-secondary); font-size: 11px; overflow-wrap: anywhere; }
  .failed .file-meta, .conversion-error { color: var(--status-error); }
  .format-row { padding-top: 4px; border-top: 1px solid var(--border-subtle); font-size: 12px; }
  .format-row select { flex: 1; min-width: 0; min-height: 32px; border: 1px solid var(--border-strong); border-radius: 8px; padding: 4px 8px; font: inherit; color: var(--text-primary); background: var(--surface-raised); color-scheme: dark; }
  .destination { color: var(--text-secondary); font-size: 11px; }
  .destination > span { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .hint { font-size: 11px; }
  .hint summary { min-height: 20px; color: var(--text-secondary); cursor: pointer; }
  .hint summary:focus-visible { outline: 2px solid var(--accent-focus); outline-offset: 2px; }
  .hint button { padding: 0; min-height: 24px; text-decoration: underline; }
  .result-footer { justify-content: space-between; font-size: 12px; color: var(--tool-teal); }
  progress { width: 100%; height: 3px; border: 0; background: var(--surface-active); accent-color: var(--tool-teal); }
  progress::-webkit-progress-bar { background: var(--surface-active); } progress::-webkit-progress-value { background: var(--tool-teal); }
  .conversion-error { padding: 6px 0; font-size: 12px; overflow-wrap: anywhere; }
  .recent { display: flex; flex-direction: column; align-items: stretch; gap: 2px; font-size: 11px; color: var(--text-secondary); border-top: 1px solid var(--border-subtle); padding-top: 8px; }
  .recent button { text-align: left; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
</style>
