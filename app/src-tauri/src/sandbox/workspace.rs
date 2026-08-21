use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    Node,
    Python,
    Rust,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    pub workspace_id: String,
    pub original_path: String,
    pub workspace_path: String,
    pub project_type: ProjectType,
    pub default_test_command: String,
    pub created_at: String,
}

pub struct SandboxWorkspace;

impl SandboxWorkspace {
    /// Detects the project type of a given directory.
    pub fn detect_project_type(project_dir: &Path) -> (ProjectType, String) {
        if project_dir.join("Cargo.toml").exists() {
            (ProjectType::Rust, "cargo test".to_string())
        } else if project_dir.join("package.json").exists() {
            (ProjectType::Node, "npm test".to_string())
        } else if project_dir.join("pyproject.toml").exists() || project_dir.join("pytest.ini").exists() || project_dir.join("requirements.txt").exists() {
            (ProjectType::Python, "pytest".to_string())
        } else {
            (ProjectType::General, "echo 'No test command defined'".to_string())
        }
    }

    /// Creates an isolated temporary workspace for safe modification and testing.
    pub fn create_staging_workspace(
        sandbox_base_dir: &Path,
        original_project_dir: &Path,
    ) -> Result<WorkspaceMetadata> {
        let workspace_id = format!("ws_{}", Uuid::new_v4());
        let workspace_dir = sandbox_base_dir.join("workspaces").join(&workspace_id);
        fs::create_dir_all(&workspace_dir)
            .with_context(|| format!("Failed to create workspace dir {:?}", workspace_dir))?;

        let (project_type, default_test_command) = Self::detect_project_type(original_project_dir);

        // Copy files excluding heavyweight or environment-specific dirs
        Self::copy_project_tree(original_project_dir, &workspace_dir)?;

        // Node projects: if original has node_modules, create symlink for fast testing
        if project_type == ProjectType::Node {
            let orig_nm = original_project_dir.join("node_modules");
            let ws_nm = workspace_dir.join("node_modules");
            if orig_nm.exists() && !ws_nm.exists() {
                let _ = std::os::unix::fs::symlink(&orig_nm, &ws_nm);
            }
        }

        // Rust projects: symlink target cache if safe
        if project_type == ProjectType::Rust {
            let orig_target = original_project_dir.join("target");
            let ws_target = workspace_dir.join("target");
            if orig_target.exists() && !ws_target.exists() {
                let _ = std::os::unix::fs::symlink(&orig_target, &ws_target);
            }
        }

        Ok(WorkspaceMetadata {
            workspace_id,
            original_path: original_project_dir.to_string_lossy().to_string(),
            workspace_path: workspace_dir.to_string_lossy().to_string(),
            project_type,
            default_test_command,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    fn copy_project_tree(src: &Path, dst: &Path) -> Result<()> {
        if !src.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(src)?.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip directories and sensitive files
            if name == ".git"
                || name == "node_modules"
                || name == "target"
                || name == ".venv"
                || name == "venv"
                || name == "__pycache__"
                || name == ".env"
                || name == ".credentials.yaml"
            {
                continue;
            }

            let target = dst.join(&name);
            if path.is_dir() {
                fs::create_dir_all(&target)?;
                Self::copy_project_tree(&path, &target)?;
            } else if path.is_file() {
                fs::copy(&path, &target)?;
            }
        }

        Ok(())
    }

    /// Discards and removes a staging workspace.
    pub fn discard_workspace(workspace_dir: &Path) -> Result<()> {
        if workspace_dir.exists() {
            fs::remove_dir_all(workspace_dir)?;
        }
        Ok(())
    }
}
