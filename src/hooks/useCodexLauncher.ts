import { useState } from "react";
import type {
  EnvironmentCheckResult,
  InstallRepairResult,
  LauncherConfig,
  LaunchResult,
  OpenLogsResult
} from "../types";
import { invokeOrFallback } from "../lib/tauri";

const FALLBACK_ENV_RESULT: EnvironmentCheckResult = {
  platform: "other",
  codexInstalled: false,
  codexVersion: null,
  desktopAppInstalled: false,
  desktopAppPath: null,
  offlineCliBundled: true,
  offlineCliPath: "browser-preview/offline-runtime",
  wslCheckCommand: null,
  wslAvailable: false,
  macosTerminalCommandExample: null,
  status: "ok",
  summary: "当前为浏览器预览模式，环境检测占位结果已就绪。",
  details: [
    "Tauri invoke 未接入，使用前端占位逻辑。",
    "默认会优先尝试官方 Codex Desktop，不存在时才回退到 CLI。"
  ]
};

const EMPTY_ENV_RESULT: EnvironmentCheckResult = {
  platform: "other",
  codexInstalled: false,
  codexVersion: null,
  desktopAppInstalled: false,
  desktopAppPath: null,
  offlineCliBundled: false,
  offlineCliPath: null,
  wslCheckCommand: null,
  wslAvailable: false,
  macosTerminalCommandExample: null,
  status: "idle",
  summary: "尚未检测环境。",
  details: []
};

export function useCodexLauncher() {
  const [isBusy, setIsBusy] = useState(false);
  const [environment, setEnvironment] =
    useState<EnvironmentCheckResult>(EMPTY_ENV_RESULT);
  const [lastMessage, setLastMessage] = useState("等待操作。");

  const checkEnvironment = async () => {
    setIsBusy(true);
      setEnvironment({
        ...EMPTY_ENV_RESULT,
        status: "running",
        summary: "正在检测环境..."
      });
    try {
      const result = await invokeOrFallback<EnvironmentCheckResult>(
        "detect_environment",
        undefined,
        FALLBACK_ENV_RESULT
      );
      setEnvironment(result);
      setLastMessage("环境检测已完成。");
    } catch (error) {
      setEnvironment({
        ...EMPTY_ENV_RESULT,
        status: "error",
        summary: "环境检测失败。",
        details: [String(error)]
      });
      setLastMessage("环境检测失败，请查看日志。");
    } finally {
      setIsBusy(false);
    }
  };

  const installOrRepairCodex = async () => {
    setIsBusy(true);
    try {
      const result = await invokeOrFallback<InstallRepairResult>(
        "install_or_repair_codex",
        undefined,
        {
          success: true,
          codexInstalled: false,
          codexVersion: null,
          desktopAppInstalled: false,
          desktopAppPath: null,
          offlineCliBundled: true,
          offlineCliPath: "browser-preview/offline-runtime",
          installPageUrl: "https://openai.com/codex/get-started/",
          openedDownloadPage: false,
          message: "浏览器模式：内置离线 CLI 可用，不依赖商店。"
        }
      );
      setLastMessage(result.message);
    } catch (error) {
      setLastMessage(`安装或修复失败：${String(error)}`);
    } finally {
      setIsBusy(false);
    }
  };

  const saveAndLaunch = async (config: LauncherConfig) => {
    setIsBusy(true);
    try {
      const result = await invokeOrFallback<LaunchResult>(
        "save_and_launch",
        {
          request: {
            profileName: config.profileName,
            openaiApiKey: config.apiKey,
            openaiBaseUrl: config.baseUrl,
            launchMode: config.launchMode,
            launchNow: true,
            extraArgs: []
          }
        },
        {
          success: true,
          started: false,
          resolvedLaunchMode: config.launchMode,
          launchTarget: config.launchMode === "cli_only" ? "cli" : "desktop",
          command: ["codex"],
          env: {},
          configPath: "browser-preview/config.toml",
          authPath: "browser-preview/auth.json",
          message: "浏览器模式：已保存配置并模拟启动 Codex CLI。"
        }
      );
      setLastMessage(result.message);
    } catch (error) {
      setLastMessage(`保存并启动失败：${String(error)}`);
    } finally {
      setIsBusy(false);
    }
  };

  const openLogs = async () => {
    try {
      const result = await invokeOrFallback<OpenLogsResult>(
        "open_logs",
        undefined,
        {
          success: true,
          logsDir: "browser-preview/logs",
          openCommand: [],
          status: "ok",
          message: "浏览器模式：日志入口占位。"
        }
      );
      setLastMessage(result.message);
    } catch (error) {
      setLastMessage(`打开日志失败：${String(error)}`);
    }
  };

  return {
    isBusy,
    environment,
    lastMessage,
    checkEnvironment,
    installOrRepairCodex,
    saveAndLaunch,
    openLogs
  };
}
