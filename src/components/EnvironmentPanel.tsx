import type { EnvironmentCheckResult } from "../types";

interface EnvironmentPanelProps {
  environment: EnvironmentCheckResult;
}

export function EnvironmentPanel({ environment }: EnvironmentPanelProps) {
  return (
    <section className="panel">
      <h2>环境检测</h2>
      <p>
        状态：<strong>{environment.status}</strong>
      </p>
      <p>
        平台：<strong>{environment.platform}</strong>
      </p>
      <p>
        Codex：<strong>{environment.codexInstalled ? "已检测到" : "未检测到"}</strong>
        {environment.codexVersion ? ` (${environment.codexVersion})` : ""}
      </p>
      {environment.platform === "windows" ? (
        <p>
          WSL：<strong>{environment.wslAvailable ? "可用" : "不可用"}</strong>
        </p>
      ) : null}
      <p>{environment.summary}</p>
      {environment.details.length > 0 ? (
        <ul>
          {environment.details.map((detail) => (
            <li key={detail}>{detail}</li>
          ))}
        </ul>
      ) : (
        <p>尚无详细信息。</p>
      )}
    </section>
  );
}
