import type { CaptureMode, CaptureMonitor } from "./types.ts";

export function defaultCaptureMonitor(monitors: CaptureMonitor[], configuredName: string | null): string {
  return (monitors.find((monitor) => monitor.name === configuredName) ?? monitors.find((monitor) => monitor.primary) ?? monitors[0])?.id ?? "";
}

export function captureAction(mode: CaptureMode): string {
  return mode === "area" ? "Select area" : mode === "window" ? "Capture window" : "Capture screen";
}
