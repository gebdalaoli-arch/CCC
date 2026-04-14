export interface LauncherConfig {
  baseUrl: string;
  apiKey: string;
  profileName: string;
  compatibilityMode: boolean;
  launchMode: "desktop_preferred" | "desktop_only" | "cli_only";
}

export type CheckStatus = "idle" | "running" | "ok" | "error";

export interface EnvironmentCheckResult {
  status: CheckStatus;
  summary: string;
  details: string[];
  platform: "windows" | "mac_os" | "other";
  codexInstalled: boolean;
  codexVersion?: string | null;
  desktopAppInstalled: boolean;
  desktopAppPath?: string | null;
  wslCheckCommand?: string[] | null;
  wslAvailable: boolean;
  macosTerminalCommandExample?: string[] | null;
}

export interface InstallRepairResult {
  success: boolean;
  codexInstalled: boolean;
  codexVersion?: string | null;
  desktopAppInstalled: boolean;
  desktopAppPath?: string | null;
  installPageUrl?: string | null;
  openedDownloadPage: boolean;
  message: string;
}

export interface LaunchResult {
  success: boolean;
  started: boolean;
  resolvedLaunchMode: string;
  launchTarget: string;
  command: string[];
  env: Record<string, string>;
  configPath: string;
  authPath: string;
  message: string;
}

export interface OpenLogsResult {
  success: boolean;
  logsDir: string;
  openCommand: string[];
  status: CheckStatus;
  message: string;
}
