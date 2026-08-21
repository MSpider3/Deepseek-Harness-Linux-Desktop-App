use crate::sandbox::workspace::WorkspaceMetadata;
use crate::sandbox::{ChangeApplier, DiffFileSummary, DiffGenerator, SandboxWorkspace, TestResult, TestRunner};
use crate::AppState;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub async fn create_sandbox_workspace(
    project_path: String,
    state: State<'_, AppState>,
) -> Result<WorkspaceMetadata, String> {
    let orig_dir = PathBuf::from(&project_path);
    let sandbox_dir = state.base_data_dir.parent().unwrap_or(&state.base_data_dir).join("sandbox");

    SandboxWorkspace::create_staging_workspace(&sandbox_dir, &orig_dir)
        .map_err(|e| format!("Failed to create sandbox workspace: {}", e))
}

#[tauri::command]
pub async fn run_sandbox_tests(
    workspace_path: String,
    test_command: String,
) -> Result<TestResult, String> {
    let ws_dir = PathBuf::from(&workspace_path);
    TestRunner::run_test(&ws_dir, &test_command).await
        .map_err(|e| format!("Failed to run test command: {}", e))
}

#[tauri::command]
pub async fn get_sandbox_diff(
    original_path: String,
    workspace_path: String,
) -> Result<Vec<DiffFileSummary>, String> {
    let orig_dir = PathBuf::from(&original_path);
    let ws_dir = PathBuf::from(&workspace_path);

    DiffGenerator::compute_directory_diff(&orig_dir, &ws_dir)
        .map_err(|e| format!("Failed to compute diff: {}", e))
}

#[tauri::command]
pub async fn apply_sandbox_changes(
    original_path: String,
    workspace_path: String,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let orig_dir = PathBuf::from(&original_path);
    let ws_dir = PathBuf::from(&workspace_path);

    // Create safety snapshot first
    let _ = state.snapshot_mgr.create_snapshot(
        "current",
        &orig_dir,
        "Pre-apply automatic safety checkpoint",
        Some("Created automatically before applying sandbox changes"),
        None,
    );

    ChangeApplier::apply_changes(&ws_dir, &orig_dir)
        .map_err(|e| format!("Failed to apply changes: {}", e))
}

#[tauri::command]
pub async fn discard_sandbox(workspace_path: String) -> Result<(), String> {
    let ws_dir = PathBuf::from(&workspace_path);
    SandboxWorkspace::discard_workspace(&ws_dir)
        .map_err(|e| format!("Failed to discard workspace: {}", e))
}
