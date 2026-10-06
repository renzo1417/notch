/**
 * Launchable application model.
 * Defined in PRODUCT_SPEC.md section 20.
 */
export interface LaunchableApp {
  id: string;
  name: string;
  executablePath: string;
  args: string[];
  iconPath?: string;
  enabled: boolean;
  order: number;
}
