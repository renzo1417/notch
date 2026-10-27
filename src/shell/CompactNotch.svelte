<script lang="ts">
  import MascotCanvas from "../mascot/MascotCanvas.svelte";
  import ActivitySummary from "./ActivitySummary.svelte";
  import { mediaStore } from "../modules/music/media-store.svelte";
  import { settingsStore } from "../settings/settings-store.svelte";
  import IconMusic from "../icons/IconMusic.svelte";
  let { onopen }: { onopen: () => void } = $props();
  let mascot = $state<MascotCanvas>();
  let artFailed = $state(false);
  const media = $derived(mediaStore.state);
  $effect(() => { media.artworkDataUrl; artFailed = false; });
</script>

<button type="button" class="compact-layout" onclick={onopen}
  onmouseenter={() => mascot?.triggerState("curious", 700)}
  aria-label="Open notch" aria-describedby="compact-activity" aria-expanded="false" data-focus="compact">
  {#if settingsStore.state.mascot.enabled}
    <span class="compact-mascot" aria-hidden="true"><MascotCanvas bind:this={mascot} size={32} compact interactive={false} /></span>
  {/if}
  <span class="compact-summary" id="compact-activity"><ActivitySummary compact /></span>
  {#if settingsStore.state.music.enabled && media.hasSession}
    <span class="media-indicator" class:playing={media.isPlaying} title={media.title || "Media session"}>
      {#if media.artworkDataUrl && !artFailed}
        <img src={media.artworkDataUrl} alt="" onerror={() => { artFailed = true; }} />
      {:else}<IconMusic size={16} />{/if}
    </span>
  {/if}
</button>

<style>
  .compact-layout { display: flex; align-items: center; gap: 12px; width: 100%; height: 100%; padding: 0 12px; text-align: left; border-radius: inherit; }
  .compact-mascot { display: flex; flex: none; }
  .compact-summary { min-width: 0; flex: 1; }
  .media-indicator { display: flex; align-items: center; justify-content: center; width: 24px; height: 24px; color: var(--text-secondary); opacity: .65; flex: none; }
  .media-indicator.playing { opacity: 1; }
  .media-indicator img { width: 24px; height: 24px; object-fit: cover; border-radius: 6px; }
</style>
