use crate::security::SecretStore;
use crate::storage::{Database, ModelRecord, ProviderRecord};
use anyhow::{bail, Context, Result};
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredModel {
    pub id: String,
    pub name: String,
    pub context_window: Option<i64>,
    pub max_tokens: Option<i64>,
    pub supports_reasoning: bool,
    pub supports_vision: bool,
    pub supports_tools: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConnectionResult {
    pub success: bool,
    pub latency_ms: Option<u64>,
    pub message: String,
}

pub struct ProviderManager {
    db: Database,
    #[allow(dead_code)]
    secrets: SecretStore,
    client: Client,
}

impl ProviderManager {
    pub fn new(db: Database, secrets: SecretStore) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());

        let mgr = Self {
            db,
            secrets,
            client,
        };

        let _ = mgr.seed_default_presets();
        mgr
    }

    /// Seeds initial provider presets if no providers are registered.
    pub fn seed_default_presets(&self) -> Result<()> {
        let existing = self.db.list_providers()?;
        if !existing.is_empty() {
            return Ok(());
        }

        // DeepSeek Default Preset
        let deepseek = ProviderRecord {
            id: Uuid::new_v4().to_string(),
            name: "DeepSeek".to_string(),
            provider_type: "deepseek".to_string(),
            base_url: "https://api.deepseek.com".to_string(),
            secret_ref: Some(format!("dsh_secret_{}", Uuid::new_v4())),
            is_default: true,
            compat_mode: None,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };
        self.db.save_provider(&deepseek)?;

        let ds_models = vec![
            ModelRecord {
                id: Uuid::new_v4().to_string(),
                provider_id: deepseek.id.clone(),
                model_id: "deepseek-chat".to_string(),
                display_name: "DeepSeek V3 (Chat)".to_string(),
                context_window: Some(65536),
                max_tokens: Some(8192),
                supports_reasoning: false,
                supports_vision: false,
                supports_tools: true,
                discovered_at: Utc::now().to_rfc3339(),
            },
            ModelRecord {
                id: Uuid::new_v4().to_string(),
                provider_id: deepseek.id.clone(),
                model_id: "deepseek-reasoner".to_string(),
                display_name: "DeepSeek R1 (Reasoner)".to_string(),
                context_window: Some(65536),
                max_tokens: Some(8192),
                supports_reasoning: true,
                supports_vision: false,
                supports_tools: true,
                discovered_at: Utc::now().to_rfc3339(),
            },
        ];
        self.db.replace_models(&deepseek.id, &ds_models)?;

        // OpenAI Preset
        let openai = ProviderRecord {
            id: Uuid::new_v4().to_string(),
            name: "OpenAI".to_string(),
            provider_type: "openai".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            secret_ref: Some(format!("dsh_secret_{}", Uuid::new_v4())),
            is_default: false,
            compat_mode: None,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };
        self.db.save_provider(&openai)?;

        let oai_models = vec![
            ModelRecord {
                id: Uuid::new_v4().to_string(),
                provider_id: openai.id.clone(),
                model_id: "gpt-4o".to_string(),
                display_name: "GPT-4o".to_string(),
                context_window: Some(128000),
                max_tokens: Some(16384),
                supports_reasoning: false,
                supports_vision: true,
                supports_tools: true,
                discovered_at: Utc::now().to_rfc3339(),
            },
            ModelRecord {
                id: Uuid::new_v4().to_string(),
                provider_id: openai.id.clone(),
                model_id: "o3-mini".to_string(),
                display_name: "o3-mini (Reasoning)".to_string(),
                context_window: Some(200000),
                max_tokens: Some(100000),
                supports_reasoning: true,
                supports_vision: false,
                supports_tools: true,
                discovered_at: Utc::now().to_rfc3339(),
            },
        ];
        self.db.replace_models(&openai.id, &oai_models)?;

        // Anthropic Claude Preset
        let claude = ProviderRecord {
            id: Uuid::new_v4().to_string(),
            name: "Anthropic Claude".to_string(),
            provider_type: "anthropic".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
            secret_ref: Some(format!("dsh_secret_{}", Uuid::new_v4())),
            is_default: false,
            compat_mode: None,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };
        self.db.save_provider(&claude)?;

        let claude_models = vec![
            ModelRecord {
                id: Uuid::new_v4().to_string(),
                provider_id: claude.id.clone(),
                model_id: "claude-3-7-sonnet-20250219".to_string(),
                display_name: "Claude 3.7 Sonnet (Hybrid Thinking)".to_string(),
                context_window: Some(200000),
                max_tokens: Some(64000),
                supports_reasoning: true,
                supports_vision: true,
                supports_tools: true,
                discovered_at: Utc::now().to_rfc3339(),
            },
        ];
        self.db.replace_models(&claude.id, &claude_models)?;

        // Local Ollama Preset
        let ollama = ProviderRecord {
            id: Uuid::new_v4().to_string(),
            name: "Local Ollama".to_string(),
            provider_type: "ollama".to_string(),
            base_url: "http://localhost:11434/v1".to_string(),
            secret_ref: None,
            is_default: false,
            compat_mode: Some("openai-compatible".to_string()),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };
        self.db.save_provider(&ollama)?;

        Ok(())
    }

    /// Tests connection and credentials against the provider API.
    pub async fn test_connection(
        &self,
        provider_type: &str,
        base_url: &str,
        api_key: Option<&str>,
    ) -> TestConnectionResult {
        let start = std::time::Instant::now();
        let trimmed_url = base_url.trim_end_matches('/');

        let test_url = match provider_type {
            "ollama" => format!("{}/models", trimmed_url),
            "anthropic" => format!("{}/v1/messages", trimmed_url),
            _ => {
                if trimmed_url.ends_with("/v1") {
                    format!("{}/models", trimmed_url)
                } else {
                    format!("{}/v1/models", trimmed_url)
                }
            }
        };

        let mut req = self.client.get(&test_url);
        if let Some(key) = api_key {
            if !key.trim().is_empty() {
                if provider_type == "anthropic" {
                    req = req.header("x-api-key", key.trim())
                             .header("anthropic-version", "2023-06-01");
                } else {
                    req = req.header("Authorization", format!("Bearer {}", key.trim()));
                }
            }
        }

        match req.send().await {
            Ok(resp) => {
                let latency = start.elapsed().as_millis() as u64;
                let status = resp.status();
                if status.is_success() || status.as_u16() == 400 || status.as_u16() == 404 {
                    // Anthropic returns 400 on GET /messages but validates auth
                    TestConnectionResult {
                        success: true,
                        latency_ms: Some(latency),
                        message: format!("Connection successful (HTTP {}, {}ms)", status.as_u16(), latency),
                    }
                } else if status.as_u16() == 401 || status.as_u16() == 403 {
                    TestConnectionResult {
                        success: false,
                        latency_ms: Some(latency),
                        message: format!("Authentication failed (HTTP {}): Invalid API key or permission denied", status.as_u16()),
                    }
                } else {
                    TestConnectionResult {
                        success: false,
                        latency_ms: Some(latency),
                        message: format!("Provider endpoint returned HTTP {}", status.as_u16()),
                    }
                }
            }
            Err(e) => TestConnectionResult {
                success: false,
                latency_ms: None,
                message: format!("Connection failed: {}", e),
            },
        }
    }

    /// Discovers available models from the provider endpoint.
    pub async fn discover_models(
        &self,
        provider_type: &str,
        base_url: &str,
        api_key: Option<&str>,
    ) -> Result<Vec<DiscoveredModel>> {
        let trimmed_url = base_url.trim_end_matches('/');

        if provider_type == "deepseek" {
            return Ok(vec![
                DiscoveredModel {
                    id: "deepseek-chat".to_string(),
                    name: "DeepSeek V3 (Chat)".to_string(),
                    context_window: Some(65536),
                    max_tokens: Some(8192),
                    supports_reasoning: false,
                    supports_vision: false,
                    supports_tools: true,
                },
                DiscoveredModel {
                    id: "deepseek-reasoner".to_string(),
                    name: "DeepSeek R1 (Reasoner)".to_string(),
                    context_window: Some(65536),
                    max_tokens: Some(8192),
                    supports_reasoning: true,
                    supports_vision: false,
                    supports_tools: true,
                },
            ]);
        }

        let models_url = if trimmed_url.ends_with("/v1") || provider_type == "ollama" {
            format!("{}/models", trimmed_url)
        } else {
            format!("{}/v1/models", trimmed_url)
        };

        let mut req = self.client.get(&models_url);
        if let Some(key) = api_key {
            if !key.trim().is_empty() {
                req = req.header("Authorization", format!("Bearer {}", key.trim()));
            }
        }

        let resp = req.send().await.context("Failed to query models endpoint")?;
        if !resp.status().is_success() {
            bail!("Models endpoint returned HTTP {}", resp.status());
        }

        let body: serde_json::Value = resp.json().await.context("Failed to parse models JSON response")?;

        let mut list = Vec::new();
        if let Some(data_array) = body.get("data").and_then(|d| d.as_array()) {
            for item in data_array {
                if let Some(id) = item.get("id").and_then(|i| i.as_str()) {
                    let name = item.get("name").and_then(|n| n.as_str()).unwrap_or(id).to_string();
                    let is_reasoning = id.contains("r1") || id.contains("o1") || id.contains("o3") || id.contains("reason");
                    let is_vision = id.contains("vision") || id.contains("4o") || id.contains("gemini") || id.contains("claude");
                    let ctx = item.get("context_length").and_then(|c| c.as_i64()).or(Some(128000));

                    list.push(DiscoveredModel {
                        id: id.to_string(),
                        name,
                        context_window: ctx,
                        max_tokens: Some(8192),
                        supports_reasoning: is_reasoning,
                        supports_vision: is_vision,
                        supports_tools: true,
                    });
                }
            }
        }

        Ok(list)
    }
}
