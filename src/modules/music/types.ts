/**
 * Media session capabilities model.
 * Matches Windows.Media.Control.GlobalSystemMediaTransportControlsSessionPlaybackControls.
 */
export interface MediaCapabilities {
  canPlay: boolean;
  canPause: boolean;
  canTogglePlayPause: boolean;
  canSkipPrevious: boolean;
  canSkipNext: boolean;
}

/**
 * Media session state model.
 * Defined in PRODUCT_SPEC.md section 13 and Phase 3 requirements.
 */
export interface MediaState {
  hasSession: boolean;
  isPlaying: boolean;
  title?: string;
  artist?: string;
  album?: string;
  artworkDataUrl?: string;
  positionMs?: number;
  durationMs?: number;
  capabilities: MediaCapabilities;
  sourceAppId?: string;
}

export const DEFAULT_MEDIA_CAPABILITIES: MediaCapabilities = {
  canPlay: false,
  canPause: false,
  canTogglePlayPause: false,
  canSkipPrevious: false,
  canSkipNext: false,
};

export const DEFAULT_MEDIA_STATE: MediaState = {
  hasSession: false,
  isPlaying: false,
  title: undefined,
  artist: undefined,
  album: undefined,
  artworkDataUrl: undefined,
  positionMs: undefined,
  durationMs: undefined,
  capabilities: DEFAULT_MEDIA_CAPABILITIES,
  sourceAppId: undefined,
};
