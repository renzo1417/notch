<script lang="ts">
  import IconMusic from "../../icons/IconMusic.svelte";
  import IconPlay from "../../icons/IconPlay.svelte";
  import IconSkipBack from "../../icons/IconSkipBack.svelte";
  import IconSkipForward from "../../icons/IconSkipForward.svelte";
</script>

<div class="music-zone" aria-label="Media player: No media currently playing">
  <!-- Stable artwork container -->
  <div class="art-container" aria-hidden="true">
    <IconMusic size={16} color="var(--text-tertiary)" />
  </div>

  <!-- Track metadata and controls column -->
  <div class="meta-container">
    <div class="text-group">
      <span class="track-title" title="No media playing">No media</span>
      <span class="track-artist" title="Windows media session idle">System audio idle</span>
    </div>

    <!-- Subdued transport controls -->
    <div class="controls-row" aria-hidden="true">
      <button type="button" class="ctrl-btn disabled" disabled aria-label="Previous track (inactive)">
        <IconSkipBack size={10} color="var(--text-tertiary)" />
      </button>
      <button type="button" class="ctrl-btn disabled" disabled aria-label="Play/Pause (inactive)">
        <IconPlay size={10} color="var(--text-tertiary)" />
      </button>
      <button type="button" class="ctrl-btn disabled" disabled aria-label="Next track (inactive)">
        <IconSkipForward size={10} color="var(--text-tertiary)" />
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
  }

  .meta-container {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 3px;
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
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .track-artist {
    font-size: var(--font-size-tiny);
    line-height: var(--line-height-tight);
    color: var(--text-tertiary);
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
    width: 18px;
    height: 18px;
    border-radius: 4px;
    background: transparent;
    opacity: 0.45;
    cursor: default;
    padding: 0;
  }

  .ctrl-btn.disabled {
    pointer-events: none;
  }
</style>
