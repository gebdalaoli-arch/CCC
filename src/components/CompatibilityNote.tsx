export function CompatibilityNote() {
  return (
    <aside className="compatibility-note" data-testid="compatibility-note">
      <h2>兼容模式说明</h2>
      <p>
        启动前会清理 subscription tokens 与历史订阅登录残留，改用隔离 profile，
        还会尝试把默认环境中的会话历史桥接进当前隔离环境，并归一化
        OpenAI/openai provider 标签。默认优先启动官方 Codex Desktop，不存在时再回退到 CLI。
      </p>
    </aside>
  );
}
