use std::{fs, path::Path};

use rusqlite::Connection;
use serde_json::Value;

use crate::error::LauncherResult;

const CANONICAL_PROVIDER: &str = "openai";

#[derive(Debug, Default, Clone, Copy)]
pub struct ProviderRepairReport {
    pub session_files_changed: u32,
    pub sqlite_rows_changed: u32,
}

pub fn normalize_openai_provider_labels(codex_home: &Path) -> LauncherResult<ProviderRepairReport> {
    let sessions_root = codex_home.join("sessions");
    let sqlite_path = codex_home.join("state_5.sqlite");

    Ok(ProviderRepairReport {
        session_files_changed: normalize_session_provider_labels(&sessions_root)?,
        sqlite_rows_changed: normalize_sqlite_provider_labels(&sqlite_path)?,
    })
}

fn normalize_session_provider_labels(sessions_root: &Path) -> LauncherResult<u32> {
    if !sessions_root.exists() {
        return Ok(0);
    }

    let mut changed = 0;
    for entry in walkdir(sessions_root)? {
        if entry.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
            continue;
        }

        let raw = fs::read_to_string(&entry)?;
        let mut lines: Vec<String> = raw.lines().map(ToString::to_string).collect();
        if lines.is_empty() {
            continue;
        }

        let mut first: Value = match serde_json::from_str(&lines[0]) {
            Ok(value) => value,
            Err(_) => continue,
        };

        let payload = match first.get_mut("payload").and_then(Value::as_object_mut) {
            Some(payload) => payload,
            None => continue,
        };

        let provider = payload
            .get("model_provider")
            .and_then(Value::as_str)
            .unwrap_or_default();

        if !is_openai_alias(provider) || provider == CANONICAL_PROVIDER {
            continue;
        }

        payload.insert(
            "model_provider".to_string(),
            Value::String(CANONICAL_PROVIDER.to_string()),
        );
        lines[0] = serde_json::to_string(&first)?;
        let newline = if raw.ends_with('\n') { "\n" } else { "" };
        fs::write(&entry, format!("{}{}", lines.join("\n"), newline))?;
        changed += 1;
    }

    Ok(changed)
}

fn normalize_sqlite_provider_labels(sqlite_path: &Path) -> LauncherResult<u32> {
    if !sqlite_path.exists() {
        return Ok(0);
    }

    let connection = Connection::open(sqlite_path)?;
    let changed = connection.execute(
        "UPDATE threads SET model_provider = ?1 WHERE LOWER(model_provider) = LOWER(?2) AND model_provider <> ?1",
        [CANONICAL_PROVIDER, CANONICAL_PROVIDER],
    )?;

    Ok(changed as u32)
}

fn is_openai_alias(provider: &str) -> bool {
    provider.eq_ignore_ascii_case(CANONICAL_PROVIDER)
}

fn walkdir(root: &Path) -> LauncherResult<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(path);
            }
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::is_openai_alias;

    #[test]
    fn openai_alias_check_is_case_insensitive() {
        assert!(is_openai_alias("OpenAI"));
        assert!(is_openai_alias("openai"));
        assert!(!is_openai_alias("anthropic"));
    }
}
