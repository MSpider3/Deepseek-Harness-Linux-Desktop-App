use crate::storage::UpdateHistoryRecord;
use crate::updater::{UpdateChannel, UpdateCheckResult, UpdateResult};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn check_for_updates(channel: Option<String>, state: State<'_, AppState>) -> Result<UpdateCheckResult, String> {
    let chan = match channel.as_deref() {
        Some("rc") => UpdateChannel::ReleaseCandidate,
        Some("dev") => UpdateChannel::Development,
        _ => UpdateChannel::Stable,
    };

    state.update_mgr.check_for_updates(chan).await
        .map_err(|e| format!("Failed to check for updates: {}", e))
}

#[tauri::command]
pub async fn apply_update(target_version: String, state: State<'_, AppState>) -> Result<UpdateResult, String> {
    state.update_mgr.apply_update(&target_version).await
        .map_err(|e| format!("Failed to apply update: {}", e))
}

#[tauri::command]
pub async fn rollback_runtime(state: State<'_, AppState>) -> Result<UpdateResult, String> {
    state.update_mgr.rollback()
        .map_err(|e| format!("Failed to roll back: {}", e))
}

#[tauri::command]
pub async fn get_update_history(state: State<'_, AppState>) -> Result<Vec<UpdateHistoryRecord>, String> {
    state.db.list_update_history()
        .map_err(|e| format!("Failed to get update history: {}", e))
}
