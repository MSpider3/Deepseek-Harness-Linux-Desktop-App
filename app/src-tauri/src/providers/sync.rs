use crate::security::SecretStore;
use crate::storage::Database;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub struct ProviderConfigSyncer;

impl ProviderConfigSyncer {
    /// Synchronizes database providers and secret store into $DSH_HOME configuration files and environment variable map.
    /// Files written:
    /// - $DSH_HOME/.credentials.yaml (0600 mode) - read by @deepseek-ai/dsh-credentials-local
    /// - $DSH_HOME/settings.yaml - read by @deepseek-ai/dsh-settings-file (llm-deepseek and llm-pi-ai namespaces)
    /// - $DSH_HOME/.env (0600 mode) - machine-level defaults read by DSH launch environment
    /// - $DSH_HOME/cordis.patch.yml - fallback layer
    pub fn sync(
        dsh_home: &Path,
        db: &Database,
        secrets: &SecretStore,
    ) -> Result<HashMap<String, String>> {
        fs::create_dir_all(dsh_home)
            .with_context(|| format!("Failed to create DSH_HOME dir {:?}", dsh_home))?;

        #[cfg(unix)]
        if let Ok(metadata) = fs::metadata(dsh_home) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o700);
            let _ = fs::set_permissions(dsh_home, perms);
        }

        let providers = db.list_providers()?;
        let mut child_env = HashMap::new();
        let mut credentials_map = serde_yaml::Mapping::new();
        let mut pi_ai_providers = serde_json::Map::new();
        let mut deepseek_base_url = "https://api.deepseek.com".to_string();

        for p in &providers {
            let is_deepseek = p.provider_type == "deepseek"
                || p.name.to_lowercase().contains("deepseek");

            let name_env_var = format!("{}_API_KEY", p.name.to_uppercase().replace([' ', '-'], "_"));
            let standard_env_var = match p.provider_type.as_str() {
                "deepseek" => "DEEPSEEK_API_KEY".to_string(),
                "openai" => "OPENAI_API_KEY".to_string(),
                "anthropic" => "ANTHROPIC_API_KEY".to_string(),
                "openrouter" => "OPENROUTER_API_KEY".to_string(),
                "groq" => "GROQ_API_KEY".to_string(),
                "ollama" => "OLLAMA_API_KEY".to_string(),
                _ => name_env_var.clone(),
            };

            let route_key = if is_deepseek {
                deepseek_base_url = p.base_url.clone();
                "deepseek".to_string()
            } else {
                match p.provider_type.as_str() {
                    "openai" => "openai".to_string(),
                    "anthropic" => "anthropic".to_string(),
                    "openrouter" => "openrouter".to_string(),
                    "groq" => "groq".to_string(),
                    "ollama" => "ollama".to_string(),
                    _ => p.name.to_lowercase().replace(' ', "-"),
                }
            };

            // If secret exists, store in child env and credentials map
            if let Some(ref sec_ref) = p.secret_ref {
                if let Ok(Some(secret_val)) = secrets.get_secret(sec_ref) {
                    if !secret_val.trim().is_empty() {
                        child_env.insert(standard_env_var.clone(), secret_val.clone());
                        child_env.insert(name_env_var.clone(), secret_val.clone());
                        credentials_map.insert(
                            serde_yaml::Value::String(standard_env_var.clone()),
                            serde_yaml::Value::String(secret_val.clone()),
                        );
                        credentials_map.insert(
                            serde_yaml::Value::String(name_env_var.clone()),
                            serde_yaml::Value::String(secret_val.clone()),
                        );
                        if is_deepseek {
                            child_env.insert("DEEPSEEK_API_KEY".to_string(), secret_val.clone());
                            credentials_map.insert(
                                serde_yaml::Value::String("DEEPSEEK_API_KEY".to_string()),
                                serde_yaml::Value::String(secret_val),
                            );
                        }
                    }
                }
            }

            let models = db.list_models(&p.id)?;
            let mut model_list = Vec::new();
            for m in models {
                let mut m_obj = serde_json::json!({
                    "id": m.model_id,
                    "name": m.display_name,
                });
                if let Some(ctx) = m.context_window {
                    m_obj["contextWindow"] = serde_json::json!(ctx);
                }
                if let Some(max_tok) = m.max_tokens {
                    m_obj["maxTokens"] = serde_json::json!(max_tok);
                }
                if m.supports_reasoning {
                    m_obj["reasoningEfforts"] = serde_json::json!({
                        "off": null,
                        "low": "low",
                        "medium": "medium",
                        "high": "high"
                    });
                }
                model_list.push(m_obj);
            }

            let mut prov_obj = serde_json::json!({
                "apiKeyEnv": standard_env_var,
                "baseURL": p.base_url,
                "displayName": p.name,
            });

            if !model_list.is_empty() {
                prov_obj["models"] = serde_json::Value::Array(model_list);
            }

            pi_ai_providers.insert(route_key, prov_obj);
        }

        // 1. Write $DSH_HOME/.credentials.yaml with mode 0600
        let creds_yaml_str = serde_yaml::to_string(&credentials_map)
            .context("Failed to serialize .credentials.yaml")?;
        let cred_file = dsh_home.join(".credentials.yaml");
        fs::write(&cred_file, &creds_yaml_str)
            .with_context(|| format!("Failed to write .credentials.yaml to {:?}", cred_file))?;

        #[cfg(unix)]
        {
            if let Ok(file) = fs::File::open(&cred_file) {
                if let Ok(metadata) = file.metadata() {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o600);
                    let _ = fs::set_permissions(&cred_file, perms);
                }
            }
        }

        // 2. Read and merge into $DSH_HOME/settings.yaml
        let settings_file = dsh_home.join("settings.yaml");
        let mut settings_doc: serde_yaml::Mapping = if settings_file.exists() {
            let content = fs::read_to_string(&settings_file).unwrap_or_default();
            serde_yaml::from_str(&content).unwrap_or_default()
        } else {
            serde_yaml::Mapping::new()
        };

        // Update llm-deepseek section
        let deepseek_settings = serde_json::json!({
            "apiKeyEnv": "DEEPSEEK_API_KEY",
            "baseURL": deepseek_base_url,
        });
        if let Ok(ds_yaml_val) = serde_yaml::to_value(&deepseek_settings) {
            settings_doc.insert(
                serde_yaml::Value::String("llm-deepseek".to_string()),
                ds_yaml_val,
            );
        }

        // Update llm-pi-ai section
        let pi_ai_settings = serde_json::json!({
            "providers": pi_ai_providers,
        });
        if let Ok(pi_yaml_val) = serde_yaml::to_value(&pi_ai_settings) {
            settings_doc.insert(
                serde_yaml::Value::String("llm-pi-ai".to_string()),
                pi_yaml_val,
            );
        }

        let settings_yaml_str = serde_yaml::to_string(&settings_doc)
            .context("Failed to serialize settings.yaml")?;
        fs::write(&settings_file, &settings_yaml_str)
            .with_context(|| format!("Failed to write settings.yaml to {:?}", settings_file))?;

        #[cfg(unix)]
        {
            if let Ok(file) = fs::File::open(&settings_file) {
                if let Ok(metadata) = file.metadata() {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o600);
                    let _ = fs::set_permissions(&settings_file, perms);
                }
            }
        }

        // 3. Write $DSH_HOME/.env
        let mut env_lines = Vec::new();
        for (k, v) in &child_env {
            env_lines.push(format!("{}={}", k, v));
        }
        let env_content = env_lines.join("\n");
        let env_file = dsh_home.join(".env");
        let _ = fs::write(&env_file, &env_content);
        #[cfg(unix)]
        {
            if let Ok(file) = fs::File::open(&env_file) {
                if let Ok(metadata) = file.metadata() {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o600);
                    let _ = fs::set_permissions(&env_file, perms);
                }
            }
        }

        // 4. Write cordis.patch.yml
        let patch_doc = serde_json::json!([
            {
                "id": "llm",
                "name": "@deepseek-ai/dsh-llm-pi-ai",
                "config": {
                    "providers": pi_ai_providers
                }
            }
        ]);
        let patch_yaml_str = serde_yaml::to_string(&patch_doc)
            .context("Failed to serialize cordis.patch.yml")?;
        let patch_file = dsh_home.join("cordis.patch.yml");
        let _ = fs::write(&patch_file, patch_yaml_str);

        // 5. Fallback sync to standard ~/.dsh if different from dsh_home
        if let Some(user_home) = dirs::home_dir() {
            let default_dsh = user_home.join(".dsh");
            if default_dsh != dsh_home {
                let _ = fs::create_dir_all(&default_dsh);
                #[cfg(unix)]
                if let Ok(metadata) = fs::metadata(&default_dsh) {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o700);
                    let _ = fs::set_permissions(&default_dsh, perms);
                }

                let default_cred = default_dsh.join(".credentials.yaml");
                let _ = fs::write(&default_cred, &creds_yaml_str);
                #[cfg(unix)]
                if let Ok(file) = fs::File::open(&default_cred) {
                    if let Ok(metadata) = file.metadata() {
                        let mut perms = metadata.permissions();
                        perms.set_mode(0o600);
                        let _ = fs::set_permissions(&default_cred, perms);
                    }
                }

                let default_settings = default_dsh.join("settings.yaml");
                let _ = fs::write(&default_settings, &settings_yaml_str);
                #[cfg(unix)]
                if let Ok(file) = fs::File::open(&default_settings) {
                    if let Ok(metadata) = file.metadata() {
                        let mut perms = metadata.permissions();
                        perms.set_mode(0o600);
                        let _ = fs::set_permissions(&default_settings, perms);
                    }
                }
            }
        }

        Ok(child_env)
    }
}
