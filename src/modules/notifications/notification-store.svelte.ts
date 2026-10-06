import { contextEngine, ContextPriority } from "../../app/state/context-engine";
import type { InternalNotification } from "./types";

class NotificationStore {
  activeNotification = $state<InternalNotification | null>(null);

  /**
   * Pushes a local internal notification into the context priority engine (P1_COMPLETION).
   * Upon expiration, the context engine seamlessly reveals the previous meaningful context.
   */
  notify(params: {
    id?: string;
    title: string;
    message: string;
    durationMs?: number;
  }): void {
    const id = params.id ?? `notif_${Date.now()}`;
    const durationMs = params.durationMs ?? 4500;

    const notification: InternalNotification = {
      id,
      title: params.title,
      message: params.message,
      createdAtMs: Date.now(),
      durationMs,
    };

    this.activeNotification = notification;

    // Registers at P1_COMPLETION (higher precedence than P2 Focus Timer and P4 Coding)
    contextEngine.pushTemporaryContext({
      id,
      mode: "notification",
      priority: ContextPriority.P1_COMPLETION,
      durationMs,
      data: notification,
    });

    setTimeout(() => {
      if (this.activeNotification?.id === id) {
        this.activeNotification = null;
      }
    }, durationMs);
  }

  /**
   * Dismisses the active notification immediately.
   */
  dismiss(): void {
    if (this.activeNotification) {
      contextEngine.dismissTemporaryContext(this.activeNotification.id);
      this.activeNotification = null;
    }
  }
}

export const notificationStore = new NotificationStore();
