use crate::{
    installer, launcher, logging,
    models::{EnvironmentReport, InstallOrRepairResponse, LaunchRequest, LaunchResponse, OpenLogsResponse},
    platform,
};

#[tauri::command]
pub fn detect_environment() -> Result<EnvironmentReport, String> {
    platform::detect_environment().map_err(|err| err.to_string())
}

#[tauri::command]
pub fn install_or_repair_codex() -> Result<InstallOrRepairResponse, String> {
    installer::install_or_repair_codex().map_err(|err| err.to_string())
}

#[tauri::command]
pub fn save_and_launch(request: LaunchRequest) -> Result<LaunchResponse, String> {
    launcher::save_and_launch(request).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn open_logs(codex_home: Option<String>, profile_name: Option<String>) -> Result<OpenLogsResponse, String> {
    logging::open_logs(codex_home.as_deref(), profile_name.as_deref()).map_err(|err| err.to_string())
}
