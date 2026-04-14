use std::{collections::BTreeMap, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::error::LauncherResult;

pub const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub base_url: String,
    pub wire_api: String,
    pub requires_openai_auth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexConfig {
    pub preferred_auth_method: String,
    pub model_provider: String,
    pub model_providers: BTreeMap<String, ProviderConfig>,
}

impl CodexConfig {
    pub fn openai_compatible(base_url: &str) -> Self {
        let mut model_providers = BTreeMap::new();
        model_providers.insert(
            "openai".to_string(),
            ProviderConfig {
                name: "OpenAI".to_string(),
                base_url: base_url.to_string(),
                wire_api: "responses".to_string(),
                requires_openai_auth: true,
            },
        );

        Self {
            preferred_auth_method: "apikey".to_string(),
            model_provider: "openai".to_string(),
            model_providers,
        }
    }
}

pub fn generate_codex_config_toml(base_url: Option<&str>) -> LauncherResult<String> {
    let resolved_base_url = base_url
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(DEFAULT_OPENAI_BASE_URL);
    let config = CodexConfig::openai_compatible(resolved_base_url);
    Ok(toml::to_string_pretty(&config)?)
}

pub fn write_codex_config(path: &Path, base_url: Option<&str>) -> LauncherResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let config_toml = generate_codex_config_toml(base_url)?;
    fs::write(path, config_toml)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::generate_codex_config_toml;

    #[test]
    fn config_generation_contains_required_openai_compat_fields() {
        let text = generate_codex_config_toml(Some("https://proxy.example.com/v1"))
            .expect("config generation should succeed");

        assert!(text.contains("preferred_auth_method = \"apikey\""));
        assert!(text.contains("model_provider = \"openai\""));
        assert!(text.contains("[model_providers.openai]"));
        assert!(text.contains("name = \"OpenAI\""));
        assert!(text.contains("base_url = \"https://proxy.example.com/v1\""));
        assert!(text.contains("wire_api = \"responses\""));
        assert!(text.contains("requires_openai_auth = true"));
    }
}
