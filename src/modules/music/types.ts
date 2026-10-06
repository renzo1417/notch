/**
 * Media session state model.
 * Defined in PRODUCT_SPEC.md section 13.
 */
export interface MediaState {
  isPlaying: boolean;
  title?: string;
  artist?: string;
  album?: string;
  artworkUrl?: string;
  positionMs?: number;
  durationMs?: number;
}
