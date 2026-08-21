use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatusInfo {
    pub is_git_repo: bool,
    pub branch: Option<String>,
    pub head_commit: Option<String>,
    pub modified_files: Vec<String>,
    pub staged_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub is_clean: bool,
}

pub struct GitInspector;

impl GitInspector {
    /// Inspects git status of the project directory.
    pub fn inspect_project(project_dir: &Path) -> Result<GitStatusInfo> {
        let git_dir = project_dir.join(".git");
        if !git_dir.exists() {
            return Ok(GitStatusInfo {
                is_git_repo: false,
                branch: None,
                head_commit: None,
                modified_files: Vec::new(),
                staged_files: Vec::new(),
                untracked_files: Vec::new(),
                is_clean: true,
            });
        }

        let branch = Command::new("git")
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .current_dir(project_dir)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

        let head_commit = Command::new("git")
            .arg("rev-parse")
            .arg("--short")
            .arg("HEAD")
            .current_dir(project_dir)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

        let mut modified_files = Vec::new();
        let mut staged_files = Vec::new();
        let mut untracked_files = Vec::new();

        if let Ok(output) = Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .current_dir(project_dir)
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.len() < 3 {
                        continue;
                    }
                    let index_status = &line[0..1];
                    let worktree_status = &line[1..2];
                    let filename = line[3..].trim().to_string();

                    if index_status == "?" {
                        untracked_files.push(filename);
                    } else {
                        if index_status != " " && index_status != "?" {
                            staged_files.push(filename.clone());
                        }
                        if worktree_status != " " && worktree_status != "?" {
                            modified_files.push(filename);
                        }
                    }
                }
            }
        }

        let is_clean = modified_files.is_empty() && staged_files.is_empty() && untracked_files.is_empty();

        Ok(GitStatusInfo {
            is_git_repo: true,
            branch,
            head_commit,
            modified_files,
            staged_files,
            untracked_files,
            is_clean,
        })
    }
}
