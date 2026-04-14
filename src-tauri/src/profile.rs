use std::{fs, path::PathBuf};

use crate::error::LauncherResult;

#[derive(Debug, Clone)]
pub struct ProfilePaths {
    pub codex_home: PathBuf,
    pub profile_dir: PathBuf,
    pub config_path: PathBuf,
    pub auth_path: PathBuf,
    pub logs_dir: PathBuf,
}

pub fn sanitize_ascii_component(input: &str) -> String {
    let mut sanitized = String::new();
    let mut last_was_separator = false;

    for ch in input.chars() {
        let mapped = if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            ch
        } else if ch.is_whitespace() {
            '-'
        } else {
            '_'
        };

        if matches!(mapped, '-' | '_') {
            if !last_was_separator {
                sanitized.push(mapped);
            }
            last_was_separator = true;
        } else {
            sanitized.push(mapped);
            last_was_separator = false;
        }
    }

    let trimmed = sanitized.trim_matches(|c| c == '-' || c == '_').to_string();
    if trimmed.is_empty() {
        "default".to_string()
    } else {
        trimmed
    }
}

#[cfg(target_os = "windows")]
fn default_ascii_codex_home() -> PathBuf {
    if PathBuf::from("D:\\codex_data").exists() {
        PathBuf::from("D:\\codex_data\\codex-launcher")
    } else {
        PathBuf::from("C:\\codex_data\\codex-launcher")
    }
}

#[cfg(target_os = "macos")]
fn default_ascii_codex_home() -> PathBuf {
    PathBuf::from("/tmp/codex-launcher")
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn default_ascii_codex_home() -> PathBuf {
    PathBuf::from("/tmp/codex-launcher")
}

fn ensure_ascii_home(path: PathBuf) -> PathBuf {
    if path.to_string_lossy().is_ascii() {
        path
    } else {
        default_ascii_codex_home()
    }
}

pub fn resolve_codex_home(input: Option<&str>) -> PathBuf {
    let base = input
        .filter(|v| !v.trim().is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("CODEX_HOME")
                .ok()
                .filter(|v| !v.trim().is_empty())
                .map(PathBuf::from)
        })
        .unwrap_or_else(default_ascii_codex_home);

    ensure_ascii_home(base)
}

pub fn resolve_profile_paths(codex_home: Option<&str>, profile_name: Option<&str>) -> ProfilePaths {
    let resolved_home = resolve_codex_home(codex_home);
    let profile_component = sanitize_ascii_component(profile_name.unwrap_or("default"));
    let profile_dir = resolved_home.join("profiles").join(profile_component);
    let config_path = profile_dir.join("config.toml");
    let auth_path = profile_dir.join("auth.json");
    let logs_dir = profile_dir.join("logs");

    ProfilePaths {
        codex_home: resolved_home,
        profile_dir,
        config_path,
        auth_path,
        logs_dir,
    }
}

pub fn ensure_profile_dirs(paths: &ProfilePaths) -> LauncherResult<()> {
    fs::create_dir_all(&paths.profile_dir)?;
    fs::create_dir_all(&paths.logs_dir)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::resolve_profile_paths;

    #[test]
    fn ascii_profile_path_logic_handles_non_ascii_input() {
        let paths = resolve_profile_paths(Some("D:\\测试目录\\codex"), Some("默认 配置🔥"));

        assert!(paths.codex_home.to_string_lossy().is_ascii());
        assert!(paths.profile_dir.to_string_lossy().is_ascii());
        assert!(!paths.profile_dir.to_string_lossy().contains("测试目录"));
        assert!(!paths.profile_dir.to_string_lossy().contains("默认"));
    }
}
