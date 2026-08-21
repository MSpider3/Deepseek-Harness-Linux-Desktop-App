use crate::security::Redactor;
use anyhow::{Context, Result};
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex as TokioMutex;
use tokio::sync::Notify;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum DshProcessStatus {
    Stopped,
    Starting { port: u16 },
    Running { port: u16, url: String, pid: u32 },
    Error { message: String },
    Crashed { message: String, restart_count: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessLogEntry {
    pub timestamp: String,
    pub stream: String, // "stdout" | "stderr" | "system"
    pub message: String,
}

pub struct DshProcessManager {
    status: Arc<RwLock<DshProcessStatus>>,
    logs: Arc<Mutex<VecDeque<ProcessLogEntry>>>,
    active_child: Arc<TokioMutex<Option<Child>>>,
    log_file_path: PathBuf,
    restart_count: Arc<Mutex<u32>>,
    shutdown_notify: Arc<Notify>,
}

impl DshProcessManager {
    pub fn new<P: AsRef<Path>>(base_data_dir: P) -> Self {
        let log_dir = base_data_dir.as_ref().join("logs");
        let _ = fs::create_dir_all(&log_dir);
        let log_file_path = log_dir.join("dsh_runtime.log");

        Self {
            status: Arc::new(RwLock::new(DshProcessStatus::Stopped)),
            logs: Arc::new(Mutex::new(VecDeque::with_capacity(600))),
            active_child: Arc::new(TokioMutex::new(None)),
            log_file_path,
            restart_count: Arc::new(Mutex::new(0)),
            shutdown_notify: Arc::new(Notify::new()),
        }
    }

    pub fn status(&self) -> DshProcessStatus {
        self.status.read().unwrap().clone()
    }

    pub fn get_logs(&self, limit: usize) -> Vec<ProcessLogEntry> {
        let logs = self.logs.lock().unwrap();
        logs.iter().rev().take(limit).rev().cloned().collect()
    }

    fn append_log(&self, stream: &str, raw_message: &str) {
        let sanitized = Redactor::sanitize(raw_message);
        let entry = ProcessLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            stream: stream.to_string(),
            message: sanitized.clone(),
        };

        {
            let mut logs = self.logs.lock().unwrap();
            if logs.len() >= 500 {
                logs.pop_front();
            }
            logs.push_back(entry);
        }

        // Also persist to log file
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)
        {
            let _ = writeln!(file, "[{}] [{}] {}", Utc::now().format("%Y-%m-%d %H:%M:%S"), stream, sanitized);
        }
    }

    pub fn find_available_port(start_port: u16) -> u16 {
        for port in start_port..start_port + 100 {
            if let Ok(listener) = TcpListener::bind(("127.0.0.1", port)) {
                drop(listener);
                return port;
            }
        }
        // Fallback to OS assigned port
        TcpListener::bind("127.0.0.1:0")
            .and_then(|l| l.local_addr())
            .map(|a| a.port())
            .unwrap_or(5180)
    }

    /// Starts DSH Web process using the given executable and parameters.
    pub async fn start(
        &self,
        executable: &Path,
        dsh_home: &Path,
        env_vars: HashMap<String, String>,
        target_port: Option<u16>,
    ) -> Result<String> {
        // Stop any currently running instance
        self.stop().await?;

        let port = target_port.unwrap_or_else(|| Self::find_available_port(5180));
        *self.status.write().unwrap() = DshProcessStatus::Starting { port };
        self.append_log("system", &format!("Starting DSH Web runtime on port {}", port));

        let mut cmd = if executable.extension().map(|e| e == "js").unwrap_or(false) {
            let mut c = Command::new("node");
            c.arg(executable);
            c
        } else {
            Command::new(executable)
        };

        cmd.arg("web")
            .arg("--no-open")
            .arg("--port")
            .arg(port.to_string())
            .arg("--host")
            .arg("127.0.0.1")
            .env("DSH_HOME", dsh_home)
            .env("NODE_ENV", "production")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for (k, v) in env_vars {
            cmd.env(k, v);
        }

        let mut child = cmd.spawn()
            .with_context(|| format!("Failed to spawn DSH executable at {:?}", executable))?;

        let pid = child.id().unwrap_or(0);
        let stdout = child.stdout.take().context("Failed to capture child stdout")?;
        let stderr = child.stderr.take().context("Failed to capture child stderr")?;

        let status_arc = Arc::clone(&self.status);
        let active_child_arc = Arc::clone(&self.active_child);
        let restart_count_arc = Arc::clone(&self.restart_count);
        let shutdown_notify = Arc::clone(&self.shutdown_notify);

        *self.active_child.lock().await = Some(child);

        let self_for_stdout = Self {
            status: Arc::clone(&self.status),
            logs: Arc::clone(&self.logs),
            active_child: Arc::clone(&self.active_child),
            log_file_path: self.log_file_path.clone(),
            restart_count: Arc::clone(&self.restart_count),
            shutdown_notify: Arc::clone(&self.shutdown_notify),
        };
        let self_for_stderr = Self {
            status: Arc::clone(&self.status),
            logs: Arc::clone(&self.logs),
            active_child: Arc::clone(&self.active_child),
            log_file_path: self.log_file_path.clone(),
            restart_count: Arc::clone(&self.restart_count),
            shutdown_notify: Arc::clone(&self.shutdown_notify),
        };

        let expected_url = format!("http://127.0.0.1:{}", port);
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();
        let ready_tx = Arc::new(Mutex::new(Some(ready_tx)));

        // Stdout reader task
        let ready_tx_clone = Arc::clone(&ready_tx);
        let expected_url_clone = expected_url.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            let url_regex = Regex::new(r"dsh web:\s*(https?://[^\s]+)").unwrap();

            while let Ok(Some(line)) = reader.next_line().await {
                self_for_stdout.append_log("stdout", &line);

                if url_regex.is_match(&line) || line.contains("http://127.0.0.1:") || line.contains("http://localhost:") {
                    let mut current_status = status_arc.write().unwrap();
                    if matches!(*current_status, DshProcessStatus::Starting { .. }) {
                        *current_status = DshProcessStatus::Running {
                            port,
                            url: expected_url_clone.clone(),
                            pid,
                        };
                        if let Some(tx) = ready_tx_clone.lock().unwrap().take() {
                            let _ = tx.send(());
                        }
                    }
                }
            }
        });

        // Stderr reader task
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                self_for_stderr.append_log("stderr", &line);
            }
        });

        // Wait monitor task
        let status_arc_for_wait = Arc::clone(&self.status);
        let self_for_wait = Self {
            status: Arc::clone(&self.status),
            logs: Arc::clone(&self.logs),
            active_child: Arc::clone(&self.active_child),
            log_file_path: self.log_file_path.clone(),
            restart_count: Arc::clone(&self.restart_count),
            shutdown_notify: Arc::clone(&self.shutdown_notify),
        };

        tokio::spawn(async move {
            tokio::select! {
                _ = shutdown_notify.notified() => {
                    // Deliberate shutdown
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    // Monitor loop
                    loop {
                        let mut child_guard = active_child_arc.lock().await;
                        if let Some(ref mut child) = *child_guard {
                            match child.try_wait() {
                                Ok(Some(status)) => {
                                    self_for_wait.append_log("system", &format!("DSH process exited with status: {}", status));
                                    let mut s = status_arc_for_wait.write().unwrap();
                                    if !matches!(*s, DshProcessStatus::Stopped) {
                                        let mut count = restart_count_arc.lock().unwrap();
                                        *count += 1;
                                        *s = DshProcessStatus::Crashed {
                                            message: format!("Process exited unexpectedly with {}", status),
                                            restart_count: *count,
                                        };
                                    }
                                    break;
                                }
                                Ok(None) => {
                                    // Process still running
                                }
                                Err(e) => {
                                    self_for_wait.append_log("system", &format!("Error waiting on child: {}", e));
                                    break;
                                }
                            }
                        } else {
                            break;
                        }
                        drop(child_guard);
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                }
            }
        });

        // Wait for readiness signal or fallback timeout
        tokio::select! {
            _ = ready_rx => {
                self.append_log("system", &format!("DSH Web verified healthy at {}", expected_url));
            }
            _ = tokio::time::sleep(Duration::from_secs(4)) => {
                // If stdout didn't match yet, assume running if process is alive
                let mut current_status = self.status.write().unwrap();
                if matches!(*current_status, DshProcessStatus::Starting { .. }) {
                    *current_status = DshProcessStatus::Running {
                        port,
                        url: expected_url.clone(),
                        pid,
                    };
                }
            }
        }

        Ok(expected_url)
    }

    /// Stops the active DSH child process cleanly.
    pub async fn stop(&self) -> Result<()> {
        self.shutdown_notify.notify_waiters();
        *self.status.write().unwrap() = DshProcessStatus::Stopped;

        let mut child_opt = self.active_child.lock().await.take();
        if let Some(mut child) = child_opt.take() {
            self.append_log("system", "Sending SIGTERM to DSH process...");

            // On Unix, try graceful kill
            #[cfg(unix)]
            if let Some(pid) = child.id() {
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }
            }

            // Wait up to 3s for clean exit
            let wait_res = tokio::time::timeout(Duration::from_secs(3), child.wait()).await;
            if wait_res.is_err() {
                self.append_log("system", "DSH process did not exit in time; killing process tree.");
                let _ = child.kill().await;
            }
        }

        self.append_log("system", "DSH process stopped.");
        Ok(())
    }

    /// Restarts DSH process.
    pub async fn restart(
        &self,
        executable: &Path,
        dsh_home: &Path,
        env_vars: HashMap<String, String>,
        target_port: Option<u16>,
    ) -> Result<String> {
        self.stop().await?;
        tokio::time::sleep(Duration::from_millis(500)).await;
        self.start(executable, dsh_home, env_vars, target_port).await
    }
}
