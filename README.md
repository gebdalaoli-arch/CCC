# Codex Launcher

## 项目目的

这是一个通用型 `Codex Launcher` 原型，目标是把“填写第三方 `OpenAI-compatible` 的 `Base URL + API Key` 并启动 Codex”做成普通用户可用的桌面程序。应用使用 `Tauri 2 + Rust + React`，负责环境检测、配置生成、兼容修复和 CLI 启动，不依赖用户手工改配置文件。

## 当前支持范围

- Windows：优先按兼容模式运行，准备隔离 `CODEX_HOME`、最小 `auth.json`、清理订阅 token 残留，并提供基础安装/修复入口。
- macOS：同样走兼容模式，按本机终端启动 `Codex CLI`。
- 第三方接口：第一版仅支持 `OpenAI-compatible` 且兼容 `Responses API` 的服务。

## 为什么必须做隔离 profile 和 auth scrub

你现有可用启动器已经证明，Codex 在混用第三方 API 和原有登录态时，可能会被历史 `provider` 状态或 `tokens` 残留重新拉回订阅模式。为避免这个问题，当前实现采用以下策略：

- 始终使用隔离的 `CODEX_HOME`
- 每次启动都重写最小 `config.toml`
- 每次启动都重写最小 `auth.json`
- 启动前移除 `tokens` 残留
- 启动前归一化 `OpenAI` / `openai` provider 标签，尽量避免历史对话被拆成两套
- 通过 ASCII 路径规避中文用户名带来的路径兼容问题

## 开发运行

1. 安装 Node.js、Rust、Cargo。
2. 在项目根目录运行 `npm install`。
3. 仅预览前端界面：`npm run dev`
4. 启动 Tauri 开发模式：`npm run tauri dev`
5. 运行前端测试：`npm run test`
6. 运行前端构建：`npm run build`
7. 运行 Rust 测试：`cargo test --manifest-path src-tauri/Cargo.toml`

## 一键脚本

Windows 下提供了两个根目录脚本：

- `start-launcher.ps1`
  以隐藏窗口方式启动 `npm run tauri dev`，并把日志写入 `logs/tauri-dev.log`
- `stop-launcher.ps1`
  根据 `tauri-dev.pid` 优雅停止当前项目对应的 dev 进程树

可直接粘贴到 Windows `Win + R` 的命令：

```text
powershell -NoProfile -ExecutionPolicy Bypass -File "D:\挣钱\反代\start-launcher.ps1"
powershell -NoProfile -ExecutionPolicy Bypass -File "D:\挣钱\反代\stop-launcher.ps1"
```

## 当前实现状态

- 已有 React 前端壳，包含 `Base URL`、`API Key`、`Profile Name` 输入和兼容模式提示
- 已有 Rust 后端骨架，包含 config 生成、auth scrub、ASCII profile 路径、环境检测、安装探测、启动命令构建
- 已加入 provider repair，启动前会尽量把历史 `OpenAI` / `openai` 记录归一成同一标签，减少订阅/API 切换后的记录分裂
- `install_or_repair_codex` 已有基础版，会先检测现有 `codex`，再尝试平台对应的安装命令
- `save_and_launch` 已按终端方式构建启动命令，而不是无终端直接 spawn `codex`

## 已知限制

- Windows 的最佳体验仍应优先走 WSL2；当前原型先实现原生兼容链路和基础安装逻辑
- 第三方服务若不兼容 `Responses API`，可能仍无法启动
- 第一版还没有把 API Key 接入系统安全存储，当前写入的是隔离 profile 中的最小 `auth.json`

## 日志位置

- 开发脚本日志：`logs/tauri-dev.log`
- Dev 进程状态：`tauri-dev.pid`
- 应用运行日志：由 Tauri 后端按 profile 输出到对应 `logs/` 目录
