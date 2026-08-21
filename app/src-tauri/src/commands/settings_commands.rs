use crate::linux::DesktopEntryManager;
use crate::AppState;
use std::fs::File;
use std::io::Write;
use tauri::State;

#[tauri::command]
pub async fn get_setting(key: String, state: State<'_, AppState>) -> Result<Option<String>, String> {
    state.db.get_setting(&key)
        .map_err(|e| format!("Failed to get setting: {}", e))
}

#[tauri::command]
pub async fn set_setting(key: String, value: String, state: State<'_, AppState>) -> Result<(), String> {
    state.db.set_setting(&key, &value)
        .map_err(|e| format!("Failed to set setting: {}", e))
}

#[tauri::command]
pub async fn install_desktop_launcher() -> Result<String, String> {
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to determine executable path: {}", e))?;

    let icon_path = "ai.deepseek.harness.linux";
    let target = DesktopEntryManager::install_desktop_entry(&current_exe.to_string_lossy(), icon_path)
        .map_err(|e| format!("Failed to install desktop launcher: {}", e))?;

    Ok(target.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn export_diagnostics(target_path: String, state: State<'_, AppState>) -> Result<(), String> {
    let runtime_info = state.runtime_mgr.get_runtime_info().ok();
    let dsh_status = state.process_mgr.status();
    let logs = state.process_mgr.get_logs(300);
    let providers = state.db.list_providers().ok();
    let update_history = state.db.list_update_history().ok();

    let diag = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "runtime_info": runtime_info,
        "dsh_status": dsh_status,
        "providers_count": providers.as_ref().map(|p| p.len()),
        "providers_names": providers.map(|p| p.into_iter().map(|item| item.name).collect::<Vec<_>>()),
        "recent_logs": logs,
        "update_history": update_history,
    });

    let mut file = File::create(&target_path)
        .map_err(|e| format!("Failed to create diagnostic file at {}: {}", target_path, e))?;

    let json_str = serde_json::to_string_pretty(&diag)
        .map_err(|e| format!("Failed to format json: {}", e))?;

    file.write_all(json_str.as_bytes())
        .map_err(|e| format!("Failed to write diagnostic file: {}", e))?;

    Ok(())
}
