use thiserror::Error;

pub type LauncherResult<T> = Result<T, LauncherError>;

#[derive(Debug, Error)]
pub enum LauncherError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("TOML serialization error: {0}")]
    Toml(#[from] toml::ser::Error),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Command failed: {0}")]
    CommandFailed(String),
}
