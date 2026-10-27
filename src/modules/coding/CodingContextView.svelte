<script lang="ts">
  import { codingStore } from "./coding-store.svelte";

  const session = $derived(codingStore.state);

  const primaryTitle = $derived(
    session.projectLabel || session.appName || "Active Session"
  );

  const secondaryMeta = $derived(
    session.projectLabel && session.appName
      ? `${session.appName} · ${session.elapsedMinutes > 0 ? `${session.elapsedMinutes} min` : "just started"}`
      : session.appName
      ? `Coding · ${session.elapsedMinutes > 0 ? `${session.elapsedMinutes} min` : "just started"}`
      : "Active development"
  );
</script>

<div class="coding-context" aria-label="Coding context: {primaryTitle} · {secondaryMeta}">
  <div class="context-body">
    <div class="title-row">
      <span class="context-title" title={primaryTitle}>{primaryTitle}</span>
      <div class="coding-dot" title="Active coding session" aria-hidden="true"></div>
    </div>
    <span class="context-meta" title={secondaryMeta}>{secondaryMeta}</span>
  </div>
</div>

<style>
  .coding-context {
    display: flex;
    flex-direction: column;
    justify-content: center;
    width: 100%;
    min-width: 0;
    user-select: none;
  }

  .context-body {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: var(--space-6);
    min-width: 0;
  }

  .context-title {
    font-size: 14.5px;
    font-weight: 600;
    color: var(--text-primary);
    line-height: var(--line-height-tight);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    letter-spacing: -0.01em;
  }

  .coding-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background-color: var(--status-info);
    flex-shrink: 0;
    opacity: 0.9;
  }

  .context-meta {
    font-size: 12px;
    color: var(--text-secondary);
    line-height: var(--line-height-tight);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
