export function CompatibilityNote() {
  return (
    <aside className="compatibility-note" data-testid="compatibility-note">
      <h2>兼容模式说明</h2>
      <p>
        启动前会清理 subscription tokens 与历史订阅登录残留，改用隔离 profile，
        并归一化 OpenAI/openai provider 标签后启动 Codex CLI。
      </p>
    </aside>
  );
}
