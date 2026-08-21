use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeVersionEntry {
    pub version: String,
    pub path: String,
    pub is_current: bool,
    pub is_previous: bool,
    pub installed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub is_installed: bool,
    pub current_version: Option<String>,
    pub previous_version: Option<String>,
    pub runtime_root: String,
    pub executable_path: Option<String>,
    pub node_version: Option<String>,
    pub versions: Vec<RuntimeVersionEntry>,
}

#[derive(Clone)]
pub struct RuntimeManager {
    base_runtime_dir: PathBuf,
}

impl RuntimeManager {
    pub fn new<P: AsRef<Path>>(base_runtime_dir: P) -> Self {
        Self {
            base_runtime_dir: base_runtime_dir.as_ref().to_path_buf(),
        }
    }

    pub fn versions_dir(&self) -> PathBuf {
        self.base_runtime_dir.join("versions")
    }

    pub fn staging_dir(&self) -> PathBuf {
        self.base_runtime_dir.join("staging")
    }

    pub fn current_symlink(&self) -> PathBuf {
        self.base_runtime_dir.join("current")
    }

    pub fn previous_symlink(&self) -> PathBuf {
        self.base_runtime_dir.join("previous")
    }

    pub fn dsh_home_dir(&self) -> PathBuf {
        self.base_runtime_dir.parent().unwrap_or(&self.base_runtime_dir).join("data").join("dsh-home")
    }

    pub fn ensure_directories(&self) -> Result<()> {
        fs::create_dir_all(self.versions_dir())?;
        fs::create_dir_all(self.staging_dir())?;
        fs::create_dir_all(self.dsh_home_dir())?;
        Ok(())
    }

    /// Resolves the currently active DSH binary executable path.
    pub fn get_executable_path(&self) -> Result<PathBuf> {
        let current = self.current_symlink();
        if !current.exists() {
            bail!("No active DSH runtime found at {:?}", current);
        }

        // Check common locations:
        // 1. current/node_modules/.bin/dsh
        let bin1 = current.join("node_modules").join(".bin").join("dsh");
        if bin1.exists() {
            return Ok(bin1);
        }

        // 2. current/node_modules/@deepseek-ai/dsh/lib/bin.js
        let bin2 = current.join("node_modules").join("@deepseek-ai").join("dsh").join("lib").join("bin.js");
        if bin2.exists() {
            return Ok(bin2);
        }

        // 3. current/lib/bin.js
        let bin3 = current.join("lib").join("bin.js");
        if bin3.exists() {
            return Ok(bin3);
        }

        bail!("Could not locate dsh executable inside active runtime directory {:?}", current);
    }

    /// Returns comprehensive information on installed runtimes.
    pub fn get_runtime_info(&self) -> Result<RuntimeInfo> {
        self.ensure_directories()?;

        let current_target = self.read_symlink_target(&self.current_symlink());
        let previous_target = self.read_symlink_target(&self.previous_symlink());

        let current_ver = current_target.as_ref().and_then(|p| p.file_name()).and_then(|n| n.to_str()).map(String::from);
        let previous_ver = previous_target.as_ref().and_then(|p| p.file_name()).and_then(|n| n.to_str()).map(String::from);

        let mut versions = Vec::new();
        if let Ok(entries) = fs::read_dir(self.versions_dir()) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let path = entry.path();
                    let is_current = current_ver.as_deref() == Some(&name);
                    let is_previous = previous_ver.as_deref() == Some(&name);

                    let installed_at = entry.metadata()
                        .and_then(|m| m.created().or_else(|_| m.modified()))
                        .map(|t| {
                            let dt: chrono::DateTime<chrono::Utc> = t.into();
                            dt.to_rfc3339()
                        })
                        .unwrap_or_else(|_| "unknown".to_string());

                    versions.push(RuntimeVersionEntry {
                        version: name,
                        path: path.to_string_lossy().to_string(),
                        is_current,
                        is_previous,
                        installed_at,
                    });
                }
            }
        }

        // Sort versions descending
        versions.sort_by(|a, b| b.version.cmp(&a.version));

        let exec_path = self.get_executable_path().ok().map(|p| p.to_string_lossy().to_string());
        let is_installed = current_ver.is_some() && exec_path.is_some();

        let node_version = std::process::Command::new("node")
            .arg("-v")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string());

        Ok(RuntimeInfo {
            is_installed,
            current_version: current_ver,
            previous_version: previous_ver,
            runtime_root: self.base_runtime_dir.to_string_lossy().to_string(),
            executable_path: exec_path,
            node_version,
            versions,
        })
    }

    /// Sets the active runtime version atomically by symlinking `current`.
    pub fn activate_version(&self, version: &str) -> Result<()> {
        let version_path = self.versions_dir().join(version);
        if !version_path.exists() {
            bail!("Runtime version {} does not exist at {:?}", version, version_path);
        }

        let current = self.current_symlink();
        let previous = self.previous_symlink();

        // If current symlink exists and points elsewhere, move it to previous
        if let Ok(old_target) = fs::read_link(&current) {
            let _ = fs::remove_file(&previous);
            let _ = std::os::unix::fs::symlink(&old_target, &previous);
        }

        // Atomic symlink switch via temp link
        let tmp_link = self.base_runtime_dir.join(format!(".tmp_current_{}", uuid::Uuid::new_v4()));
        std::os::unix::fs::symlink(&version_path, &tmp_link)
            .with_context(|| format!("Failed to create symlink {:?}", tmp_link))?;

        fs::rename(&tmp_link, &current)
            .with_context(|| format!("Failed to atomically rename symlink to {:?}", current))?;

        Ok(())
    }

    /// Rolls back the active version to the previous version.
    pub fn rollback(&self) -> Result<String> {
        let previous = self.previous_symlink();
        if !previous.exists() {
            bail!("No previous known-good version found to roll back to.");
        }

        let prev_target = fs::read_link(&previous)
            .context("Failed to read previous version symlink")?;
        let prev_version = prev_target
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid version directory name")?
            .to_string();

        self.activate_version(&prev_version)?;
        Ok(prev_version)
    }

    fn read_symlink_target(&self, path: &Path) -> Option<PathBuf> {
        if path.exists() || fs::symlink_metadata(path).is_ok() {
            fs::read_link(path).ok().map(|target| {
                if target.is_relative() {
                    path.parent().unwrap_or(Path::new("")).join(target)
                } else {
                    target
                }
            })
        } else {
            None
        }
    }
}
