<script lang="ts">
  import { onMount } from "svelte";
  import { mediaStore } from "./media-store.svelte";
  import IconMusic from "../../icons/IconMusic.svelte";
  import IconPlay from "../../icons/IconPlay.svelte";
  import IconPause from "../../icons/IconPause.svelte";
  import IconSkipBack from "../../icons/IconSkipBack.svelte";
  import IconSkipForward from "../../icons/IconSkipForward.svelte";

  onMount(() => {
    mediaStore.init();
  });

  const media = $derived(mediaStore.state);
  const isPlaying = $derived(media.isPlaying);
  const hasActiveTrack = $derived(
    media.hasSession && Boolean(media.title || media.isPlaying)
  );

  const displayTitle = $derived(
    hasActiveTrack && media.title ? media.title : "Nothing playing"
  );

  const displayArtist = $derived(
    hasActiveTrack
      ? (media.artist || media.album || "Unknown artist")
      : "No active media"
  );

  const canPlayPause = $derived(
    media.capabilities.canTogglePlayPause ||
    media.capabilities.canPlay ||
    media.capabilities.canPause
  );
  const canSkipPrev = $derived(media.capabilities.canSkipPrevious);
  const canSkipNext = $derived(media.capabilities.canSkipNext);

  let artLoadFailed = $state(false);

  // Reset art error if new artwork url appears
  $effect(() => {
    if (media.artworkDataUrl) {
      artLoadFailed = false;
    }
  });

  function handlePrev(e: MouseEvent) {
    e.stopPropagation();
    if (canSkipPrev) {
      mediaStore.previous();
    }
  }

  function handleTogglePlay(e: MouseEvent) {
    e.stopPropagation();
    if (canPlayPause) {
      mediaStore.togglePlayPause();
    }
  }

  function handleNext(e: MouseEvent) {
    e.stopPropagation();
    if (canSkipNext) {
      mediaStore.next();
    }
  }
</script>

<div
  class="music-zone"
  aria-label="Media player: {displayTitle} by {displayArtist}"
>
  <!-- Stable 38x38px artwork container -->
  <div class="art-container" aria-hidden="true">
    {#if media.artworkDataUrl && !artLoadFailed}
      <img
        src={media.artworkDataUrl}
        alt="Track artwork"
        class="art-img"
        onerror={() => (artLoadFailed = true)}
      />
    {:else}
      <IconMusic
        size={16}
        color={hasActiveTrack ? "var(--text-secondary)" : "var(--text-tertiary)"}
      />
    {/if}
  </div>

  <!-- Stable metadata & transport controls column -->
  <div class="meta-container">
    <div class="text-group">
      <span
        class="track-title"
        class:dimmed={!hasActiveTrack}
        title={media.title || displayTitle}
      >
        {displayTitle}
      </span>
      <span
        class="track-artist"
        title={media.artist ? `${media.artist}${media.album ? ` — ${media.album}` : ""}` : displayArtist}
      >
        {displayArtist}
      </span>
    </div>

    <!-- Interactive transport controls -->
    <div class="controls-row">
      <button
        type="button"
        class="ctrl-btn"
        class:disabled={!canSkipPrev}
        disabled={!canSkipPrev}
        onclick={handlePrev}
        title={canSkipPrev ? "Previous track" : "Previous track (unavailable)"}
        aria-label="Previous track"
      >
        <IconSkipBack
          size={10}
          color={canSkipPrev ? "var(--text-secondary)" : "var(--text-tertiary)"}
        />
      </button>

      <button
        type="button"
        class="ctrl-btn play-btn"
        class:disabled={!canPlayPause}
        disabled={!canPlayPause}
        onclick={handleTogglePlay}
        title={canPlayPause ? (isPlaying ? "Pause" : "Play") : "Playback controls unavailable"}
        aria-label={isPlaying ? "Pause" : "Play"}
      >
        {#if isPlaying}
          <IconPause
            size={11}
            color={canPlayPause ? "var(--text-primary)" : "var(--text-tertiary)"}
          />
        {:else}
          <IconPlay
            size={11}
            color={canPlayPause ? "var(--text-primary)" : "var(--text-tertiary)"}
          />
        {/if}
      </button>

      <button
        type="button"
        class="ctrl-btn"
        class:disabled={!canSkipNext}
        disabled={!canSkipNext}
        onclick={handleNext}
        title={canSkipNext ? "Next track" : "Next track (unavailable)"}
        aria-label="Next track"
      >
        <IconSkipForward
          size={10}
          color={canSkipNext ? "var(--text-secondary)" : "var(--text-tertiary)"}
        />
      </button>
    </div>
  </div>
</div>

<style>
  .music-zone {
    display: flex;
    align-items: center;
    gap: var(--space-8);
    width: 176px;
    height: 100%;
    padding: 0 var(--space-8);
    user-select: none;
    box-sizing: border-box;
    flex-shrink: 0;
  }

  .art-container {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 38px;
    height: 38px;
    background-color: var(--surface-raised);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-control);
    flex-shrink: 0;
    overflow: hidden;
  }

  .art-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .meta-container {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }

  .text-group {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .track-title {
    font-size: var(--font-size-label);
    font-weight: 600;
    line-height: var(--line-height-tight);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .track-title.dimmed {
    color: var(--text-secondary);
    font-weight: 500;
  }

  .track-artist {
    font-size: var(--font-size-tiny);
    line-height: var(--line-height-tight);
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .controls-row {
    display: flex;
    align-items: center;
    gap: var(--space-6);
    margin-top: 2px;
  }

  .ctrl-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: 4px;
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 0;
    color: inherit;
    transition: background-color var(--motion-micro) var(--ease-natural),
                transform var(--motion-micro) var(--ease-natural),
                opacity var(--motion-micro) var(--ease-natural);
  }

  .ctrl-btn:hover:not(.disabled) {
    background-color: var(--surface-hover);
  }

  .ctrl-btn:active:not(.disabled) {
    transform: scale(0.90);
  }

  .ctrl-btn.disabled {
    opacity: 0.35;
    cursor: default;
    pointer-events: none;
  }

  .play-btn {
    width: 22px;
    height: 22px;
    border-radius: 5px;
  }
</style>
