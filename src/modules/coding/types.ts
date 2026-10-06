/**
 * Coding context model.
 * Defined in PRODUCT_SPEC.md section 14.
 */
export interface CodingContextState {
  appName: string;
  projectName?: string;
  elapsedMinutes: number;
  isActive: boolean;
}
