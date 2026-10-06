/**
 * Launchable application model.
 * Defined in PRODUCT_SPEC.md section 20 & 21.
 */
export const MAX_PINNED_BEFORE_OVERFLOW = 5;

export interface LaunchableApp {
  id: string;
  name: string;
  executablePath: string;
  args: string[];
  iconPath?: string;
  iconDataUrl?: string;
  enabled: boolean;
  order: number;
  exists?: boolean;
}

export interface InspectedApp {
  name: string;
  executablePath: string;
  args: string[];
  iconDataUrl?: string;
  isShortcut: boolean;
}
