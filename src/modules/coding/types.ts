/**
 * Coding context session model.
 * Defined in PRODUCT_SPEC.md section 14.
 */

export interface CodingSession {
  appName: string;
  projectLabel?: string;
  elapsedMinutes: number;
  processName: string;
  windowTitle: string;
  isActive: boolean;
  sessionStartMs: number;
}

export const initialCodingSession: CodingSession = {
  appName: "",
  projectLabel: undefined,
  elapsedMinutes: 0,
  processName: "",
  windowTitle: "",
  isActive: false,
  sessionStartMs: 0,
};
