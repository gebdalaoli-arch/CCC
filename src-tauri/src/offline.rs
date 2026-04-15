use std::{
    env,
    path::PathBuf,
};

use tauri::{path::BaseDirectory, AppHandle, Manager};

use crate::{error::LauncherResult, profile::ProfilePaths};

const OFFLINE_RUNTIME_DIR: &str = "offline-runtime";

#[derive(Debug, Clone)]
pub struct OfflineCliInfo {
    pub available: bool,
    pub runtime_dir: Option<PathBuf>,
    pub node_path: Option<PathBuf>,
    pub entry_script: Option<PathBuf>,
}

pub fn resolve_offline_cli(app: Option<&AppHandle>) -> LauncherResult<OfflineCliInfo> {
    let candidates = candidate_runtime_dirs(app)?;
    for runtime_dir in candidates {
        let node_path = runtime_dir.join(node_binary_name());
        let entry_script = runtime_dir
            .join("node_modules")
            .join("@openai")
            .join("codex")
            .join("bin")
            .join("codex.js");

        if node_path.exists() && entry_script.exists() {
            return Ok(OfflineCliInfo {
                available: true,
                runtime_dir: Some(runtime_dir),
                node_path: Some(node_path),
                entry_script: Some(entry_script),
            });
        }
    }

    Ok(OfflineCliInfo {
        available: false,
        runtime_dir: None,
        node_path: None,
        entry_script: None,
    })
}

pub fn build_offline_cli_args(paths: &ProfilePaths, extra_args: &[String]) -> Vec<String> {
    let mut args = vec![
        "--config".to_string(),
        paths.config_path.to_string_lossy().to_string(),
    ];
    args.extend(extra_args.to_vec());
    args
}

fn candidate_runtime_dirs(app: Option<&AppHandle>) -> LauncherResult<Vec<PathBuf>> {
    let mut candidates = Vec::new();

    if let Some(app) = app {
        if let Ok(resource_dir) = app.path().resolve(OFFLINE_RUNTIME_DIR, BaseDirectory::Resource) {
            candidates.push(resource_dir);
        }
    }

    if let Ok(current_dir) = env::current_dir() {
        candidates.push(current_dir.join("src-tauri").join(OFFLINE_RUNTIME_DIR));
        candidates.push(current_dir.join(OFFLINE_RUNTIME_DIR));
    }

    Ok(dedupe_paths(candidates))
}

fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = std::collections::HashSet::new();
    paths
        .into_iter()
        .filter(|path| seen.insert(path.to_string_lossy().to_string()))
        .collect()
}

fn node_binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "node.exe"
    } else {
        "node"
    }
}

#[cfg(test)]
mod tests {
    use crate::profile::resolve_profile_paths;

    use super::build_offline_cli_args;

    #[test]
    fn offline_cli_args_include_config_and_extra_args() {
        let paths = resolve_profile_paths(Some("D:\\codex_data\\offline-test"), Some("offline"));
        let args = build_offline_cli_args(&paths, &["chat".to_string()]);
        assert_eq!(args.first().map(String::as_str), Some("--config"));
        assert!(args.iter().any(|arg| arg == "chat"));
    }
}
