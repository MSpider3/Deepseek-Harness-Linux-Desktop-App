use anyhow::Result;
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffFileSummary {
    pub file_path: String,
    pub status: String, // "modified" | "added" | "deleted"
    pub additions: usize,
    pub deletions: usize,
    pub unified_diff: String,
}

pub struct DiffGenerator;

impl DiffGenerator {
    /// Compares two directories and generates a unified diff and summary for all changed files.
    pub fn compute_directory_diff(original_dir: &Path, modified_dir: &Path) -> Result<Vec<DiffFileSummary>> {
        let mut summaries = Vec::new();
        let mut all_files = std::collections::BTreeSet::new();

        // Scan modified dir
        Self::collect_relative_files(modified_dir, modified_dir, &mut all_files)?;
        // Scan original dir
        Self::collect_relative_files(original_dir, original_dir, &mut all_files)?;

        for rel_path in all_files {
            // Skip .git, node_modules, target, .venv
            if rel_path.starts_with(".git")
                || rel_path.starts_with("node_modules")
                || rel_path.starts_with("target")
                || rel_path.starts_with(".venv")
            {
                continue;
            }

            let orig_file = original_dir.join(&rel_path);
            let mod_file = modified_dir.join(&rel_path);

            let orig_content = if orig_file.exists() {
                fs::read_to_string(&orig_file).unwrap_or_default()
            } else {
                String::new()
            };

            let mod_content = if mod_file.exists() {
                fs::read_to_string(&mod_file).unwrap_or_default()
            } else {
                String::new()
            };

            if orig_content == mod_content {
                continue;
            }

            let status = if !orig_file.exists() && mod_file.exists() {
                "added"
            } else if orig_file.exists() && !mod_file.exists() {
                "deleted"
            } else {
                "modified"
            };

            let diff = TextDiff::from_lines(&orig_content, &mod_content);
            let mut additions = 0;
            let mut deletions = 0;
            let mut unified_diff = String::new();

            for change in diff.iter_all_changes() {
                match change.tag() {
                    ChangeTag::Delete => {
                        deletions += 1;
                        unified_diff.push_str(&format!("-{}", change));
                    }
                    ChangeTag::Insert => {
                        additions += 1;
                        unified_diff.push_str(&format!("+{}", change));
                    }
                    ChangeTag::Equal => {
                        unified_diff.push_str(&format!(" {}", change));
                    }
                }
            }

            summaries.push(DiffFileSummary {
                file_path: rel_path,
                status: status.to_string(),
                additions,
                deletions,
                unified_diff,
            });
        }

        Ok(summaries)
    }

    fn collect_relative_files(base_dir: &Path, current_dir: &Path, set: &mut std::collections::BTreeSet<String>) -> Result<()> {
        if !current_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(current_dir)?.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::collect_relative_files(base_dir, &path, set)?;
            } else if path.is_file() {
                if let Ok(rel) = path.strip_prefix(base_dir) {
                    set.insert(rel.to_string_lossy().to_string());
                }
            }
        }
        Ok(())
    }
}
