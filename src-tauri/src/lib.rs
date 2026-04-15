pub mod auth;
pub mod commands;
pub mod error;
pub mod installer;
pub mod launcher;
pub mod logging;
pub mod models;
pub mod offline;
pub mod platform;
pub mod profile;
pub mod repair;
pub mod settings;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::detect_environment,
            commands::install_or_repair_codex,
            commands::save_and_launch,
            commands::open_logs
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Codex Launcher backend");
}
