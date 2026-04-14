export interface LauncherConfig {
  baseUrl: string;
  apiKey: string;
  profileName: string;
  compatibilityMode: boolean;
}

export type CheckStatus = "idle" | "running" | "ok" | "error";

export interface EnvironmentCheckResult {
  status: CheckStatus;
  summary: string;
  details: string[];
  platform: "windows" | "mac_os" | "other";
  codexInstalled: boolean;
  codexVersion?: string | null;
  wslCheckCommand?: string[] | null;
  wslAvailable: boolean;
  macosTerminalCommandExample?: string[] | null;
}

export interface InstallRepairResult {
  success: boolean;
  codexInstalled: boolean;
  codexVersion?: string | null;
  message: string;
}

export interface LaunchResult {
  success: boolean;
  started: boolean;
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
