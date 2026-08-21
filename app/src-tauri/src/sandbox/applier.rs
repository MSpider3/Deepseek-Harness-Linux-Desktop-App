use crate::sandbox::diff::DiffGenerator;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

pub struct ChangeApplier;

impl ChangeApplier {
    /// Applies approved changes from the sandbox workspace back into the original project directory.
    pub fn apply_changes(workspace_dir: &Path, original_project_dir: &Path) -> Result<usize> {
        if !workspace_dir.exists() {
            bail!("Sandbox workspace does not exist at {:?}", workspace_dir);
        }
        if !original_project_dir.exists() {
            bail!("Target project directory does not exist at {:?}", original_project_dir);
        }

        let diff_summaries = DiffGenerator::compute_directory_diff(original_project_dir, workspace_dir)?;
        let mut applied_count = 0;

        for diff in &diff_summaries {
            let orig_path = original_project_dir.join(&diff.file_path);
            let ws_path = workspace_dir.join(&diff.file_path);

            match diff.status.as_str() {
                "added" | "modified" => {
                    if ws_path.exists() {
                        if let Some(parent) = orig_path.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        fs::copy(&ws_path, &orig_path)
                            .with_context(|| format!("Failed to copy {:?} to {:?}", ws_path, orig_path))?;
                        applied_count += 1;
                    }
                }
                "deleted" => {
                    if orig_path.exists() {
                        fs::remove_file(&orig_path)
                            .with_context(|| format!("Failed to delete {:?}", orig_path))?;
                        applied_count += 1;
                    }
                }
                _ => {}
            }
        }

        Ok(applied_count)
    }
}
