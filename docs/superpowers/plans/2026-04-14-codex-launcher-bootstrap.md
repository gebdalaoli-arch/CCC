# Codex Launcher Bootstrap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first working Tauri-based Codex Launcher that can collect a custom OpenAI-compatible Base URL and API key, prepare an isolated CODEX_HOME, scrub subscription auth state, and install or launch Codex CLI on Windows/macOS.

**Architecture:** Use a Tauri 2 desktop shell with a small web UI, a Rust backend split into settings/profile/auth/platform/installer/launcher modules, and a thin stateful frontend that calls typed Tauri commands. Keep Codex CLI external and launch it in system terminals instead of embedding a terminal. Generate a compatibility-focused isolated profile that forces `preferred_auth_method = "apikey"` and rewrites `auth.json` before every launch.

**Tech Stack:** Tauri 2, Rust, TypeScript, Vite, React, Vitest, Rust tests

---

### Task 1: Scaffold the desktop app and baseline tooling

**Files:**
- Create: `package.json`
- Create: `tsconfig.json`
- Create: `vite.config.ts`
- Create: `index.html`
- Create: `src/main.tsx`
- Create: `src/App.tsx`
- Create: `src/styles.css`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `.gitignore`

- [ ] **Step 1: Write the failing frontend smoke test**

```tsx
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import App from "../src/App";

describe("App", () => {
  it("renders launcher title", () => {
    render(<App />);
    expect(screen.getByText(/Codex Launcher/i)).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- --run`
Expected: FAIL because the app scaffold and test wiring do not exist yet

- [ ] **Step 3: Create the minimal Tauri + Vite app shell**

```tsx
export default function App() {
  return (
    <main>
      <h1>Codex Launcher</h1>
      <p>Connect your OpenAI-compatible gateway and launch Codex CLI.</p>
    </main>
  );
}
```

```rust
fn main() {
    codex_launcher_lib::run();
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test -- --run`
Expected: PASS with one rendered smoke test

- [ ] **Step 5: Commit**

```bash
git add .gitignore package.json tsconfig.json vite.config.ts index.html src src-tauri
git commit -m "feat: scaffold tauri codex launcher"
```

### Task 2: Implement shared models, settings, isolated profile generation, and auth scrub logic

**Files:**
- Create: `src/lib/types.ts`
- Create: `src-tauri/src/models.rs`
- Create: `src-tauri/src/settings.rs`
- Create: `src-tauri/src/profile.rs`
- Create: `src-tauri/src/auth.rs`
- Create: `src-tauri/src/logging.rs`
- Test: `src-tauri/src/profile.rs`
- Test: `src-tauri/src/settings.rs`
- Test: `src-tauri/src/auth.rs`

- [ ] **Step 1: Write the failing Rust tests**

```rust
#[test]
fn generates_ascii_profile_paths() {
    let paths = build_profile_paths("default", PlatformKind::MacOS, "501").unwrap();
    assert!(paths.codex_home.to_string_lossy().contains("CodexLauncher"));
    assert!(!paths.codex_home.to_string_lossy().contains("中文"));
}

#[test]
fn renders_gateway_config_without_inline_secret() {
    let config = render_gateway_config("https://example.com/v1").unwrap();
    assert!(config.contains("preferred_auth_method = \"apikey\""));
    assert!(config.contains("[model_providers.openai]"));
    assert!(!config.contains("sk-"));
}

#[test]
fn strips_subscription_tokens_from_auth_payload() {
    let payload = sanitize_auth_json(r#"{"OPENAI_API_KEY":"sk-test","tokens":{"access_token":"x"}}"#).unwrap();
    assert!(payload.contains("\"OPENAI_API_KEY\""));
    assert!(!payload.contains("access_token"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test profile::tests::generates_ascii_profile_paths settings::tests::renders_gateway_config_without_inline_secret auth::tests::strips_subscription_tokens_from_auth_payload`
Expected: FAIL because the modules and functions do not exist yet

- [ ] **Step 3: Write minimal profile/settings implementation**

```rust
pub fn render_gateway_config(base_url: &str) -> Result<String, LauncherError> {
    Ok(format!(
        "preferred_auth_method = \"apikey\"\nmodel_provider = \"openai\"\n\n[model_providers.openai]\nname = \"OpenAI\"\nbase_url = \"{}\"\nwire_api = \"responses\"\nrequires_openai_auth = true\n",
        base_url
    ))
}
```

```rust
pub fn sanitize_profile_name(input: &str) -> String {
    input
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() || ch == '-' { ch } else { '-' })
        .collect()
}
```

```rust
pub fn sanitize_auth_json(input: &str) -> Result<String, LauncherError> {
    let mut value: serde_json::Value = serde_json::from_str(input)?;
    if let Some(object) = value.as_object_mut() {
        object.remove("tokens");
    }
    Ok(serde_json::to_string_pretty(&value)?)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test profile::tests::generates_ascii_profile_paths settings::tests::renders_gateway_config_without_inline_secret auth::tests::strips_subscription_tokens_from_auth_payload`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/lib/types.ts src-tauri/src/models.rs src-tauri/src/settings.rs src-tauri/src/profile.rs src-tauri/src/auth.rs src-tauri/src/logging.rs
git commit -m "feat: add isolated profile and auth scrub logic"
```

### Task 3: Implement platform detection, installer checks, and launch commands

**Files:**
- Create: `src-tauri/src/error.rs`
- Create: `src-tauri/src/platform.rs`
- Create: `src-tauri/src/installer.rs`
- Create: `src-tauri/src/launcher.rs`
- Modify: `src-tauri/src/main.rs`
- Test: `src-tauri/src/platform.rs`
- Test: `src-tauri/src/installer.rs`

- [ ] **Step 1: Write the failing Rust tests for command building**

```rust
#[test]
fn builds_windows_wsl_launch_command() {
    let cmd = build_launch_command(&LaunchRequest::windows_wsl("default"));
    assert!(cmd.program.contains("wsl"));
    assert!(cmd.args.join(" ").contains("OPENAI_API_KEY"));
}

#[test]
fn detects_codex_from_version_output() {
    let status = parse_codex_version_output("codex 0.120.0").unwrap();
    assert_eq!(status.version.as_deref(), Some("0.120.0"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test platform::tests::builds_windows_wsl_launch_command installer::tests::detects_codex_from_version_output`
Expected: FAIL because launcher and installer helpers do not exist yet

- [ ] **Step 3: Implement command builders and installer probes**

```rust
pub fn parse_codex_version_output(stdout: &str) -> Result<CodexInstallStatus, LauncherError> {
    let version = stdout.split_whitespace().nth(1).map(str::to_string);
    Ok(CodexInstallStatus { installed: version.is_some(), version })
}
```

```rust
pub fn build_windows_wsl_env_snippet(req: &LaunchRequest) -> String {
    format!(
        "export CODEX_HOME='{}'; export OPENAI_API_KEY='${{OPENAI_API_KEY}}'; codex",
        req.codex_home
    )
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test platform::tests::builds_windows_wsl_launch_command installer::tests::detects_codex_from_version_output`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/error.rs src-tauri/src/platform.rs src-tauri/src/installer.rs src-tauri/src/launcher.rs src-tauri/src/main.rs
git commit -m "feat: add platform detection and launch commands"
```

### Task 4: Build the launcher UI and wire it to Tauri commands

**Files:**
- Create: `src/components/launcher-form.tsx`
- Create: `src/components/environment-panel.tsx`
- Create: `src/components/action-bar.tsx`
- Create: `src/components/compatibility-note.tsx`
- Create: `src/hooks/use-launcher-state.ts`
- Modify: `src/App.tsx`
- Modify: `src/styles.css`
- Test: `src/App.test.tsx`

- [ ] **Step 1: Write the failing UI test**

```tsx
it("disables launch until base url and api key are filled", () => {
  render(<App />);
  expect(screen.getByRole("button", { name: /保存并启动/i })).toBeDisabled();
});

it("shows compatibility mode notice", () => {
  render(<App />);
  expect(screen.getByText(/会清理订阅登录残留/i)).toBeInTheDocument();
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- --run`
Expected: FAIL because the form and action state do not exist yet

- [ ] **Step 3: Implement the minimal connected UI**

```tsx
const canLaunch = baseUrl.trim() !== "" && apiKey.trim() !== "";

<button disabled={!canLaunch} onClick={handleSaveAndLaunch}>
  保存并启动
</button>
```

```tsx
<label>
  Base URL
  <input value={baseUrl} onChange={(event) => setBaseUrl(event.target.value)} />
</label>
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test -- --run`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/components src/hooks src/App.tsx src/styles.css src/App.test.tsx
git commit -m "feat: add launcher configuration ui"
```

### Task 5: Add run scripts, docs, and cross-check verification

**Files:**
- Create: `README.md`
- Create: `start-launcher.ps1`
- Create: `stop-launcher.ps1`
- Modify: `package.json`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Write the failing verification checklist in documentation**

```md
- [ ] `npm test -- --run`
- [ ] `cargo test`
- [ ] `npm run tauri build` or `npm run tauri dev`
```

- [ ] **Step 2: Run the current verification commands to confirm gaps**

Run: `npm test -- --run`
Expected: FAIL or partial FAIL until the scripts and docs are in place

- [ ] **Step 3: Implement scripts and documentation**

```powershell
Start-Process -FilePath "npm" -ArgumentList "run", "tauri", "dev" -WindowStyle Hidden
```

```json
"scripts": {
  "dev": "vite",
  "build": "tsc && vite build",
  "test": "vitest",
  "tauri": "tauri"
}
```

- [ ] **Step 4: Run full verification**

Run: `npm test -- --run`
Expected: PASS

Run: `cargo test`
Expected: PASS

Run: `npm run build`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add README.md start-launcher.ps1 stop-launcher.ps1 package.json src-tauri/Cargo.toml
git commit -m "chore: add launcher run scripts and docs"
```
