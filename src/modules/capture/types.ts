export type CaptureMode = "area" | "window" | "screen";
export interface CaptureMonitor { id: string; name: string; width: number; height: number; primary: boolean }
export interface CaptureWindow { id: string; name: string }
export interface CaptureTargets { supported: boolean; monitors: CaptureMonitor[]; windows: CaptureWindow[] }
export interface CaptureResult { status: "copied" | "cancelled"; width: number; height: number }
