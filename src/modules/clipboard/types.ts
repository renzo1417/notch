/**
 * Clipboard item and history model.
 * Defined in PRODUCT_SPEC.md section 17.
 */
export interface ClipboardItem {
  id: string;
  text: string;
  timestampMs: number;
}

export interface ClipboardState {
  items: ClipboardItem[];
  isPaused: boolean;
  maxItems: number;
}
