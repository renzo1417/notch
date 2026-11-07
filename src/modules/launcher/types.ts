/**
 * Launchable application model.
 * Defined in PRODUCT_SPEC.md section 20 & 21.
 */
export const MAX_PINNED_BEFORE_OVERFLOW = 5;

export type LaunchTarget =
  | { kind: "executable"; path: string; args: string[]; workingDirectory?: string | null }
  | { kind: "shortcut"; path: string }
  | { kind: "packaged"; aumid: string };

export interface InstalledApp {
  id: string;
  name: string;
  source: string;
  target: LaunchTarget;
}

export interface InstalledAppCatalog {
  apps: InstalledApp[];
  warnings: string[];
}

export function launchIdentity(app: { target?: LaunchTarget | null; executablePath?: string; args?: string[] }): string {
  const target = app.target ?? (/\.lnk$/i.test(app.executablePath ?? "")
    ? { kind: "shortcut" as const, path: app.executablePath! }
    : { kind: "executable" as const, path: app.executablePath ?? "", args: app.args ?? [] });
  if (target.kind === "packaged") return `app:${target.aumid.toLowerCase()}`;
  const path = target.path.replaceAll("/", "\\").toLowerCase();
  if (target.kind === "shortcut") return `lnk:${path}`;
  return `exe:${path}|${JSON.stringify(target.args)}|${target.workingDirectory ?? ""}`;
}

export function searchInstalledApps(apps: InstalledApp[], query: string): InstalledApp[] {
  const words = query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
  return apps.filter((app) => words.every((word) => `${app.name} ${app.source}`.toLocaleLowerCase().includes(word)));
}

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
  target?: LaunchTarget | null;
}

export interface InspectedApp {
  name: string;
  executablePath: string;
  args: string[];
  iconDataUrl?: string;
  isShortcut: boolean;
  target?: LaunchTarget;
}
