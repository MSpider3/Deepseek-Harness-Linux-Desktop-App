use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct RuntimeInstaller;

impl RuntimeInstaller {
    /// Installs a specific version or dist-tag of `@deepseek-ai/dsh` into the target directory.
    pub fn install_package(target_dir: &Path, version_or_tag: &str) -> Result<String> {
        fs::create_dir_all(target_dir)
            .with_context(|| format!("Failed to create target directory {:?}", target_dir))?;

        // Write isolated package.json in the target directory
        let package_json_content = serde_json::json!({
            "name": "dsh-isolated-runtime",
            "version": "1.0.0",
            "private": true,
            "type": "module",
            "dependencies": {
                "@deepseek-ai/dsh": version_or_tag
            }
        });

        let pkg_path = target_dir.join("package.json");
        fs::write(&pkg_path, serde_json::to_string_pretty(&package_json_content)?)
            .with_context(|| format!("Failed to write package.json to {:?}", pkg_path))?;

        // Run npm install in target directory
        let output = Command::new("npm")
            .arg("install")
            .arg("--no-audit")
            .arg("--no-fund")
            .arg("--production")
            .current_dir(target_dir)
            .output()
            .context("Failed to spawn npm install. Is Node.js and npm installed?")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("npm install failed with status {}: {}", output.status, stderr);
        }

        // Validate installation by finding bin and running --version
        let bin_path = target_dir.join("node_modules").join(".bin").join("dsh");
        let installed_version = if bin_path.exists() {
            let ver_output = Command::new(&bin_path)
                .arg("--version")
                .output()
                .context("Failed to run dsh --version smoke test")?;
            if ver_output.status.success() {
                String::from_utf8_lossy(&ver_output.stdout).trim().to_string()
            } else {
                version_or_tag.to_string()
            }
        } else {
            // Check direct js entrypoint
            let bin_js = target_dir.join("node_modules").join("@deepseek-ai").join("dsh").join("lib").join("bin.js");
            if bin_js.exists() {
                let ver_output = Command::new("node")
                    .arg(&bin_js)
                    .arg("--version")
                    .output()
                    .context("Failed to run node dsh bin.js --version")?;
                if ver_output.status.success() {
                    String::from_utf8_lossy(&ver_output.stdout).trim().to_string()
                } else {
                    version_or_tag.to_string()
                }
            } else {
                bail!("DSH binary not found after npm installation in {:?}", target_dir);
            }
        };

        Ok(installed_version)
    }

    /// Validates that a target installation directory contains a runnable DSH runtime.
    pub fn smoke_test(target_dir: &Path) -> Result<String> {
        let bin_path = target_dir.join("node_modules").join(".bin").join("dsh");
        let bin_js = target_dir.join("node_modules").join("@deepseek-ai").join("dsh").join("lib").join("bin.js");

        let output = if bin_path.exists() {
            Command::new(&bin_path).arg("--version").output()?
        } else if bin_js.exists() {
            Command::new("node").arg(&bin_js).arg("--version").output()?
        } else {
            bail!("No DSH binary found to smoke test in {:?}", target_dir);
        };

        if !output.status.success() {
            bail!("Smoke test failed with code {:?}", output.status.code());
        }

        let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(ver)
    }
}
