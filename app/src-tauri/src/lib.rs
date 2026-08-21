pub mod commands;
pub mod dsh;
pub mod linux;
pub mod projects;
pub mod providers;
pub mod runtime;
pub mod sandbox;
pub mod security;
pub mod storage;
pub mod updater;

use dsh::{DshHealthChecker, DshProcessManager};
use linux::create_tray;
use projects::SnapshotManager;
use providers::ProviderManager;
use runtime::RuntimeManager;
use security::SecretStore;
use storage::Database;
use updater::UpdateManager;

use std::path::PathBuf;
use tauri::Manager;

pub struct AppState {
    pub db: Database,
    pub secrets: SecretStore,
    pub runtime_mgr: RuntimeManager,
    pub process_mgr: DshProcessManager,
    pub update_mgr: UpdateManager,
    pub provider_mgr: ProviderManager,
    pub snapshot_mgr: SnapshotManager,
    pub health_checker: DshHealthChecker,
    pub base_data_dir: PathBuf,
}

pub fn run() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let base_app_dir = home.join(".local").join("share").join("deepseek-harness-linux");
    let data_dir = base_app_dir.join("data");
    let runtime_dir = base_app_dir.join("runtime");
    let sandbox_dir = base_app_dir.join("sandbox");

    let _ = std::fs::create_dir_all(&data_dir);
    let _ = std::fs::create_dir_all(&runtime_dir);
    let _ = std::fs::create_dir_all(&sandbox_dir);

    let db = Database::init(&data_dir).expect("Failed to initialize SQLite database");
    let secrets = SecretStore::init(&data_dir).expect("Failed to initialize secret store");
    let runtime_mgr = RuntimeManager::new(&runtime_dir);
    let process_mgr = DshProcessManager::new(&data_dir);
    let update_mgr = UpdateManager::new(runtime_mgr.clone(), db.clone());
    let provider_mgr = ProviderManager::new(db.clone(), secrets.clone());
    let snapshot_mgr = SnapshotManager::new(&sandbox_dir, db.clone());
    let health_checker = DshHealthChecker::new();

    let state = AppState {
        db,
        secrets,
        runtime_mgr,
        process_mgr,
        update_mgr,
        provider_mgr,
        snapshot_mgr,
        health_checker,
        base_data_dir: data_dir,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .manage(state)
        .setup(|app| {
            let handle = app.handle();
            let _ = create_tray(handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::dsh_commands::get_dsh_status,
            commands::dsh_commands::get_dsh_logs,
            commands::dsh_commands::start_dsh,
            commands::dsh_commands::stop_dsh,
            commands::dsh_commands::restart_dsh,
            commands::dsh_commands::check_dsh_health,
            commands::runtime_commands::get_runtime_info,
            commands::runtime_commands::install_runtime,
            commands::runtime_commands::activate_runtime_version,
            commands::updater_commands::check_for_updates,
            commands::updater_commands::apply_update,
            commands::updater_commands::rollback_runtime,
            commands::updater_commands::get_update_history,
            commands::provider_commands::list_providers,
            commands::provider_commands::save_provider,
            commands::provider_commands::delete_provider,
            commands::provider_commands::list_provider_models,
            commands::provider_commands::save_provider_models,
            commands::provider_commands::test_provider_connection,
            commands::provider_commands::discover_models,
            commands::provider_commands::has_provider_secret,
            commands::sandbox_commands::create_sandbox_workspace,
            commands::sandbox_commands::run_sandbox_tests,
            commands::sandbox_commands::get_sandbox_diff,
            commands::sandbox_commands::apply_sandbox_changes,
            commands::sandbox_commands::discard_sandbox,
            commands::project_commands::get_git_status,
            commands::project_commands::create_snapshot,
            commands::project_commands::list_snapshots,
            commands::project_commands::restore_snapshot,
            commands::project_commands::delete_snapshot,
            commands::settings_commands::get_setting,
            commands::settings_commands::set_setting,
            commands::settings_commands::install_desktop_launcher,
            commands::settings_commands::export_diagnostics,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                // Ensure child process is killed on exit
                let state: tauri::State<'_, AppState> = app_handle.state();
                let process_mgr = &state.process_mgr;
                let _ = tauri::async_runtime::block_on(process_mgr.stop());
            }
        });
}
