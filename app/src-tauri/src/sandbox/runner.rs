use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub stdout: String,
    pub stderr: String,
    pub command: String,
}

pub struct TestRunner;

impl TestRunner {
    /// Executes a test command in the sandbox workspace directory.
    pub async fn run_test(workspace_dir: &Path, command_str: &str) -> Result<TestResult> {
        let start = std::time::Instant::now();
        let trimmed_cmd = command_str.trim();

        let child = Command::new("bash")
            .arg("-c")
            .arg(trimmed_cmd)
            .current_dir(workspace_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to execute test command '{}'", trimmed_cmd))?;

        let timeout_duration = Duration::from_secs(120);
        let output = tokio::time::timeout(timeout_duration, child.wait_with_output()).await
            .context("Test execution timed out after 120 seconds")??;

        let duration_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(TestResult {
            success: output.status.success(),
            exit_code: output.status.code(),
            duration_ms,
            stdout,
            stderr,
            command: trimmed_cmd.to_string(),
        })
    }
}
