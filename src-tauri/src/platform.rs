use std::{env, path::PathBuf, process::Command};

use crate::{
    error::LauncherResult,
    installer,
    models::{EnvironmentReport, PlatformKind},
};

pub const DESKTOP_INSTALL_URL: &str = "https://openai.com/codex/get-started/";

#[derive(Debug, Clone)]
pub struct DesktopAppInfo {
    pub installed: bool,
    pub app_path: Option<PathBuf>,
}

pub fn detect_platform() -> PlatformKind {
    if cfg!(target_os = "windows") {
        PlatformKind::Windows
    } else if cfg!(target_os = "macos") {
        PlatformKind::MacOS
    } else {
        PlatformKind::Other
    }
}

pub fn build_wsl_check_command() -> Option<Vec<String>> {
    if cfg!(target_os = "windows") {
        Some(vec!["wsl.exe".to_string(), "--status".to_string()])
    } else {
        None
    }
}

pub fn build_macos_terminal_command(command_line: &str) -> Option<Vec<String>> {
    if cfg!(target_os = "macos") {
        Some(vec![
            "osascript".to_string(),
            "-e".to_string(),
            format!("tell application \"Terminal\" to do script \"{}\"", command_line),
            "-e".to_string(),
            "tell application \"Terminal\" to activate".to_string(),
        ])
    } else {
        None
    }
}

pub fn desktop_install_url() -> &'static str {
    DESKTOP_INSTALL_URL
}

pub fn probe_desktop_app() -> LauncherResult<DesktopAppInfo> {
    match detect_platform() {
        PlatformKind::Windows => probe_windows_desktop_app(),
        PlatformKind::MacOS => Ok(probe_macos_desktop_app()),
        PlatformKind::Other => Ok(DesktopAppInfo {
            installed: false,
            app_path: None,
        }),
    }
}

pub fn open_desktop_install_page() -> LauncherResult<()> {
    let mut command = match detect_platform() {
        PlatformKind::Windows => {
            let mut cmd = Command::new("cmd.exe");
            cmd.arg("/c").arg("start").arg("").arg(DESKTOP_INSTALL_URL);
            cmd
        }
        PlatformKind::MacOS => {
            let mut cmd = Command::new("open");
            cmd.arg(DESKTOP_INSTALL_URL);
            cmd
        }
        PlatformKind::Other => {
            let mut cmd = Command::new("xdg-open");
            cmd.arg(DESKTOP_INSTALL_URL);
            cmd
        }
    };
    command.spawn()?;
    Ok(())
}

fn probe_windows_desktop_app() -> LauncherResult<DesktopAppInfo> {
    let script = r#"
$pkg = Get-AppxPackage OpenAI.Codex | Sort-Object Version -Descending | Select-Object -First 1
if ($pkg) {
  $exe = Join-Path $pkg.InstallLocation 'app\Codex.exe'
  if (Test-Path -LiteralPath $exe) {
    Write-Output $exe
  }
}
"#;

    let output = Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(script)
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let path = stdout
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.exists());

    Ok(DesktopAppInfo {
        installed: path.is_some(),
        app_path: path,
    })
}

fn probe_macos_desktop_app() -> DesktopAppInfo {
    let mut candidates = vec![PathBuf::from("/Applications/Codex.app")];
    if let Ok(home) = env::var("HOME") {
        candidates.push(PathBuf::from(home).join("Applications/Codex.app"));
    }

    for app in candidates {
        let executable = app.join("Contents/MacOS/Codex");
        if executable.exists() {
            return DesktopAppInfo {
                installed: true,
                app_path: Some(executable),
            };
        }
    }

    DesktopAppInfo {
        installed: false,
        app_path: None,
    }
}

fn probe_wsl_available() -> bool {
    let Some(command) = build_wsl_check_command() else {
        return false;
    };
    if command.is_empty() {
        return false;
    }

    let mut process = Command::new(&command[0]);
    for arg in &command[1..] {
        process.arg(arg);
    }
    process.output().map(|output| output.status.success()).unwrap_or(false)
}

pub fn is_wsl_available() -> bool {
    probe_wsl_available()
}

pub fn detect_environment() -> LauncherResult<EnvironmentReport> {
    let codex_version = installer::probe_codex_version()?;
    let codex_installed = codex_version.is_some();
    let desktop_app = probe_desktop_app()?;
    let platform = detect_platform();
    let wsl_available = probe_wsl_available();
    let mut details = Vec::new();

    details.push(if desktop_app.installed {
        "检测到官方 Codex 桌面版。".to_string()
    } else {
        "未检测到官方 Codex 桌面版，可打开官方页面下载安装。".to_string()
    });
    details.push(if codex_installed {
        "检测到现有 Codex CLI。".to_string()
    } else {
        "尚未检测到 Codex CLI，可尝试安装或修复。".to_string()
    });
    if matches!(platform, PlatformKind::Windows) {
        details.push(if wsl_available {
            "Windows 已检测到 WSL，可作为兼容模式入口。".to_string()
        } else {
            "Windows 尚未检测到 WSL，将回退到原生命令链路。".to_string()
        });
    }

    Ok(EnvironmentReport {
        status: "ok".to_string(),
        summary: if codex_installed {
            "环境检测完成，可以直接尝试启动。".to_string()
        } else {
            "环境检测完成，尚未检测到 Codex CLI。".to_string()
        },
        details,
        platform,
        codex_installed,
        codex_version,
        desktop_app_installed: desktop_app.installed,
        desktop_app_path: desktop_app.app_path.map(|path| path.to_string_lossy().to_string()),
        wsl_check_command: build_wsl_check_command(),
        wsl_available,
        macos_terminal_command_example: build_macos_terminal_command("codex --version"),
    })
}

#[cfg(test)]
mod tests {
    use crate::models::PlatformKind;

    use super::desktop_install_url;

    #[test]
    fn desktop_install_page_uses_official_openai_url() {
        assert_eq!(desktop_install_url(), "https://openai.com/codex/get-started/");
    }

    #[test]
    fn platform_kind_enum_still_resolves() {
        assert!(matches!(PlatformKind::Other, PlatformKind::Other));
    }
}
