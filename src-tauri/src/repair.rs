use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use rusqlite::Connection;
use serde_json::Value;

use crate::error::LauncherResult;

const CANONICAL_PROVIDER: &str = "openai";
const SESSION_INDEX_FILE: &str = "session_index.jsonl";
const STATE_DB_FILE: &str = "state_5.sqlite";
const GLOBAL_STATE_FILE: &str = ".codex-global-state.json";

#[derive(Debug, Default, Clone, Copy)]
pub struct ProviderRepairReport {
    pub session_files_changed: u32,
    pub sqlite_rows_changed: u32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct HistoryBridgeReport {
    pub sources_scanned: u32,
    pub session_files_copied: u32,
    pub session_index_entries_added: u32,
    pub sqlite_rows_copied: u32,
    pub database_seeded: bool,
    pub global_state_seeded: bool,
}

pub fn normalize_openai_provider_labels(codex_home: &Path) -> LauncherResult<ProviderRepairReport> {
    let sessions_root = codex_home.join("sessions");
    let sqlite_path = codex_home.join(STATE_DB_FILE);

    Ok(ProviderRepairReport {
        session_files_changed: normalize_session_provider_labels(&sessions_root)?,
        sqlite_rows_changed: normalize_sqlite_provider_labels(&sqlite_path)?,
    })
}

pub fn bridge_history_from_default_sources(target_home: &Path) -> LauncherResult<HistoryBridgeReport> {
    let sources = default_history_sources(target_home);
    bridge_history_from_sources(target_home, &sources)
}

pub fn bridge_history_from_sources(
    target_home: &Path,
    source_homes: &[PathBuf],
) -> LauncherResult<HistoryBridgeReport> {
    let mut report = HistoryBridgeReport::default();
    fs::create_dir_all(target_home)?;

    for source_home in source_homes {
        if !source_home.exists() || same_path(source_home, target_home) {
            continue;
        }

        report.sources_scanned += 1;
        report.session_files_copied += copy_missing_session_files(source_home, target_home)?;
        report.session_index_entries_added +=
            merge_session_indexes(&source_home.join(SESSION_INDEX_FILE), &target_home.join(SESSION_INDEX_FILE))?;

        let sqlite_report = seed_or_merge_state_db(source_home, target_home)?;
        report.sqlite_rows_copied += sqlite_report.rows_copied;
        report.database_seeded |= sqlite_report.seeded;

        if seed_global_state_if_missing(source_home, target_home)? {
            report.global_state_seeded = true;
        }
    }

    Ok(report)
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

#[derive(Debug, Default, Clone, Copy)]
struct SqliteBridgeReport {
    rows_copied: u32,
    seeded: bool,
}

fn default_history_sources(target_home: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(codex_home) = std::env::var("CODEX_HOME") {
        candidates.push(PathBuf::from(codex_home));
    }

    #[cfg(target_os = "windows")]
    {
        candidates.push(PathBuf::from("D:\\codex_data"));
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            candidates.push(PathBuf::from(user_profile).join(".codex"));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(home) = std::env::var("HOME") {
            candidates.push(PathBuf::from(home).join(".codex"));
        }
    }

    let mut seen = HashSet::new();
    candidates
        .into_iter()
        .filter(|path| path.exists())
        .filter(|path| !same_path(path, target_home))
        .filter(|path| seen.insert(normalize_path_key(path)))
        .collect()
}

fn copy_missing_session_files(source_home: &Path, target_home: &Path) -> LauncherResult<u32> {
    let source_sessions = source_home.join("sessions");
    let target_sessions = target_home.join("sessions");
    if !source_sessions.exists() {
        return Ok(0);
    }

    let mut copied = 0;
    for entry in walkdir(&source_sessions)? {
        if entry.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
            continue;
        }

        let relative = match entry.strip_prefix(&source_sessions) {
            Ok(relative) => relative,
            Err(_) => continue,
        };
        let target_path = target_sessions.join(relative);
        if target_path.exists() {
            continue;
        }

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&entry, &target_path)?;
        copied += 1;
    }

    Ok(copied)
}

fn merge_session_indexes(source_index: &Path, target_index: &Path) -> LauncherResult<u32> {
    if !source_index.exists() {
        return Ok(0);
    }

    let mut merged = load_index_entries(target_index)?;
    let before = merged.len();

    for (id, entry) in load_index_entries(source_index)? {
        match merged.get(&id) {
            Some(existing) if existing == &entry => {}
            Some(existing) => {
                let existing_updated = existing
                    .get("updated_at")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let incoming_updated = entry
                    .get("updated_at")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                if incoming_updated >= existing_updated {
                    merged.insert(id, entry);
                }
            }
            None => {
                merged.insert(id, entry);
            }
        }
    }

    write_index_entries(target_index, &merged)?;
    Ok((merged.len().saturating_sub(before)) as u32)
}

fn load_index_entries(path: &Path) -> LauncherResult<BTreeMap<String, Value>> {
    let mut entries = BTreeMap::new();
    if !path.exists() {
        return Ok(entries);
    }

    for line in fs::read_to_string(path)?.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let entry: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let Some(id) = entry.get("id").and_then(Value::as_str) else {
            continue;
        };
        entries.insert(id.to_string(), entry);
    }

    Ok(entries)
}

fn write_index_entries(path: &Path, entries: &BTreeMap<String, Value>) -> LauncherResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut sorted = entries.values().cloned().collect::<Vec<_>>();
    sorted.sort_by(|left, right| {
        let left_updated = left
            .get("updated_at")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let right_updated = right
            .get("updated_at")
            .and_then(Value::as_str)
            .unwrap_or_default();
        left_updated
            .cmp(right_updated)
            .then_with(|| {
                left.get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .cmp(right.get("id").and_then(Value::as_str).unwrap_or_default())
            })
    });

    let payload = if sorted.is_empty() {
        String::new()
    } else {
        let lines = sorted
            .into_iter()
            .map(|entry| serde_json::to_string(&entry))
            .collect::<Result<Vec<_>, _>>()?;
        format!("{}\n", lines.join("\n"))
    };
    fs::write(path, payload)?;
    Ok(())
}

fn seed_or_merge_state_db(source_home: &Path, target_home: &Path) -> LauncherResult<SqliteBridgeReport> {
    let source_db = source_home.join(STATE_DB_FILE);
    let target_db = target_home.join(STATE_DB_FILE);

    if !source_db.exists() {
        return Ok(SqliteBridgeReport::default());
    }

    if !target_db.exists() {
        fs::copy(source_db, target_db)?;
        let seeded_rows = count_threads(&target_home.join(STATE_DB_FILE)).unwrap_or(0);
        return Ok(SqliteBridgeReport {
            rows_copied: seeded_rows,
            seeded: true,
        });
    }

    merge_threads_between_databases(&source_db, &target_db)
}

fn merge_threads_between_databases(source_db: &Path, target_db: &Path) -> LauncherResult<SqliteBridgeReport> {
    let source = Connection::open(source_db)?;
    let target = Connection::open(target_db)?;

    let source_columns = thread_columns(&source)?;
    let target_columns = thread_columns(&target)?;
    if source_columns.is_empty() || target_columns.is_empty() {
        return Ok(SqliteBridgeReport::default());
    }

    let common_columns = source_columns
        .iter()
        .filter(|column| target_columns.contains(column))
        .cloned()
        .collect::<Vec<_>>();
    if common_columns.is_empty() {
        return Ok(SqliteBridgeReport::default());
    }

    let select_sql = format!("SELECT {} FROM threads", common_columns.join(", "));
    let id_index = common_columns
        .iter()
        .position(|column| column == "id")
        .unwrap_or(0);

    let mut stmt = source.prepare(&select_sql)?;
    let rows = stmt.query_map([], |row| {
        let mut values = Vec::with_capacity(common_columns.len());
        for index in 0..common_columns.len() {
            values.push(row.get::<usize, rusqlite::types::Value>(index)?);
        }
        Ok(values)
    })?;

    let placeholders = vec!["?"; common_columns.len()].join(", ");
    let insert_sql = format!(
        "INSERT INTO threads ({}) VALUES ({})",
        common_columns.join(", "),
        placeholders
    );

    let tx = target.unchecked_transaction()?;
    let mut inserted = 0u32;
    for row in rows {
        let values = row?;
        let thread_id = match &values[id_index] {
            rusqlite::types::Value::Text(id) => id.clone(),
            _ => continue,
        };

        let exists = tx.query_row(
            "SELECT 1 FROM threads WHERE id = ?1 LIMIT 1",
            [&thread_id],
            |_| Ok(()),
        );

        if exists.is_ok() {
            continue;
        }

        tx.execute(&insert_sql, rusqlite::params_from_iter(values.into_iter()))?;
        inserted += 1;
    }
    tx.commit()?;

    Ok(SqliteBridgeReport {
        rows_copied: inserted,
        seeded: false,
    })
}

fn thread_columns(connection: &Connection) -> LauncherResult<Vec<String>> {
    let mut stmt = connection.prepare("PRAGMA table_info(threads)")?;
    let rows = stmt.query_map([], |row| row.get::<usize, String>(1))?;
    let mut columns = Vec::new();
    for row in rows {
        columns.push(row?);
    }
    Ok(columns)
}

fn count_threads(path: &Path) -> LauncherResult<u32> {
    let connection = Connection::open(path)?;
    let count: i64 = connection.query_row("SELECT COUNT(*) FROM threads", [], |row| row.get(0))?;
    Ok(count as u32)
}

fn seed_global_state_if_missing(source_home: &Path, target_home: &Path) -> LauncherResult<bool> {
    let source = source_home.join(GLOBAL_STATE_FILE);
    let target = target_home.join(GLOBAL_STATE_FILE);
    if !source.exists() || target.exists() {
        return Ok(false);
    }

    fs::copy(source, target)?;
    Ok(true)
}

fn same_path(left: &Path, right: &Path) -> bool {
    normalize_path_key(left) == normalize_path_key(right)
}

fn normalize_path_key(path: &Path) -> String {
    let text = path.to_string_lossy();
    if cfg!(target_os = "windows") {
        text.to_lowercase()
    } else {
        text.to_string()
    }
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
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use rusqlite::Connection;

    use super::{bridge_history_from_sources, is_openai_alias, SESSION_INDEX_FILE, STATE_DB_FILE};

    #[test]
    fn openai_alias_check_is_case_insensitive() {
        assert!(is_openai_alias("OpenAI"));
        assert!(is_openai_alias("openai"));
        assert!(!is_openai_alias("anthropic"));
    }

    #[test]
    fn history_bridge_copies_sessions_index_and_state_db() {
        let root = make_temp_dir("bridge");
        let source = root.join("source");
        let target = root.join("target");
        fs::create_dir_all(source.join("sessions/2026/04/14")).expect("create source sessions");

        let session_file = source
            .join("sessions/2026/04/14/rollout-2026-04-14T00-00-00-test.jsonl");
        fs::write(
            &session_file,
            "{\"type\":\"session_meta\",\"payload\":{\"id\":\"thread-1\",\"model_provider\":\"OpenAI\"}}\n",
        )
        .expect("write session");
        fs::write(
            source.join(SESSION_INDEX_FILE),
            "{\"id\":\"thread-1\",\"thread_name\":\"hello\",\"updated_at\":\"2026-04-14T00:00:00Z\"}\n",
        )
        .expect("write index");

        let source_db = source.join(STATE_DB_FILE);
        create_threads_db(&source_db, "thread-1", "OpenAI").expect("create source db");

        let report = bridge_history_from_sources(&target, &[source.clone()]).expect("bridge should succeed");

        assert_eq!(report.sources_scanned, 1);
        assert_eq!(report.session_files_copied, 1);
        assert_eq!(report.session_index_entries_added, 1);
        assert!(report.database_seeded);
        assert!(target
            .join("sessions/2026/04/14/rollout-2026-04-14T00-00-00-test.jsonl")
            .exists());
        assert!(target.join(SESSION_INDEX_FILE).exists());
        assert!(target.join(STATE_DB_FILE).exists());
    }

    fn make_temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time works")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("codex-launcher-{label}-{unique}"));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create temp root");
        path
    }

    fn create_threads_db(path: &Path, thread_id: &str, provider: &str) -> rusqlite::Result<()> {
        let connection = Connection::open(path)?;
        connection.execute_batch(
            "
            CREATE TABLE threads (
                id TEXT PRIMARY KEY,
                rollout_path TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                source TEXT NOT NULL,
                model_provider TEXT NOT NULL,
                cwd TEXT NOT NULL,
                title TEXT NOT NULL,
                sandbox_policy TEXT NOT NULL,
                approval_mode TEXT NOT NULL,
                tokens_used INTEGER NOT NULL DEFAULT 0,
                has_user_event INTEGER NOT NULL DEFAULT 0,
                archived INTEGER NOT NULL DEFAULT 0,
                archived_at INTEGER,
                git_sha TEXT,
                git_branch TEXT,
                git_origin_url TEXT,
                cli_version TEXT NOT NULL DEFAULT '',
                first_user_message TEXT NOT NULL DEFAULT '',
                agent_nickname TEXT,
                agent_role TEXT,
                memory_mode TEXT NOT NULL DEFAULT 'enabled',
                model TEXT,
                reasoning_effort TEXT,
                agent_path TEXT
            );
            ",
        )?;
        connection.execute(
            "
            INSERT INTO threads (
                id, rollout_path, created_at, updated_at, source, model_provider, cwd, title,
                sandbox_policy, approval_mode, tokens_used, has_user_event, archived, archived_at,
                git_sha, git_branch, git_origin_url, cli_version, first_user_message, agent_nickname,
                agent_role, memory_mode, model, reasoning_effort, agent_path
            ) VALUES (
                ?1, '/tmp/rollout', 1, 2, 'desktop', ?2, 'D:/workspace', 'hello',
                'danger-full-access', 'never', 0, 0, 0, NULL,
                NULL, NULL, NULL, '', '', NULL,
                NULL, 'enabled', NULL, NULL, NULL
            )
            ",
            [thread_id, provider],
        )?;
        Ok(())
    }
}
