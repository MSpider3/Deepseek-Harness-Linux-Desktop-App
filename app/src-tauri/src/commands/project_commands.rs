use crate::projects::{GitInspector, GitStatusInfo};
use crate::storage::SnapshotRecord;
use crate::AppState;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub async fn get_git_status(project_path: String) -> Result<GitStatusInfo, String> {
    let p = PathBuf::from(&project_path);
    GitInspector::inspect_project(&p)
        .map_err(|e| format!("Failed to inspect git repo: {}", e))
}

#[tauri::command]
pub async fn create_snapshot(
    project_id: String,
    project_path: String,
    title: String,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<SnapshotRecord, String> {
    let p = PathBuf::from(&project_path);
    let git_info = GitInspector::inspect_project(&p).ok();
    let commit = git_info.and_then(|g| g.head_commit);

    state.snapshot_mgr.create_snapshot(&project_id, &p, &title, description.as_deref(), commit.as_deref())
        .map_err(|e| format!("Failed to create snapshot: {}", e))
}

#[tauri::command]
pub async fn list_snapshots(project_id: String, state: State<'_, AppState>) -> Result<Vec<SnapshotRecord>, String> {
    state.db.list_snapshots(&project_id)
        .map_err(|e| format!("Failed to list snapshots: {}", e))
}

#[tauri::command]
pub async fn restore_snapshot(
    snapshot_id: String,
    project_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let p = PathBuf::from(&project_path);
    state.snapshot_mgr.restore_snapshot(&snapshot_id, &p)
        .map_err(|e| format!("Failed to restore snapshot: {}", e))
}

#[tauri::command]
pub async fn delete_snapshot(snapshot_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.snapshot_mgr.delete_snapshot(&snapshot_id)
        .map_err(|e| format!("Failed to delete snapshot: {}", e))
}
