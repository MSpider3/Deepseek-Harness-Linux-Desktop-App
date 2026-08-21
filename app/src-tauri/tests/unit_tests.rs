use deepseek_harness_linux_lib::projects::GitInspector;
use deepseek_harness_linux_lib::providers::ProviderConfigSyncer;
use deepseek_harness_linux_lib::runtime::RuntimeManager;
use deepseek_harness_linux_lib::sandbox::{ChangeApplier, DiffGenerator, ProjectType, SandboxWorkspace};
use deepseek_harness_linux_lib::security::{Redactor, SecretStore};
use deepseek_harness_linux_lib::storage::{Database, ProviderRecord};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_secret_redaction() {
    let raw = "Authorization: Bearer sk-ant-api03-abcdef1234567890xyz and secret=superSecret12345";
    let sanitized = Redactor::sanitize(raw);
    assert!(!sanitized.contains("sk-ant-api03-abcdef1234567890xyz"));
    assert!(!sanitized.contains("superSecret12345"));
    assert!(sanitized.contains("••••••••"));
}

#[test]
fn test_secret_store_encrypted_vault() {
    let dir = tempdir().unwrap();
    let store = SecretStore::init(dir.path()).unwrap();

    store.set_secret("openai_key", "sk-proj-test99999").unwrap();
    assert_eq!(store.get_secret("openai_key").unwrap().as_deref(), Some("sk-proj-test99999"));
    assert!(store.has_secret("openai_key"));

    // Reload from disk to verify persistence and AES decryption
    let reloaded_store = SecretStore::init(dir.path()).unwrap();
    assert_eq!(reloaded_store.get_secret("openai_key").unwrap().as_deref(), Some("sk-proj-test99999"));

    // Delete
    assert!(reloaded_store.delete_secret("openai_key").unwrap());
    assert!(!reloaded_store.has_secret("openai_key"));
}

#[test]
fn test_runtime_manager_symlinks_and_rollback() {
    let dir = tempdir().unwrap();
    let mgr = RuntimeManager::new(dir.path());
    mgr.ensure_directories().unwrap();

    let ver1_dir = mgr.versions_dir().join("0.1.0-rc.7");
    let ver2_dir = mgr.versions_dir().join("0.1.0-rc.8");
    fs::create_dir_all(&ver1_dir).unwrap();
    fs::create_dir_all(&ver2_dir).unwrap();

    // Activate v1
    mgr.activate_version("0.1.0-rc.7").unwrap();
    let info1 = mgr.get_runtime_info().unwrap();
    assert_eq!(info1.current_version.as_deref(), Some("0.1.0-rc.7"));
    assert_eq!(info1.previous_version, None);

    // Activate v2 (v1 becomes previous)
    mgr.activate_version("0.1.0-rc.8").unwrap();
    let info2 = mgr.get_runtime_info().unwrap();
    assert_eq!(info2.current_version.as_deref(), Some("0.1.0-rc.8"));
    assert_eq!(info2.previous_version.as_deref(), Some("0.1.0-rc.7"));

    // Rollback to v1
    let rolled = mgr.rollback().unwrap();
    assert_eq!(rolled, "0.1.0-rc.7");
    let info3 = mgr.get_runtime_info().unwrap();
    assert_eq!(info3.current_version.as_deref(), Some("0.1.0-rc.7"));
}

#[test]
fn test_diff_and_change_applier() {
    let orig_dir = tempdir().unwrap();
    let ws_dir = tempdir().unwrap();

    let f1 = orig_dir.path().join("index.ts");
    let f2 = orig_dir.path().join("deleted.ts");
    fs::write(&f1, "const a = 1;\n").unwrap();
    fs::write(&f2, "to delete\n").unwrap();

    // Copy to ws
    let ws_f1 = ws_dir.path().join("index.ts");
    let ws_f3 = ws_dir.path().join("new_file.ts");
    fs::write(&ws_f1, "const a = 2;\n").unwrap();
    fs::write(&ws_f3, "export const brand = 'DeepSeek';\n").unwrap();

    let diffs = DiffGenerator::compute_directory_diff(orig_dir.path(), ws_dir.path()).unwrap();
    assert_eq!(diffs.len(), 3);

    let count = ChangeApplier::apply_changes(ws_dir.path(), orig_dir.path()).unwrap();
    assert_eq!(count, 3);

    assert_eq!(fs::read_to_string(&f1).unwrap(), "const a = 2;\n");
    assert!(orig_dir.path().join("new_file.ts").exists());
    assert!(!f2.exists());
}

#[test]
fn test_sandbox_project_detection() {
    let dir = tempdir().unwrap();
    let node_dir = dir.path().join("node_proj");
    fs::create_dir_all(&node_dir).unwrap();
    fs::write(node_dir.join("package.json"), "{}").unwrap();

    let (p_type, cmd) = SandboxWorkspace::detect_project_type(&node_dir);
    assert_eq!(p_type, ProjectType::Node);
    assert_eq!(cmd, "npm test");

    let rust_dir = dir.path().join("rust_proj");
    fs::create_dir_all(&rust_dir).unwrap();
    fs::write(rust_dir.join("Cargo.toml"), "[package]").unwrap();

    let (r_type, r_cmd) = SandboxWorkspace::detect_project_type(&rust_dir);
    assert_eq!(r_type, ProjectType::Rust);
    assert_eq!(r_cmd, "cargo test");
}

#[test]
fn test_provider_syncer() {
    let dir = tempdir().unwrap();
    let dsh_home = dir.path().join("dsh-home");
    let db = Database::in_memory().unwrap();
    let secrets = SecretStore::in_memory();

    let prov = ProviderRecord {
        id: "p1".to_string(),
        name: "TestDeepSeek".to_string(),
        provider_type: "deepseek".to_string(),
        base_url: "https://api.deepseek.com".to_string(),
        secret_ref: Some("sec_ds_1".to_string()),
        is_default: true,
        compat_mode: None,
        created_at: "2026-08-20T00:00:00Z".to_string(),
        updated_at: "2026-08-20T00:00:00Z".to_string(),
    };

    db.save_provider(&prov).unwrap();
    secrets.set_secret("sec_ds_1", "sk-mock-key-12345").unwrap();

    let env_map = ProviderConfigSyncer::sync(&dsh_home, &db, &secrets).unwrap();
    assert_eq!(env_map.get("TESTDEEPSEEK_API_KEY").map(|s| s.as_str()), Some("sk-mock-key-12345"));

    let patch_file = dsh_home.join("cordis.patch.yml");
    assert!(patch_file.exists());
    let patch_content = fs::read_to_string(patch_file).unwrap();
    assert!(patch_content.contains("@deepseek-ai/dsh-llm-pi-ai"));
    assert!(patch_content.contains("https://api.deepseek.com"));
}

#[test]
fn test_git_inspector() {
    let dir = tempdir().unwrap();
    let info = GitInspector::inspect_project(dir.path()).unwrap();
    assert!(!info.is_git_repo);
    assert!(info.is_clean);
}
