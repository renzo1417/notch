/**
 * Downloads state and watched folder models.
 * Defined in PRODUCT_SPEC.md section 16.
 */

export interface DownloadState {
  fileName: string;
  progressPercent?: number;
  isCompleted: boolean;
  filePath?: string;
}

export interface WatchedFolder {
  id: string;
  path: string;
  isDefault: boolean;
  exists: boolean;
}
