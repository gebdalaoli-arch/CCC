use std::process::Command;

use crate::{
    error::LauncherResult,
    installer,
    models::{EnvironmentReport, PlatformKind},
};

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
    let platform = detect_platform();
    let wsl_available = probe_wsl_available();
    let mut details = Vec::new();

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
        wsl_check_command: build_wsl_check_command(),
        wsl_available,
        macos_terminal_command_example: build_macos_terminal_command("codex --version"),
    })
}
