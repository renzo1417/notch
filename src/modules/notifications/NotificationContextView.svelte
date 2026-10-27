<script lang="ts">
  import { notificationStore } from "./notification-store.svelte";
  import IconCheck from "../../icons/IconCheck.svelte";

  const notif = $derived(notificationStore.activeNotification);
  const title = $derived(notif?.title || "Complete");
  const message = $derived(notif?.message || "Notification completed");
</script>

<div class="notif-context" aria-label="Notification: {title} · {message}">
  <div class="context-header">
    <span class="context-category">NOTIFICATION · COMPLETED</span>
    <div class="check-dot" aria-hidden="true">
      <IconCheck size={9} color="var(--status-success)" />
    </div>
  </div>
  <div class="context-body">
    <span class="context-title" title={title}>{title}</span>
    <span class="context-meta" title={message}>{message}</span>
  </div>
</div>

<style>
  .notif-context {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 2px;
    width: 100%;
    min-width: 0;
    user-select: none;
    box-sizing: border-box;
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

  .check-dot {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 13px;
    height: 13px;
    border-radius: 50%;
    background-color: var(--surface-hover);
    flex-shrink: 0;
  }

  .context-body {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }

  .context-title {
    font-size: var(--font-size-title);
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
