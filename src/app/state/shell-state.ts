/**
 * Finite state model for the desktop notch shell.
 * Defined in PRODUCT_SPEC.md section 5.
 */
export type NotchShellState =
  | "dormant"
  | "compact"
  | "peek"
  | "expanded"
  | "panel"
  | "settings";

export interface ShellDimensions {
  width: number;
  height: number;
  windowWidth: number;
  windowHeight: number;
}

export const SHELL_STATE_DIMENSIONS: Record<NotchShellState, ShellDimensions> = {
  dormant: { width: 140, height: 28, windowWidth: 172, windowHeight: 40 },
  compact: { width: 300, height: 38, windowWidth: 332, windowHeight: 50 },
  peek: { width: 340, height: 44, windowWidth: 372, windowHeight: 56 },
  expanded: { width: 520, height: 76, windowWidth: 552, windowHeight: 88 },
  panel: { width: 560, height: 160, windowWidth: 592, windowHeight: 172 },
  settings: { width: 560, height: 280, windowWidth: 592, windowHeight: 292 },
};

type CollapseListener = () => void;

class ShellActionBus {
  private collapseListeners = new Set<CollapseListener>();

  onCollapse(fn: CollapseListener): () => void {
    this.collapseListeners.add(fn);
    return () => {
      this.collapseListeners.delete(fn);
    };
  }

  requestCollapse(): void {
    for (const fn of this.collapseListeners) {
      fn();
    }
  }
}

export const shellActions = new ShellActionBus();
