/**
 * Internal notification model.
 * Defined in PRODUCT_SPEC.md section 19.
 */
export interface InternalNotification {
  id: string;
  title: string;
  message: string;
  createdAtMs: number;
  durationMs?: number;
}
