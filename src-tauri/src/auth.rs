use std::{fs, path::Path};

use serde_json::{json, Value};

use crate::error::{LauncherError, LauncherResult};

pub fn scrub_auth_json(existing_json: Option<&str>, explicit_api_key: Option<&str>) -> LauncherResult<String> {
    let mut api_key = explicit_api_key
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToString::to_string);

    if api_key.is_none() {
        if let Some(raw) = existing_json {
            if !raw.trim().is_empty() {
                let value: Value = serde_json::from_str(raw)?;
                api_key = value
                    .get("OPENAI_API_KEY")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
            }
        }
    }

    let final_key =
        api_key.ok_or_else(|| LauncherError::Validation("OPENAI_API_KEY is required".to_string()))?;
    let minimal = json!({
        "OPENAI_API_KEY": final_key
    });
    Ok(serde_json::to_string_pretty(&minimal)?)
}

pub fn write_minimal_auth(path: &Path, existing_json: Option<&str>, explicit_api_key: Option<&str>) -> LauncherResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let scrubbed = scrub_auth_json(existing_json, explicit_api_key)?;
    fs::write(path, scrubbed)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::scrub_auth_json;

    #[test]
    fn auth_scrub_keeps_only_openai_api_key() {
        let existing = r#"{
            "OPENAI_API_KEY": "sk-existing",
            "tokens": {
              "access_token": "abc",
              "refresh_token": "def"
            },
            "other": "value"
        }"#;

        let output = scrub_auth_json(Some(existing), None).expect("scrub should succeed");
        let value: Value = serde_json::from_str(&output).expect("json should be valid");

        assert_eq!(value.get("OPENAI_API_KEY").and_then(Value::as_str), Some("sk-existing"));
        assert_eq!(value.as_object().map(|o| o.len()), Some(1));
        assert!(value.get("tokens").is_none());
    }
}
