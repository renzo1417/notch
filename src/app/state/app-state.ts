import type { NotchShellState } from "./shell-state";
import type { ContextMode } from "./context-engine";

export interface AppState {
  shellState: NotchShellState;
  contextMode: ContextMode;
  isBackendConnected: boolean;
}

export const initialAppState: AppState = {
  shellState: "compact",
  contextMode: "idle",
  isBackendConnected: false,
};
