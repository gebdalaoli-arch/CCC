import { useState } from "react";
import { CompatibilityNote } from "./components/CompatibilityNote";
import { EnvironmentPanel } from "./components/EnvironmentPanel";
import { useCodexLauncher } from "./hooks/useCodexLauncher";
import type { LauncherConfig } from "./types";

const INITIAL_CONFIG: LauncherConfig = {
  baseUrl: "",
  apiKey: "",
  profileName: "codex-isolated-profile",
  compatibilityMode: true
};

function App() {
  const [config, setConfig] = useState<LauncherConfig>(INITIAL_CONFIG);
  const {
    isBusy,
    environment,
    lastMessage,
    checkEnvironment,
    installOrRepairCodex,
    saveAndLaunch,
    openLogs
  } = useCodexLauncher();

  const canLaunch =
    config.baseUrl.trim().length > 0 && config.apiKey.trim().length > 0;

  return (
    <main className="app-shell">
      <header>
        <h1>Codex Launcher</h1>
        <p>第三方 API 兼容启动器，默认使用隔离 profile 与 auth scrub。</p>
      </header>

      <section className="panel">
        <h2>启动参数</h2>
        <label htmlFor="base-url">Base URL</label>
        <input
          id="base-url"
          value={config.baseUrl}
          placeholder="https://api.openai.com/v1"
          onChange={(event) =>
            setConfig((prev) => ({ ...prev, baseUrl: event.target.value }))
          }
        />

        <label htmlFor="api-key">API Key</label>
        <input
          id="api-key"
          type="password"
          value={config.apiKey}
          placeholder="sk-..."
          onChange={(event) =>
            setConfig((prev) => ({ ...prev, apiKey: event.target.value }))
          }
        />

        <label htmlFor="profile-name">Profile Name</label>
        <input
          id="profile-name"
          value={config.profileName}
          onChange={(event) =>
            setConfig((prev) => ({ ...prev, profileName: event.target.value }))
          }
        />
      </section>

      <CompatibilityNote />

      <EnvironmentPanel environment={environment} />

      <section className="panel actions">
        <h2>操作</h2>
        <div className="button-row">
          <button type="button" onClick={checkEnvironment} disabled={isBusy}>
            检测环境
          </button>
          <button
            type="button"
            onClick={installOrRepairCodex}
            disabled={isBusy}
          >
            安装或修复 Codex
          </button>
          <button
            type="button"
            onClick={() => saveAndLaunch(config)}
            disabled={!canLaunch || isBusy}
          >
            保存并启动
          </button>
          <button type="button" onClick={openLogs} disabled={isBusy}>
            打开日志
          </button>
        </div>
        <p className="status-line">{lastMessage}</p>
      </section>
    </main>
  );
}

export default App;
