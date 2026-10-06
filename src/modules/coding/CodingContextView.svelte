<script lang="ts">
  import { codingStore } from "./coding-store.svelte";

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
    <span class="context-category">CODE · {elapsedBadge}</span>
    <div class="coding-dot" aria-hidden="true"></div>
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
    gap: var(--space-6);
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
