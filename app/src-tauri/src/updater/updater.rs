use crate::runtime::{RuntimeInstaller, RuntimeManager};
use crate::storage::Database;
use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UpdateChannel {
    Stable,
    ReleaseCandidate,
    Development,
}

impl Default for UpdateChannel {
    fn default() -> Self {
        Self::Stable
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub has_update: bool,
    pub current_version: Option<String>,
    pub target_version: String,
    pub channel: UpdateChannel,
    pub all_versions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResult {
    pub success: bool,
    pub previous_version: Option<String>,
    pub new_version: String,
    pub message: String,
}

#[derive(Deserialize)]
struct NpmPackageMeta {
    #[serde(rename = "dist-tags")]
    dist_tags: HashMap<String, String>,
    versions: Option<HashMap<String, serde_json::Value>>,
}

pub struct UpdateManager {
    runtime_mgr: RuntimeManager,
    db: Database,
    client: Client,
}

impl UpdateManager {
    pub fn new(runtime_mgr: RuntimeManager, db: Database) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(8))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            runtime_mgr,
            db,
            client,
        }
    }

    /// Checks the official npm registry for available updates.
    pub async fn check_for_updates(&self, channel: UpdateChannel) -> Result<UpdateCheckResult> {
        let url = "https://registry.npmjs.org/@deepseek-ai%2Fdsh";
        let resp = self.client.get(url).send().await
            .context("Failed to connect to npm registry")?;

        if !resp.status().is_success() {
            bail!("npm registry returned HTTP status {}", resp.status());
        }

        let meta: NpmPackageMeta = resp.json().await
            .context("Failed to parse npm package metadata")?;

        let target_version = match channel {
            UpdateChannel::Stable => {
                meta.dist_tags.get("latest")
                    .cloned()
                    .unwrap_or_else(|| "0.1.0-rc.7".to_string())
            }
            UpdateChannel::ReleaseCandidate => {
                meta.dist_tags.get("next")
                    .or_else(|| meta.dist_tags.get("latest"))
                    .cloned()
                    .unwrap_or_else(|| "0.1.0-rc.8".to_string())
            }
            UpdateChannel::Development => {
                let mut vers: Vec<String> = meta.versions
                    .as_ref()
                    .map(|v| v.keys().cloned().collect())
                    .unwrap_or_default();
                vers.sort();
                vers.last().cloned().unwrap_or_else(|| "0.1.0-rc.8".to_string())
            }
        };

        let mut all_versions: Vec<String> = meta.versions
            .as_ref()
            .map(|v| v.keys().cloned().collect())
            .unwrap_or_default();
        all_versions.sort();

        let runtime_info = self.runtime_mgr.get_runtime_info()?;
        let current_ver = runtime_info.current_version;

        let has_update = match &current_ver {
            Some(curr) => curr != &target_version,
            None => true,
        };

        Ok(UpdateCheckResult {
            has_update,
            current_version: current_ver,
            target_version,
            channel,
            all_versions,
        })
    }

    /// Performs an atomic update into staging, validates via smoke tests, moves into versions, and activates.
    pub async fn apply_update(&self, target_version: &str) -> Result<UpdateResult> {
        let runtime_info = self.runtime_mgr.get_runtime_info()?;
        let current_ver = runtime_info.current_version.clone();

        let staging_version_dir = self.runtime_mgr.staging_dir().join(format!("staging_{}", target_version));
        let final_version_dir = self.runtime_mgr.versions_dir().join(target_version);

        // Clean staging if leftover
        if staging_version_dir.exists() {
            let _ = fs::remove_dir_all(&staging_version_dir);
        }

        // Step 1: Install into staging
        let installed_ver = RuntimeInstaller::install_package(&staging_version_dir, target_version)
            .context("Failed to install package into staging")?;

        // Step 2: Validate via smoke test
        RuntimeInstaller::smoke_test(&staging_version_dir)
            .context("Staged runtime failed smoke test validation")?;

        // Step 3: Move staging to final versions directory
        if final_version_dir.exists() {
            let _ = fs::remove_dir_all(&final_version_dir);
        }
        fs::rename(&staging_version_dir, &final_version_dir)
            .with_context(|| format!("Failed to move {:?} to {:?}", staging_version_dir, final_version_dir))?;

        // Step 4: Activate version atomically
        self.runtime_mgr.activate_version(&installed_ver)
            .context("Failed to activate new runtime version")?;

        // Record in database
        let _ = self.db.record_update(
            current_ver.as_deref(),
            &installed_ver,
            "success",
            None,
        );

        Ok(UpdateResult {
            success: true,
            previous_version: current_ver,
            new_version: installed_ver.clone(),
            message: format!("Successfully installed and activated DSH version {}", installed_ver),
        })
    }

    /// Rolls back the active DSH runtime to the previous known-good version.
    pub fn rollback(&self) -> Result<UpdateResult> {
        let prev_ver = self.runtime_mgr.rollback()?;
        let _ = self.db.record_update(
            None,
            &prev_ver,
            "rolled_back",
            Some("Manual user rollback"),
        );

        Ok(UpdateResult {
            success: true,
            previous_version: None,
            new_version: prev_ver.clone(),
            message: format!("Successfully rolled back to version {}", prev_ver),
        })
    }
}
