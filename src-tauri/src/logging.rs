use std::{fs, path::Path};

use crate::{
    error::LauncherResult,
    models::OpenLogsResponse,
    profile::{ensure_profile_dirs, resolve_profile_paths},
};

fn build_open_logs_command(logs_dir: &Path) -> Vec<String> {
    let logs_dir_text = logs_dir.to_string_lossy().to_string();
    if cfg!(target_os = "windows") {
        vec!["explorer.exe".to_string(), logs_dir_text]
    } else if cfg!(target_os = "macos") {
        vec!["open".to_string(), logs_dir_text]
    } else {
        vec!["xdg-open".to_string(), logs_dir_text]
    }
}

fn spawn_open_logs(command: &[String]) -> LauncherResult<()> {
    let mut process = std::process::Command::new(&command[0]);
    for arg in &command[1..] {
        process.arg(arg);
    }
    process.spawn()?;
    Ok(())
}

pub fn open_logs(codex_home: Option<&str>, profile_name: Option<&str>) -> LauncherResult<OpenLogsResponse> {
    let paths = resolve_profile_paths(codex_home, profile_name);
    ensure_profile_dirs(&paths)?;
    fs::create_dir_all(&paths.logs_dir)?;

    let open_command = build_open_logs_command(&paths.logs_dir);
    spawn_open_logs(&open_command)?;

    Ok(OpenLogsResponse {
        success: true,
        logs_dir: paths.logs_dir.to_string_lossy().to_string(),
        open_command,
        status: "ok".to_string(),
        message: "Logs directory is ready and open command was dispatched.".to_string(),
    })
}
