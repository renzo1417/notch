/**
 * Finite state model for the desktop notch shell.
 * Defined in PRODUCT_SPEC.md section 5.
 */
export type NotchShellState =
  | "dormant"
  | "compact"
  | "peek"
  | "expanded"
  | "apps"
  | "focus"
  | "clipboard"
  | "downloads"
  | "utilities"
  | "battery"
  | "panel"
  | "launcher"
  | "settings";

export interface ShellDimensions {
  width: number;
  height: number;
  windowWidth: number;
  windowHeight: number;
}

const dimensions = (width: number, height: number): ShellDimensions => ({
  width, height, windowWidth: width + 32, windowHeight: height + 16,
});

export const SHELL_STATE_DIMENSIONS: Record<NotchShellState, ShellDimensions> = {
  dormant: dimensions(160, 32),
  compact: dimensions(304, 44),
  peek: dimensions(304, 44),
  expanded: dimensions(480, 160),
  apps: dimensions(480, 328),
  focus: dimensions(480, 328),
  clipboard: dimensions(480, 328),
  downloads: dimensions(480, 328),
  utilities: dimensions(480, 328),
  battery: dimensions(480, 328),
  panel: dimensions(480, 328),
  launcher: dimensions(480, 328),
  settings: dimensions(640, 456),
};

export function normalizeShellState(state: NotchShellState): NotchShellState {
  if (state === "launcher" || state === "panel") return "apps";
  if (state === "peek" || state === "dormant") return "compact";
  return state;
}

export function backState(state: NotchShellState): NotchShellState {
  return state === "expanded" || state === "compact" ? "compact" : "expanded";
}

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
