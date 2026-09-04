use std::path::Path;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApexConfig {
    #[serde(default)]
    pub provider: ProviderConfig,
    #[serde(default)]
    pub models: ModelPoolConfig,
    #[serde(default)]
    pub agent: AgentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    #[serde(default = "default_provider_type")]
    pub provider_type: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    pub api_key: Option<String>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            provider_type: default_provider_type(),
            base_url: default_base_url(),
            api_key: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPoolConfig {
    #[serde(default = "default_primary_model")]
    pub primary: String,
    #[serde(default = "default_fallback_pool")]
    pub fallback_pool: Vec<String>,
    #[serde(default = "default_fast_model")]
    pub fast_tier: String,
    #[serde(default = "default_true")]
    pub auto_fallback: bool,
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,
}

impl Default for ModelPoolConfig {
    fn default() -> Self {
        Self {
            primary: default_primary_model(),
            fallback_pool: default_fallback_pool(),
            fast_tier: default_fast_model(),
            auto_fallback: true,
            max_retries: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(default = "default_workspace_dir")]
    pub workspace_dir: String,
    #[serde(default = "default_max_steps")]
    pub max_steps: usize,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            workspace_dir: default_workspace_dir(),
            max_steps: default_max_steps(),
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
        }
    }
}

fn default_provider_type() -> String { "omniroute".to_string() }
fn default_base_url() -> String { "http://localhost:20128/v1".to_string() }
fn default_primary_model() -> String { "openrouter/qwen/qwen-2.5-coder-32b-instruct:free".to_string() }
fn default_fast_model() -> String { "openrouter/google/gemini-2.0-flash-exp:free".to_string() }
fn default_fallback_pool() -> Vec<String> {
    vec![
        "openrouter/qwen/qwen-2.5-coder-32b-instruct:free".to_string(),
        "openrouter/deepseek/deepseek-r1:free".to_string(),
        "openrouter/meta-llama/llama-3.3-70b-instruct:free".to_string(),
        "openrouter/google/gemini-2.0-flash-exp:free".to_string(),
        "google/gemini-2.0-flash".to_string(),
        "openrouter/deepseek/deepseek-r1".to_string(),
        "openrouter/qwen/qwen-2.5-coder-32b-instruct".to_string(),
    ]
}
fn default_true() -> bool { true }
fn default_max_retries() -> usize { 3 }
fn default_workspace_dir() -> String { ".".to_string() }
fn default_max_steps() -> usize { 30 }
fn default_temperature() -> f32 { 0.2 }
fn default_max_tokens() -> usize { 4096 }

impl Default for ApexConfig {
    fn default() -> Self {
        Self {
            provider: ProviderConfig::default(),
            models: ModelPoolConfig::default(),
            agent: AgentConfig::default(),
        }
    }
}

impl ApexConfig {
    pub fn load() -> Result<Self> {
        let mut config = Self::default();

        if let Some(config_dir) = dirs::config_dir() {
            let global_file = config_dir.join("apex").join("config.toml");
            if global_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&global_file) {
                    if let Ok(parsed) = toml::from_str::<ApexConfig>(&content) {
                        config = parsed;
                    }
                }
            }
        }

        let local_file = Path::new(".apex").join("config.toml");
        if local_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&local_file) {
                if let Ok(parsed) = toml::from_str::<ApexConfig>(&content) {
                    config = parsed;
                }
            }
        }

        if let Ok(url) = std::env::var("OMNIROUTE_BASE_URL") {
            config.provider.base_url = url;
        } else if let Ok(url) = std::env::var("APEX_BASE_URL") {
            config.provider.base_url = url;
        }

        if let Ok(key) = std::env::var("OMNIROUTE_API_KEY") {
            config.provider.api_key = Some(key);
        } else if let Ok(key) = std::env::var("APEX_API_KEY") {
            config.provider.api_key = Some(key);
        } else if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
            config.provider.api_key = Some(key);
        }

        if let Ok(model) = std::env::var("APEX_MODEL") {
            config.models.primary = model;
        }

        Ok(config)
    }

    pub fn get_api_key(&self) -> Option<String> {
        if let Some(ref k) = self.provider.api_key {
            if !k.trim().is_empty() {
                return Some(k.clone());
            }
        }

        // For OmniRoute and local gateway proxies, provide default fallback authorization
        if self.provider.provider_type.eq_ignore_ascii_case("omniroute")
            || self.provider.base_url.contains("localhost")
            || self.provider.base_url.contains("127.0.0.1")
        {
            Some("omniroute".to_string())
        } else {
            None
        }
    }

    pub fn save_default(path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let config = ApexConfig::default();
        let toml_str = toml::to_string_pretty(&config)?;
        std::fs::write(path, toml_str)?;
        Ok(())
    }
}
