<script lang="ts">
  import { codingStore } from "./coding-store.svelte";
  import { timerStore } from "../timer/timer-store.svelte";
  import IconTimer from "../../icons/IconTimer.svelte";

  const session = $derived(codingStore.state);

  const displayTitle = $derived(
    session.appName
      ? `${session.appName}${session.projectLabel ? ` · ${session.projectLabel}` : ""}`
      : "Coding Session"
  );

  const displayMeta = $derived(
    session.elapsedMinutes > 0
      ? `${session.elapsedMinutes} min in session`
      : "Just started"
  );

  const elapsedBadge = $derived(
    session.elapsedMinutes > 0 ? `${session.elapsedMinutes}M` : "NEW"
  );
</script>

<div class="coding-context" aria-label="Active coding context: {displayTitle} · {displayMeta}">
  <div class="context-header">
    <div class="header-left">
      <span class="context-category">CODE · {elapsedBadge}</span>
      <div class="coding-dot" aria-hidden="true"></div>
    </div>
    <button
      type="button"
      class="quick-focus-trigger"
      onclick={(e) => {
        e.stopPropagation();
        timerStore.openTimerSurface();
      }}
      title="Open Focus Timer"
      aria-label="Open Focus Timer"
    >
      <IconTimer size={10} color="var(--text-secondary)" />
      <span>Focus</span>
    </button>
  </div>
  <div class="context-body">
    <span class="context-title" title={displayTitle}>{displayTitle}</span>
    <span class="context-meta" title={displayMeta}>{displayMeta}</span>
  </div>
</div>

<style>
  .coding-context {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 2px;
    width: 100%;
    min-width: 0;
    user-select: none;
  }

  .context-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-width: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: var(--space-6);
  }

  .quick-focus-trigger {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: var(--surface-raised);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 1px 6px;
    font-size: var(--font-size-tiny);
    font-weight: 500;
    color: var(--text-secondary);
    cursor: pointer;
    line-height: 1.2;
    transition: all var(--motion-micro) var(--ease-natural);
    flex-shrink: 0;
  }

  .quick-focus-trigger:hover {
    background-color: var(--surface-hover);
    color: var(--text-primary);
    border-color: rgba(45, 212, 191, 0.4);
  }

  .context-category {
    font-size: var(--font-size-tiny);
    font-weight: 600;
    letter-spacing: 0.6px;
    color: var(--text-tertiary);
    text-transform: uppercase;
    line-height: var(--line-height-tight);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .coding-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background-color: #38bdf8;
    flex-shrink: 0;
  }

  .context-body {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }

  .context-title {
    font-size: var(--font-size-body);
    font-weight: 600;
    color: var(--text-primary);
    line-height: var(--line-height-tight);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .context-meta {
    font-size: var(--font-size-meta);
    color: var(--text-secondary);
    line-height: var(--line-height-tight);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
