# Codex Launcher Design

## Overview

目标是交付一个面向普通用户的跨平台原生桌面启动器，支持 Windows 和 macOS。用户仅需粘贴第三方 `OpenAI-compatible` 服务的 `Base URL` 与 `API Key`，应用即可自动检查、安装并启动 `Codex CLI`。启动器必须主动规避中文用户名导致的路径、编码和默认用户目录问题，并尽量不污染用户既有的 Codex 配置。

该产品第一版仅面向 `Codex CLI`，不嵌入官方桌面端，也不在应用内重建终端界面。启动器负责完成环境检查、安装、配置生成、密钥管理和终端交接，`Codex CLI` 继续在系统终端中运行，确保兼容官方 CLI 的交互体验。

## Goals

- 支持 Windows 与 macOS 的本地一键启动体验。
- 支持第三方 `OpenAI-compatible` 网关，输入 `Base URL + API Key` 即可启动。
- 自动检测 `Codex CLI` 是否存在，不存在时执行安装或修复。
- 通过隔离 `CODEX_HOME` 与 ASCII 路径规避中文用户名问题。
- 不将 API Key 写入 `config.toml`。
- 为常见失败场景提供可读错误信息、修复建议和诊断日志。

## Non-Goals

- 不支持非 `OpenAI-compatible` 协议。
- 不在第一版支持多账号同步或云端配置同步。
- 不在第一版嵌入终端组件。
- 不在第一版实现自动更新服务端或远程授权系统。
- 不修改用户已有的 `~/.codex` 或 `~/.config` 中的现有个人设置。

## Product Decisions

### Runtime model

启动器本身使用 `Tauri 2 + Rust` 实现，负责本地系统能力。前端提供一个单窗口流程式界面，后端负责环境检测、安装、配置、密钥和进程拉起。`Codex CLI` 作为外部依赖运行，不被静态打包进启动器本体。

### Windows strategy

Windows 默认路径选择 `WSL2`。OpenAI 官方文档明确说明 `Windows support is experimental`，且推荐 `WSL2 workspace` 作为更稳妥的运行方式，因此启动器在 Windows 上优先检测并使用 WSL 内部环境安装和执行 `Codex CLI`。如未检测到 WSL2，则界面提示用户安装；后续可以保留实验性原生模式，但第一版不默认开放。

### macOS strategy

macOS 直接在本机安装并运行 `Codex CLI`。优先使用 `brew install --cask codex`，其次回退到 `npm install -g @openai/codex`。启动器通过 Terminal 或 AppleScript 创建新终端窗口并注入运行时环境变量。

## System Architecture

### Frontend

前端负责以下职责：

- 渲染连接配置表单。
- 展示环境检测结果与修复按钮。
- 显示安装进度、启动结果与错误摘要。
- 允许用户查看日志、重新检测和重新安装。

界面采用三段式流程：

1. 连接配置
2. 环境检查
3. 启动 Codex

### Backend

Rust 后端按责任拆分为以下模块：

- `models`: 前后端共享的数据结构和状态枚举。
- `settings`: 启动器本地设置的读取、保存与迁移。
- `profile`: 生成隔离 `CODEX_HOME`、配置文件和 ASCII 运行目录。
- `secrets`: 平台安全存储封装。Windows 使用 Credential Manager 或 DPAPI，macOS 使用 Keychain。
- `platform`: 平台检测、安装器发现、终端拉起与命令执行。
- `installer`: `Codex CLI` 的检查、安装与修复。
- `launcher`: 注入环境变量并启动 `codex`。
- `logging`: 结构化日志、诊断信息和脱敏输出。

## Data Model

### Launcher settings

启动器自己的非敏感设置保存在本地 JSON 文件中，包含：

- `profile_name`
- `base_url`
- `preferred_install_method`
- `platform_mode`
- `last_launch_at`
- `last_detected_codex_version`
- `last_environment_report`

### Secrets

敏感数据不写入本地设置文件，只通过安全存储保存：

- `api_key`

### Runtime profile

`CODEX_HOME` 目录由启动器维护，内含：

- `config.toml`
- `auth.json`
- `log/`
- `sessions/`
- 其他 `Codex CLI` 运行时文件

## Path Strategy

### Windows

启动器配置文件位于：

- `%LOCALAPPDATA%\CodexLauncher\settings.json`

运行时 `CODEX_HOME` 位于 WSL 用户目录中的 ASCII 路径：

- `~/.codex-launcher`

这样 `Codex CLI` 的真实工作目录完全避开 Windows 中文用户名路径，不依赖 `C:\Users\中文名\.codex`。

### macOS

启动器配置文件位于：

- `~/Library/Application Support/CodexLauncher/settings.json`

运行时 `CODEX_HOME` 位于：

- `/Users/Shared/CodexLauncher/profiles/<user-id>`

目录名保持 ASCII，目录权限收窄到当前用户可读写。

## Codex Configuration Strategy

启动器写入面向 `Codex CLI` 当前兼容性问题的最小 `config.toml`。由于现有运行现象表明，当本地存在 ChatGPT 订阅登录态或旧 provider 状态时，Codex 容易重新偏向订阅模式，因此第一版不使用自定义 provider ID，而是显式覆盖内建 `openai` provider，并强制偏好 API Key 模式。配置形如：

```toml
preferred_auth_method = "apikey"
model_provider = "openai"

[model_providers.openai]
name = "OpenAI"
base_url = "https://example.com/v1"
wire_api = "responses"
requires_openai_auth = true
```

`API Key` 不写入 `config.toml`。启动器默认将 API Key 存入系统安全存储，但为了兼容 Codex 当前鉴权实现，启动前会把 API Key 同步到隔离 profile 下的最小 `auth.json`，并同步注入 `OPENAI_API_KEY` 环境变量。每次启动前还会主动移除或覆盖 `tokens` 字段，避免旧的 ChatGPT 登录态把运行模式重新拉回订阅制。

## Installation Flow

### Shared rules

- 先检测是否已经存在可执行的 `codex`。
- 若存在，记录版本并跳过安装。
- 若不存在，按平台选择合适安装方法。
- 若首选方法失败，进入回退安装链路。

### Windows

检测：

- `wsl.exe` 是否存在。
- `wsl -l -v` 是否能返回有效发行版。
- WSL 内是否已存在 `codex`、`npm` 或 `node`。

安装：

1. 若 WSL 内已存在 `codex`，直接使用。
2. 若缺少 `node/npm`，先在 WSL 内安装 Node LTS。
3. 运行 `npm install -g @openai/codex`。
4. 验证 `codex --version`。

### macOS

检测：

- `codex`
- `brew`
- `npm`

安装顺序：

1. `brew install --cask codex`
2. `npm install -g @openai/codex`
3. 若未来确认官方独立二进制分发稳定，可加入 release binary 下载回退

## Launch Flow

1. 读取本地设置与安全存储中的 API Key。
2. 生成或刷新隔离 profile。
3. 写入 `config.toml`。
4. 生成最小 `auth.json`，仅保留 API Key 相关字段并清除 `tokens`。
5. 准备运行环境变量：
   - `CODEX_HOME`
   - `OPENAI_API_KEY`
   - 可选 `OPENAI_BASE_URL` 兼容覆盖
6. 拉起终端并执行 `codex`。
7. 更新最近启动时间和版本缓存。

Windows 通过 `wsl.exe` 或 Windows Terminal 拉起，macOS 通过 Terminal / AppleScript 拉起。

## Compatibility Guardrails

根据现有可用启动器行为与已知 Codex 问题，第一版实现必须额外做以下兼容动作：

- 每次启动都将目标 profile 的 `preferred_auth_method` 固定为 `apikey`。
- 每次启动都覆盖隔离 profile 中的 `auth.json`，确保其中没有历史 `tokens`。
- 每次启动都只使用启动器自己的隔离 `CODEX_HOME`，绝不复用用户默认 `~/.codex`。
- 每次启动都将 `OPENAI_API_KEY` 注入环境变量，即使未来额外支持自定义 provider，也保留这一兼容注入。
- 若检测到历史 session 或状态文件中存在不一致 provider 标记，记录诊断并在需要时执行 profile repair。

## Error Handling

错误分为以下类别：

- `InvalidConfig`: 用户输入为空、URL 不合法或格式错误。
- `UnsupportedProvider`: 第三方服务不兼容 `responses` 风格。
- `AuthFallbackDetected`: 检测到 profile 内残留订阅 token 或鉴权状态被切回 ChatGPT。
- `InstallUnavailable`: 平台缺少安装所需依赖且自动修复失败。
- `MissingWsl`: Windows 未安装 WSL2。
- `LaunchFailed`: `codex` 拉起失败或返回非零退出码。
- `SecretStoreFailed`: 无法写入或读取系统安全存储。

每类错误都需要给出：

- 人类可读的原因摘要
- 建议的下一步操作
- 可复制的诊断信息

## Logging and Diagnostics

日志使用统一结构化格式，并写入启动器自己的日志目录。日志中不得记录完整 API Key，所有敏感值必须脱敏输出，只展示首尾少量字符。诊断包导出时排除 secret，仅包含：

- 平台与版本信息
- 环境检测结果
- 安装器执行结果
- `codex --version` 或失败消息
- 脱敏后的配置摘要

## UI Specification

### Main form

字段：

- `Base URL`
- `API Key`
- `Profile Name`

按钮：

- `检测环境`
- `安装或修复 Codex`
- `保存并启动`
- `打开日志`

### Environment panel

展示：

- 平台类型
- 安装模式
- `Codex CLI` 是否已安装
- WSL 状态或 macOS 安装状态
- 上次检测时间

### Advanced options

包含：

- Windows 平台模式说明
- 安装方式优先级
- 是否启用自动修复
- 诊断导出

## Testing Strategy

### Automated tests

- 单元测试：
  - `Base URL` 校验
  - 配置模板生成
  - 最小 `auth.json` 生成与 token 清理
  - 脱敏逻辑
  - ASCII 路径生成
  - 平台命令构建
- 集成测试：
  - 模拟 `codex --version`
  - 模拟 WSL 检测输出
  - 模拟安装成功与失败
  - 模拟终端拉起命令
  - 模拟遗留 `auth.json` 中存在 `tokens` 时的清理行为

### Manual validation

- Windows 中文用户名环境
- Windows 无 WSL2 环境
- Windows 已安装 WSL2 但无 Node 环境
- macOS 中文用户名环境
- 第三方 Base URL 填错场景
- API Key 缺失场景

## Risks and Mitigations

### Risk 1: Windows 原生兼容性不足

缓解方式：默认使用 WSL2，第一版不承诺原生 Windows CLI 模式。

### Risk 2: 第三方 OpenAI-compatible 网关兼容性参差不齐

缓解方式：产品文案明确限定 `Responses API` 兼容；检测阶段给出提前提示。

### Risk 3: Codex 认证状态会被历史登录态或 provider 状态污染

缓解方式：使用隔离 `CODEX_HOME`、强制 `preferred_auth_method = "apikey"`、覆盖最小 `auth.json` 并在启动前清理 `tokens`。

### Risk 4: 自动安装链路受用户系统环境影响

缓解方式：安装器实现多级回退策略，并提供手动修复提示。

### Risk 5: 中文用户名问题不仅体现在文件路径

缓解方式：统一使用 ASCII 运行目录、UTF-8 写文件、显式设置环境变量并尽量减少对默认 shell profile 的依赖。

## Acceptance Criteria

- 用户可在 Windows 和 macOS 上输入 `Base URL + API Key` 并成功拉起 `Codex CLI`。
- 当 `Codex CLI` 缺失时，启动器能检测并执行安装或给出明确修复提示。
- 启动器不会把 API Key 写入 `config.toml`。
- 启动器每次启动都能清除隔离 profile 中的订阅 token 残留。
- 运行时不依赖用户默认中文路径中的 `~/.codex`。
- 关键错误场景有清晰 UI 提示与日志记录。
