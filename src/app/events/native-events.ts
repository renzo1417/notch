import type { MediaState } from "../../modules/music/types";
import type { BatteryState } from "../../modules/battery/types";
import type { ClipboardItem } from "../../modules/clipboard/types";
import type { DownloadState } from "../../modules/downloads/types";

export interface ForegroundAppPayload {
  processName: string;
  windowTitle: string;
  isEditor: boolean;
}

export type NativeEvent =
  | { type: "media.changed"; payload: MediaState }
  | { type: "battery.changed"; payload: BatteryState }
  | { type: "clipboard.changed"; payload: ClipboardItem }
  | { type: "foreground.changed"; payload: ForegroundAppPayload }
  | { type: "download.changed"; payload: DownloadState }
  | { type: "download.completed"; payload: DownloadState };
