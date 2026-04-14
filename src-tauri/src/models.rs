use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformKind {
    Windows,
    MacOS,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherSettings {
    pub codex_home: String,
    pub profile_name: String,
    pub preferred_auth_method: String,
    pub model_provider: String,
    pub openai_base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentReport {
    pub status: String,
    pub summary: String,
    pub details: Vec<String>,
    pub platform: PlatformKind,
    pub codex_installed: bool,
    pub codex_version: Option<String>,
    pub wsl_check_command: Option<Vec<String>>,
    pub wsl_available: bool,
    pub macos_terminal_command_example: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRequest {
    pub codex_home: Option<String>,
    pub profile_name: Option<String>,
    pub openai_api_key: String,
    pub openai_base_url: Option<String>,
    #[serde(default)]
    pub extra_args: Vec<String>,
    #[serde(default)]
    pub launch_now: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchResponse {
    pub success: bool,
    pub started: bool,
    pub command: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub config_path: String,
    pub auth_path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallOrRepairResponse {
    pub success: bool,
    pub codex_installed: bool,
    pub codex_version: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenLogsResponse {
    pub success: bool,
    pub logs_dir: String,
    pub open_command: Vec<String>,
    pub status: String,
    pub message: String,
}
