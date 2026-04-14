use std::{collections::BTreeMap, process::Command};

use crate::{
    auth,
    error::{LauncherError, LauncherResult},
    models::{LaunchMode, LaunchRequest, LaunchResponse, PlatformKind},
    platform, profile, repair, settings,
};

pub fn build_launch_plan(request: &LaunchRequest) -> LauncherResult<(Vec<String>, BTreeMap<String, String>, profile::ProfilePaths, String, String)> {
    let paths = profile::resolve_profile_paths(request.codex_home.as_deref(), request.profile_name.as_deref());
    profile::ensure_profile_dirs(&paths)?;

    let mut env = BTreeMap::new();
    env.insert(
        "CODEX_HOME".to_string(),
        paths.codex_home.to_string_lossy().to_string(),
    );
    env.insert("OPENAI_API_KEY".to_string(), request.openai_api_key.clone());

    if let Some(base_url) = request
        .openai_base_url
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        env.insert("OPENAI_BASE_URL".to_string(), base_url.to_string());
    }

    let desktop_app = platform::probe_desktop_app()?;
    let (command, resolved_launch_mode, launch_target) =
        resolve_launch_command(request, &env, &paths, desktop_app.app_path.as_deref())?;

    Ok((command, env, paths, resolved_launch_mode, launch_target))
}

fn resolve_launch_command(
    request: &LaunchRequest,
    env: &BTreeMap<String, String>,
    paths: &profile::ProfilePaths,
    desktop_app_path: Option<&std::path::Path>,
) -> LauncherResult<(Vec<String>, String, String)> {
    match request.launch_mode {
        LaunchMode::DesktopPreferred => {
            if let Some(path) = desktop_app_path {
                let command = build_desktop_launch_command(path);
                Ok((command, "desktop_preferred".to_string(), "desktop".to_string()))
            } else {
                let command = build_cli_launch_command(env, paths, &request.extra_args)?;
                Ok((command, "desktop_preferred".to_string(), "cli_fallback".to_string()))
            }
        }
        LaunchMode::DesktopOnly => {
            let Some(path) = desktop_app_path else {
                return Err(LauncherError::CommandFailed(
                    "Codex desktop app is not installed. Please install it first.".to_string(),
                ));
            };
            let command = build_desktop_launch_command(path);
            Ok((command, "desktop_only".to_string(), "desktop".to_string()))
        }
        LaunchMode::CliOnly => {
            let command = build_cli_launch_command(env, paths, &request.extra_args)?;
            Ok((command, "cli_only".to_string(), "cli".to_string()))
        }
    }
}

fn build_cli_launch_command(
    env: &BTreeMap<String, String>,
    paths: &profile::ProfilePaths,
    extra_args: &[String],
) -> LauncherResult<Vec<String>> {
    let mut codex_args = vec![
        "--config".to_string(),
        paths.config_path.to_string_lossy().to_string(),
    ];
    codex_args.extend(extra_args.to_vec());
    build_terminal_launch_command(env, &codex_args)
}

fn build_desktop_launch_command(desktop_app_path: &std::path::Path) -> Vec<String> {
    vec![desktop_app_path.to_string_lossy().to_string()]
}

fn build_terminal_launch_command(
    env: &BTreeMap<String, String>,
    codex_args: &[String],
) -> LauncherResult<Vec<String>> {
    let codex_command = build_codex_shell_command(env, codex_args);

    let command = match platform::detect_platform() {
        PlatformKind::Windows => vec![
            "cmd.exe".to_string(),
            "/c".to_string(),
            "start".to_string(),
            "Codex Launcher".to_string(),
            "powershell.exe".to_string(),
            "-NoExit".to_string(),
            "-Command".to_string(),
            codex_command,
        ],
        PlatformKind::MacOS => vec![
            "osascript".to_string(),
            "-e".to_string(),
            format!("tell application \"Terminal\" to do script \"{}\"", escape_for_applescript(&codex_command)),
            "-e".to_string(),
            "tell application \"Terminal\" to activate".to_string(),
        ],
        PlatformKind::Other => vec![
            "sh".to_string(),
            "-lc".to_string(),
            codex_command,
        ],
    };

    Ok(command)
}

fn build_codex_shell_command(env: &BTreeMap<String, String>, codex_args: &[String]) -> String {
    let mut parts = Vec::new();
    match platform::detect_platform() {
        PlatformKind::Windows => {
            for (key, value) in env {
                parts.push(format!("$env:{key} = '{}';", escape_for_single_quotes(value)));
            }
            let arg_text = codex_args
                .iter()
                .map(|arg| format!("'{}'", escape_for_single_quotes(arg)))
                .collect::<Vec<_>>()
                .join(" ");
            parts.push(format!("codex {arg_text}"));
        }
        _ => {
            for (key, value) in env {
                parts.push(format!("export {key}='{}';", escape_for_single_quotes(value)));
            }
            let arg_text = codex_args
                .iter()
                .map(|arg| format!("'{}'", escape_for_single_quotes(arg)))
                .collect::<Vec<_>>()
                .join(" ");
            parts.push(format!("codex {arg_text}"));
        }
    }

    parts.join(" ")
}

fn escape_for_single_quotes(input: &str) -> String {
    input.replace('\'', "''")
}

fn escape_for_applescript(input: &str) -> String {
    input.replace('\\', "\\\\").replace('\"', "\\\"")
}

fn spawn_codex(command: &[String], env: &BTreeMap<String, String>) -> LauncherResult<()> {
    let mut process = Command::new(&command[0]);
    for arg in &command[1..] {
        process.arg(arg);
    }
    process.envs(env);
    let child = process.spawn()?;
    if child.id() == 0 {
        return Err(LauncherError::CommandFailed("failed to spawn codex terminal process".to_string()));
    }
    Ok(())
}

pub fn save_and_launch(request: LaunchRequest) -> LauncherResult<LaunchResponse> {
    let (command, env, paths, resolved_launch_mode, launch_target) = build_launch_plan(&request)?;

    settings::write_codex_config(&paths.config_path, request.openai_base_url.as_deref())?;
    auth::write_minimal_auth(&paths.auth_path, None, Some(&request.openai_api_key))?;
    let bridge_report = repair::bridge_history_from_default_sources(&paths.codex_home)?;
    let repair_report = repair::normalize_openai_provider_labels(&paths.codex_home)?;

    let started = if request.launch_now {
        spawn_codex(&command, &env)?;
        true
    } else {
        false
    };

    Ok(LaunchResponse {
        success: true,
        started,
        resolved_launch_mode,
        launch_target,
        command,
        env,
        config_path: paths.config_path.to_string_lossy().to_string(),
        auth_path: paths.auth_path.to_string_lossy().to_string(),
        message: if started {
            format!(
                "Codex process launched. History bridge scanned {} sources, copied {} session files, added {} index entries, copied {} sqlite rows; provider repair updated {} session files and {} sqlite rows.",
                bridge_report.sources_scanned,
                bridge_report.session_files_copied,
                bridge_report.session_index_entries_added,
                bridge_report.sqlite_rows_copied,
                repair_report.session_files_changed,
                repair_report.sqlite_rows_changed
            )
        } else {
            format!(
                "Launch plan prepared. History bridge scanned {} sources, copied {} session files, added {} index entries, copied {} sqlite rows; provider repair updated {} session files and {} sqlite rows.",
                bridge_report.sources_scanned,
                bridge_report.session_files_copied,
                bridge_report.session_index_entries_added,
                bridge_report.sqlite_rows_copied,
                repair_report.session_files_changed,
                repair_report.sqlite_rows_changed
            )
        },
    })
}

#[cfg(test)]
mod tests {
    use super::build_launch_plan;
    use crate::models::{LaunchMode, LaunchRequest};

    #[test]
    fn launch_command_includes_required_env_injection() {
        let request = LaunchRequest {
            codex_home: Some("D:\\codex_data\\launcher-test".to_string()),
            profile_name: Some("测试 Profile".to_string()),
            openai_api_key: "sk-test-key".to_string(),
            openai_base_url: Some("https://proxy.local/v1".to_string()),
            launch_mode: LaunchMode::CliOnly,
            extra_args: vec!["chat".to_string()],
            launch_now: false,
        };

        let (command, env, paths, resolved_launch_mode, launch_target) =
            build_launch_plan(&request).expect("launch plan should build");

        assert!(!command.is_empty());
        assert!(command.join(" ").contains("OPENAI_API_KEY"));
        assert_eq!(env.get("OPENAI_API_KEY").map(String::as_str), Some("sk-test-key"));
        assert_eq!(
            env.get("OPENAI_BASE_URL").map(String::as_str),
            Some("https://proxy.local/v1")
        );
        assert_eq!(
            env.get("CODEX_HOME").map(String::as_str),
            Some(paths.codex_home.to_string_lossy().as_ref())
        );
        assert_eq!(resolved_launch_mode, "cli_only");
        assert_eq!(launch_target, "cli");
    }

    #[test]
    fn desktop_only_requires_desktop_app() {
        let request = LaunchRequest {
            codex_home: Some("D:\\codex_data\\launcher-test".to_string()),
            profile_name: Some("desktop profile".to_string()),
            openai_api_key: "sk-test-key".to_string(),
            openai_base_url: Some("https://proxy.local/v1".to_string()),
            launch_mode: LaunchMode::DesktopOnly,
            extra_args: vec![],
            launch_now: false,
        };

        let result = build_launch_plan(&request);
        if cfg!(target_os = "windows") || cfg!(target_os = "macos") {
            if result.is_err() {
                assert!(result
                    .err()
                    .unwrap()
                    .to_string()
                    .contains("Codex desktop app is not installed"));
            }
        }
    }
}
