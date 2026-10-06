/**
 * Downloads state model.
 * Defined in PRODUCT_SPEC.md section 16.
 */
export interface DownloadState {
  fileName: string;
  progressPercent?: number;
  isCompleted: boolean;
  filePath?: string;
}
