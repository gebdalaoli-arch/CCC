use std::process::Command;

use regex::Regex;

use crate::{
    error::LauncherResult,
    models::{InstallOrRepairResponse, PlatformKind},
    platform,
};

pub fn parse_codex_version(output: &str) -> Option<String> {
    let regex = Regex::new(r"(?i)\b(v?\d+\.\d+\.\d+(?:[-+][0-9A-Za-z\.-]+)?)\b")
        .expect("version regex should compile");
    regex
        .captures(output)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().trim_start_matches('v').to_string()))
}

pub fn probe_codex_version() -> LauncherResult<Option<String>> {
    let output = Command::new("codex").arg("--version").output();
    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);
            let merged = format!("{stdout}\n{stderr}");
            Ok(parse_codex_version(&merged))
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.into()),
    }
}

pub fn install_or_repair_codex() -> LauncherResult<InstallOrRepairResponse> {
    let version = probe_codex_version()?;
    if let Some(version) = version {
        return Ok(InstallOrRepairResponse {
            success: true,
            codex_installed: true,
            codex_version: Some(version),
            message: "Detected existing codex installation.".to_string(),
        });
    }

    let (success, message, installed_version) = match platform::detect_platform() {
        PlatformKind::Windows if platform::is_wsl_available() => {
            run_install_command(
                "wsl.exe",
                &[
                    "bash",
                    "-lc",
                    "if command -v codex >/dev/null 2>&1; then codex --version; elif command -v npm >/dev/null 2>&1; then npm install -g @openai/codex && codex --version; else echo 'npm is required inside WSL' >&2; exit 10; fi",
                ],
            )?
        }
        _ => run_install_command("npm", &["install", "-g", "@openai/codex"])?,
    };

    Ok(InstallOrRepairResponse {
        success,
        codex_installed: installed_version.is_some(),
        codex_version: installed_version,
        message,
    })
}

fn run_install_command(program: &str, args: &[&str]) -> LauncherResult<(bool, String, Option<String>)> {
    let output = Command::new(program).args(args).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let merged = format!("{stdout}\n{stderr}");
    let version = parse_codex_version(&merged);

    if output.status.success() {
        Ok((
            true,
            if version.is_some() {
                "Codex installation or repair completed successfully.".to_string()
            } else {
                "Install command succeeded, but codex version was not parsed from output.".to_string()
            },
            version,
        ))
    } else {
        Ok((
            false,
            format!("Install command failed: {}", merged.trim()),
            version,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::parse_codex_version;

    #[test]
    fn parse_codex_version_extracts_semver() {
        let parsed = parse_codex_version("Codex CLI 1.2.3 (build abc)");
        assert_eq!(parsed.as_deref(), Some("1.2.3"));
    }
}
