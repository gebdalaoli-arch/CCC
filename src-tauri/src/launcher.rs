use std::{collections::BTreeMap, process::Command};

use crate::{
    auth,
    error::{LauncherError, LauncherResult},
    models::{LaunchRequest, LaunchResponse, PlatformKind},
    platform, profile, repair, settings,
};

pub fn build_launch_plan(request: &LaunchRequest) -> LauncherResult<(Vec<String>, BTreeMap<String, String>, profile::ProfilePaths)> {
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

    let mut codex_args = vec![
        "--config".to_string(),
        paths.config_path.to_string_lossy().to_string(),
    ];
    codex_args.extend(request.extra_args.clone());
    let command = build_terminal_launch_command(&env, &codex_args)?;

    Ok((command, env, paths))
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
    let (command, env, paths) = build_launch_plan(&request)?;

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
    use crate::models::LaunchRequest;

    #[test]
    fn launch_command_includes_required_env_injection() {
        let request = LaunchRequest {
            codex_home: Some("D:\\codex_data\\launcher-test".to_string()),
            profile_name: Some("测试 Profile".to_string()),
            openai_api_key: "sk-test-key".to_string(),
            openai_base_url: Some("https://proxy.local/v1".to_string()),
            extra_args: vec!["chat".to_string()],
            launch_now: false,
        };

        let (command, env, paths) = build_launch_plan(&request).expect("launch plan should build");

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
    }
}
