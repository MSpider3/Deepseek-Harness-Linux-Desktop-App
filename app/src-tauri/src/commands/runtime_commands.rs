use crate::runtime::{RuntimeInfo, RuntimeInstaller};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_runtime_info(state: State<'_, AppState>) -> Result<RuntimeInfo, String> {
    state.runtime_mgr.get_runtime_info()
        .map_err(|e| format!("Failed to get runtime info: {}", e))
}

#[tauri::command]
pub async fn install_runtime(version_or_tag: Option<String>, state: State<'_, AppState>) -> Result<String, String> {
    let tag = version_or_tag.unwrap_or_else(|| "latest".to_string());
    let staging_dir = state.runtime_mgr.staging_dir().join(format!("install_{}", uuid::Uuid::new_v4()));

    let installed_ver = RuntimeInstaller::install_package(&staging_dir, &tag)
        .map_err(|e| format!("Installation failed: {}", e))?;

    let final_dir = state.runtime_mgr.versions_dir().join(&installed_ver);
    if final_dir.exists() {
        let _ = std::fs::remove_dir_all(&final_dir);
    }

    std::fs::rename(&staging_dir, &final_dir)
        .map_err(|e| format!("Failed to move runtime to final directory: {}", e))?;

    state.runtime_mgr.activate_version(&installed_ver)
        .map_err(|e| format!("Failed to activate runtime: {}", e))?;

    let _ = state.db.record_update(None, &installed_ver, "success", None);

    Ok(installed_ver)
}

#[tauri::command]
pub async fn activate_runtime_version(version: String, state: State<'_, AppState>) -> Result<(), String> {
    state.runtime_mgr.activate_version(&version)
        .map_err(|e| format!("Failed to activate version: {}", e))
}
