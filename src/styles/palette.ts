/** Shared by live CSS tokens and the procedural canvas companion. */
export const GRAPHITE = {
  surfaceRgb: "21, 22, 24",
  raised: "#202226",
  text: "#eceef1",
  secondary: "#adb1b8",
  accent: "#d8dce2",
  warning: "#e4b46a",
  error: "#df9191",
  chassis: "#373b41",
  ear: "#454a52",
  visor: "#121416",
} as const;

export function normalizeHex(hex: string): string {
  const value = hex.trim();
  return value.length === 4
    ? `#${[...value.slice(1)].map((digit) => digit + digit).join("")}`
    : value;
}

export function accentForeground(hex: string): string {
  const value = normalizeHex(hex).slice(1);
  const channels = [0, 2, 4].map((offset) => {
    const channel = parseInt(value.slice(offset, offset + 2), 16) / 255;
    return channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
  });
  const luminance = channels[0] * 0.2126 + channels[1] * 0.7152 + channels[2] * 0.0722;
  return luminance > 0.179 ? "#151618" : "#ffffff";
}
