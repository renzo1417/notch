<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import MascotCanvas from "../mascot/MascotCanvas.svelte";
  import IconMusic from "../icons/IconMusic.svelte";
  import { mediaStore } from "../modules/music/media-store.svelte";
  import { codingStore } from "../modules/coding/coding-store.svelte";
  import { contextEngine, type ContextCandidate } from "../app/state/context-engine";

  interface Props {
    backendConnected: boolean;
  }

  let { backendConnected }: Props = $props();

  let activeCandidate = $state<ContextCandidate>(contextEngine.current);
  let unsubContext: (() => void) | null = null;

  onMount(() => {
    mediaStore.init();
    codingStore.init();
    unsubContext = contextEngine.subscribe((cand) => {
      activeCandidate = cand;
    });
  });

  onDestroy(() => {
    if (unsubContext) {
      unsubContext();
    }
  });

  const media = $derived(mediaStore.state);
  const isPlaying = $derived(media.isPlaying);
  const mediaTooltip = $derived(
    isPlaying && media.title
      ? `${media.title}${media.artist ? ` · ${media.artist}` : ""}`
      : media.hasSession && media.title
      ? `Paused: ${media.title}`
      : "No audio playing"
  );

  const codingSession = $derived(codingStore.state);
  const compactCodingLabel = $derived(
    codingSession.appName
      ? `${codingSession.appName}${codingSession.projectLabel ? ` · ${codingSession.projectLabel}` : ""}${codingSession.elapsedMinutes > 0 ? ` · ${codingSession.elapsedMinutes}m` : ""}`
      : "Coding"
  );

  const compactLabel = $derived(
    activeCandidate.mode === "coding"
      ? compactCodingLabel
      : activeCandidate.mode === "notification"
      ? (typeof activeCandidate.data === "string" ? activeCandidate.data : "Notification")
      : "Windows Notch"
  );
</script>

<div class="compact-layout" aria-label="Compact Notch Shell: {compactLabel}">
  <!-- Left: Mini companion avatar -->
  <div class="compact-mascot" aria-hidden="true">
    <MascotCanvas size={26} compact={true} interactive={false} />
  </div>

  <!-- Center: Immediate context summary -->
  <div class="compact-center">
    <div
      class="status-dot"
      class:connected={backendConnected}
      class:coding={activeCandidate.mode === "coding"}
      aria-hidden="true"
      title={activeCandidate.mode === "coding" ? `Coding: ${compactCodingLabel}` : backendConnected ? "Native backend connected" : "Connecting to backend..."}
    ></div>
    <span class="center-label" title={compactLabel}>{compactLabel}</span>
  </div>

  <!-- Right: Subdued media indicator -->
  <div
    class="compact-media"
    class:playing={isPlaying}
    aria-hidden="true"
    title={mediaTooltip}
  >
    {#if isPlaying && media.artworkDataUrl}
      <img src={media.artworkDataUrl} alt="" class="compact-art" />
    {:else}
      <IconMusic
        size={13}
        color={isPlaying ? "var(--text-primary)" : "var(--text-tertiary)"}
      />
    {/if}
  </div>
</div>


<style>
  .compact-layout {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    height: 100%;
    user-select: none;
    padding: 0 var(--space-12);
    box-sizing: border-box;
  }

  .compact-mascot {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    flex-shrink: 0;
  }

  .compact-center {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-6);
    flex: 1;
    min-width: 0;
    padding: 0 var(--space-8);
  }

  .status-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background-color: var(--text-tertiary);
    transition: background-color var(--motion-micro) var(--ease-natural);
    flex-shrink: 0;
  }

  .status-dot.connected {
    background-color: #22c55e;
  }

  .status-dot.coding {
    background-color: #38bdf8;
  }

  .center-label {
    font-size: var(--font-size-label);
    font-weight: 600;
    letter-spacing: 0.2px;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .compact-media {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    flex-shrink: 0;
    opacity: 0.6;
    border-radius: 4px;
    overflow: hidden;
    transition: opacity var(--motion-micro) var(--ease-natural);
  }

  .compact-media.playing {
    opacity: 1;
  }

  .compact-art {
    width: 16px;
    height: 16px;
    border-radius: 3px;
    object-fit: cover;
  }
</style>

