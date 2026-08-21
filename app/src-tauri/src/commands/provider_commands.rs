use crate::providers::{DiscoveredModel, ProviderConfigSyncer, TestConnectionResult};
use crate::storage::{ModelRecord, ProviderRecord};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_providers(state: State<'_, AppState>) -> Result<Vec<ProviderRecord>, String> {
    state.db.list_providers()
        .map_err(|e| format!("Failed to list providers: {}", e))
}

#[tauri::command]
pub async fn save_provider(
    provider: ProviderRecord,
    secret_value: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if let (Some(sec_ref), Some(sec_val)) = (provider.secret_ref.as_deref(), secret_value.as_deref()) {
        if !sec_val.trim().is_empty() {
            state.secrets.set_secret(sec_ref, sec_val)
                .map_err(|e| format!("Failed to save secret: {}", e))?;
        }
    }

    state.db.save_provider(&provider)
        .map_err(|e| format!("Failed to save provider: {}", e))?;

    // Instantly sync into DSH_HOME .credentials.yaml, settings.yaml, and .env
    let dsh_home = state.runtime_mgr.dsh_home_dir();
    let _ = ProviderConfigSyncer::sync(&dsh_home, &state.db, &state.secrets);

    Ok(())
}

#[tauri::command]
pub async fn delete_provider(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.db.delete_provider(&id)
        .map_err(|e| format!("Failed to delete provider: {}", e))?;

    // Instantly sync into DSH_HOME .credentials.yaml, settings.yaml, and .env
    let dsh_home = state.runtime_mgr.dsh_home_dir();
    let _ = ProviderConfigSyncer::sync(&dsh_home, &state.db, &state.secrets);

    Ok(())
}

#[tauri::command]
pub async fn list_provider_models(provider_id: String, state: State<'_, AppState>) -> Result<Vec<ModelRecord>, String> {
    state.db.list_models(&provider_id)
        .map_err(|e| format!("Failed to list models: {}", e))
}

#[tauri::command]
pub async fn save_provider_models(
    provider_id: String,
    models: Vec<ModelRecord>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.db.replace_models(&provider_id, &models)
        .map_err(|e| format!("Failed to save models: {}", e))?;

    // Instantly sync into DSH_HOME .credentials.yaml, settings.yaml, and .env
    let dsh_home = state.runtime_mgr.dsh_home_dir();
    let _ = ProviderConfigSyncer::sync(&dsh_home, &state.db, &state.secrets);

    Ok(())
}

#[tauri::command]
pub async fn test_provider_connection(
    provider_type: String,
    base_url: String,
    api_key: Option<String>,
    state: State<'_, AppState>,
) -> Result<TestConnectionResult, String> {
    Ok(state.provider_mgr.test_connection(&provider_type, &base_url, api_key.as_deref()).await)
}

#[tauri::command]
pub async fn discover_models(
    provider_type: String,
    base_url: String,
    api_key: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<DiscoveredModel>, String> {
    state.provider_mgr.discover_models(&provider_type, &base_url, api_key.as_deref()).await
        .map_err(|e| format!("Failed to discover models: {}", e))
}

#[tauri::command]
pub async fn has_provider_secret(secret_ref: String, state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.secrets.has_secret(&secret_ref))
}
