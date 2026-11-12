export interface ConversionFile {
  path: string; name: string; extension: string; bytes: number;
  state: "pending" | "running" | "done" | "failed" | "cancelled";
  progress: number | null; outputs: string[]; error: string | null;
}
export interface ConversionSnapshot {
  jobId: string; running: boolean; files: ConversionFile[]; target: string; message: string;
}
export interface ConverterAvailability {
  status: "missing" | "ready" | "incompatible" | ""; version: string | null; message: string;
}
export interface ConversionInspection { files: ConversionFile[]; targets: string[]; unavailableTargets: string[] }

export const FORMAT_NAMES: Record<string, string> = { jpg: "JPG", png: "PNG", webp: "WebP", pdf: "PDF", docx: "Word", xlsx: "Excel", csv: "CSV", txt: "Text", html: "HTML" };
export const INPUT_EXTENSIONS = new Set(["jpg", "jpeg", "png", "webp", "heic", "heif", "tif", "tiff", "bmp", "gif", "avif", "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "md", "csv"]);

export function supportedDrop(paths: string[]): boolean {
  return paths.length > 0 && paths.length <= 20 && paths.every(path => INPUT_EXTENSIONS.has(path.split(".").pop()?.toLowerCase() ?? ""));
}

export function insideDropTarget(position: { x: number; y: number }, scale: number, rect: { left: number; right: number; top: number; bottom: number }): boolean {
  if (!Number.isFinite(scale) || scale <= 0) return false;
  const x = position.x / scale, y = position.y / scale;
  return x >= rect.left && x < rect.right && y >= rect.top && y < rect.bottom;
}

export function formatBytes(bytes: number): string {
  return bytes < 1024 * 1024 ? `${Math.max(1, Math.round(bytes / 1024))} KB` : `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function fileStatus(file: ConversionFile): string {
  if (file.state === "running") return file.progress === null ? "Converting…" : `${Math.round(file.progress * 100)}%`;
  return { pending: "Queued", done: "Converted", failed: "Failed", cancelled: "Cancelled" }[file.state];
}
