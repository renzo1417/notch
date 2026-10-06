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
}

export const SHELL_STATE_DIMENSIONS: Record<NotchShellState, ShellDimensions> = {
  dormant: { width: 140, height: 28 },
  compact: { width: 280, height: 36 },
  peek: { width: 340, height: 44 },
  expanded: { width: 520, height: 72 },
  panel: { width: 560, height: 160 },
  settings: { width: 560, height: 280 },
};
