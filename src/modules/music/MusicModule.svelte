<script lang="ts">
  import { mediaStore } from "./media-store.svelte";
  import { settingsStore } from "../../settings/settings-store.svelte";
  import IconMusic from "../../icons/IconMusic.svelte";
  import IconPlay from "../../icons/IconPlay.svelte";
  import IconPause from "../../icons/IconPause.svelte";
  import IconSkipBack from "../../icons/IconSkipBack.svelte";
  import IconSkipForward from "../../icons/IconSkipForward.svelte";
  let { mini = false }: { mini?: boolean } = $props();
  let artFailed = $state(false);
  let error = $state("");
  const media = $derived(mediaStore.state);
  const canToggle = $derived(media.capabilities.canTogglePlayPause || (media.isPlaying ? media.capabilities.canPause : media.capabilities.canPlay));
  $effect(() => { media.artworkDataUrl; artFailed = false; });
  async function skip(direction: "previous" | "next") {
    error = "";
    if (!(await mediaStore[direction]())) error = "Couldn’t skip track. Try again.";
  }
  async function toggle() {
    error = "";
    if (!(await mediaStore.togglePlayPause())) error = "Playback failed. Try again.";
  }
</script>

{#if settingsStore.state.music.enabled && media.hasSession}
  <div class="music-zone" class:mini aria-label="Music player">
    <div class="art-container" aria-hidden="true">
      {#if media.artworkDataUrl && !artFailed}<img src={media.artworkDataUrl} alt="" onerror={() => { artFailed = true; }} />
      {:else}<IconMusic size={16} />{/if}
    </div>
    {#if !mini}
      <div class="metadata">
        <span class="track-title" title={media.title || "Media session"}>{media.title || "Media session"}</span>
        <span role={error ? "status" : undefined} class="track-artist" title={media.artist || media.album || ""}>{error || media.artist || media.album || (media.isPlaying ? "Playing" : "Paused")}</span>
      </div>
    {/if}
    {#if settingsStore.state.music.showControls}
      <div class="controls-row">
        {#if !mini}<button type="button" disabled={!media.capabilities.canSkipPrevious} onclick={() => skip("previous")} aria-label="Previous track" title="Previous track"><IconSkipBack size={16} /></button>{/if}
        <button type="button" disabled={!canToggle} onclick={toggle} aria-label={media.isPlaying ? "Pause" : "Play"} title={error || (media.isPlaying ? "Pause" : "Play")}>
          {#if media.isPlaying}<IconPause size={16} />{:else}<IconPlay size={16} />{/if}
        </button>
        {#if !mini}<button type="button" disabled={!media.capabilities.canSkipNext} onclick={() => skip("next")} aria-label="Next track" title="Next track"><IconSkipForward size={16} /></button>{/if}
      </div>
    {/if}
    {#if mini && error}<span class="sr-only" role="status">{error}</span>{/if}
  </div>
{/if}

<style>
  .music-zone { display: flex; align-items: center; gap: 12px; min-width: 0; width: 100%; }
  .art-container { display: flex; align-items: center; justify-content: center; width: 36px; height: 36px; border-radius: 8px; background: var(--surface-raised); color: var(--text-secondary); flex: none; overflow: hidden; }
  .art-container img { width: 100%; height: 100%; object-fit: cover; }
  .metadata { flex: 1; min-width: 0; display: grid; gap: 3px; }
  .track-title, .track-artist { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .track-title { font-size: 13px; font-weight: 500; }
  .track-artist { font-size: 12px; color: var(--text-secondary); }
  .controls-row { display: flex; gap: 4px; flex: none; }
  .controls-row button { display: flex; align-items: center; justify-content: center; width: 32px; height: 32px; border-radius: 8px; color: var(--text-primary); }
  .controls-row button:hover:not(:disabled) { background: var(--surface-hover); }
  .mini { width: auto; gap: 6px; flex: none; }
  .mini .art-container { width: 24px; height: 24px; border-radius: 6px; }
</style>
