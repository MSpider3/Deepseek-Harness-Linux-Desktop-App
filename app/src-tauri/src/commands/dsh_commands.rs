use crate::dsh::{DshProcessStatus, HealthStatus, ProcessLogEntry};
use crate::providers::ProviderConfigSyncer;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_dsh_status(state: State<'_, AppState>) -> Result<DshProcessStatus, String> {
    Ok(state.process_mgr.status())
}

#[tauri::command]
pub async fn get_dsh_logs(limit: Option<usize>, state: State<'_, AppState>) -> Result<Vec<ProcessLogEntry>, String> {
    Ok(state.process_mgr.get_logs(limit.unwrap_or(200)))
}

#[tauri::command]
pub async fn start_dsh(port: Option<u16>, state: State<'_, AppState>) -> Result<String, String> {
    let exec_path = state.runtime_mgr.get_executable_path()
        .map_err(|e| format!("Runtime not ready: {}", e))?;

    let dsh_home = state.runtime_mgr.dsh_home_dir();
    
    // Sync provider configuration into DSH_HOME cordis.patch.yml & collect env
    let child_env = ProviderConfigSyncer::sync(&dsh_home, &state.db, &state.secrets)
        .map_err(|e| format!("Failed to synchronize provider configurations: {}", e))?;

    state.process_mgr.start(&exec_path, &dsh_home, child_env, port).await
        .map_err(|e| format!("Failed to start DSH: {}", e))
}

#[tauri::command]
pub async fn stop_dsh(state: State<'_, AppState>) -> Result<(), String> {
    state.process_mgr.stop().await
        .map_err(|e| format!("Failed to stop DSH: {}", e))
}

#[tauri::command]
pub async fn restart_dsh(port: Option<u16>, state: State<'_, AppState>) -> Result<String, String> {
    let exec_path = state.runtime_mgr.get_executable_path()
        .map_err(|e| format!("Runtime not ready: {}", e))?;

    let dsh_home = state.runtime_mgr.dsh_home_dir();
    let child_env = ProviderConfigSyncer::sync(&dsh_home, &state.db, &state.secrets)
        .map_err(|e| format!("Failed to synchronize provider configurations: {}", e))?;

    state.process_mgr.restart(&exec_path, &dsh_home, child_env, port).await
        .map_err(|e| format!("Failed to restart DSH: {}", e))
}

#[tauri::command]
pub async fn check_dsh_health(port: u16, state: State<'_, AppState>) -> Result<HealthStatus, String> {
    Ok(state.health_checker.check_web_port(port).await)
}
