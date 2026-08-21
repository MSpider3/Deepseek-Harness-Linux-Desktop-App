# DeepSeek Harness Linux — Comprehensive Developer & Contributor Guide

Welcome to the **DeepSeek Harness Linux** developer architecture and contribution guide. This document provides a comprehensive technical blueprint of the desktop application's internal subsystems, IPC contracts, data flows, and step-by-step instructions for extending the codebase.

---

## 🏗️ 1. High-Level Architecture & System Design

DeepSeek Harness Linux is engineered as a **native Linux desktop shell wrapping the official `@deepseek-ai/dsh` upstream distribution**. Rather than maintaining a fragile fork of DeepSeek Harness, our application manages upstream `@deepseek-ai/dsh` as an isolated child runtime, enhancing it with native Linux system capabilities.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    DeepSeek Harness Linux Desktop Shell                     │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                    React 18 + TypeScript Frontend                      │  │
│  │  - Viewports: DSH Webview, Providers, Runtime, Sandbox, Snapshots,     │  │
│  │               Streaming Logs, System Settings, First Run Wizard        │  │
│  │  - Service Bridge: tauriApi.ts (Typed IPC invoke wrapper)             │  │
│  └───────────────────────────────────┬───────────────────────────────────┘  │
│                                      │ Tauri IPC (Commands / Events)        │
│                                      ▼                                      │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                       Rust Backend Core (Tauri 2)                     │  │
│  │                                                                       │  │
│  │  ┌─────────────────────────┐         ┌─────────────────────────────┐  │  │
│  │  │   Process Supervisor    │         │    Provider Config Syncer   │  │  │
│  │  │ - Spawns Node child     │         │ - Writes .credentials.yaml  │  │  │
│  │  │ - Dynamic loopback port │         │ - Writes settings.yaml      │  │  │
│  │  │ - Ring buffer logging   │         │ - Writes .env & patch.yml   │  │  │
│  │  └────────────┬────────────┘         └──────────────┬──────────────┘  │  │
│  │               │                                     │                 │  │
│  │  ┌────────────┴────────────┐         ┌──────────────┴──────────────┐  │  │
│  │  │   Secret Store (Keyring)│         │   Sandbox & Diff Applier    │  │  │
│  │  │ - Linux Secret Service  │         │ - Isolated temp workspace   │  │  │
│  │  │ - AES-256-GCM vault     │         │ - Multi-framework test run  │  │  │
│  │  │ - Redaction filter      │         │ - Unified diff & patcher    │  │  │
│  │  └────────────┬────────────┘         └──────────────┬──────────────┘  │  │
│  │               │                                     │                 │  │
│  │  ┌────────────┴────────────┐         ┌──────────────┴──────────────┐  │  │
│  │  │   Runtime Manager       │         │   SQLite Database (Storage) │  │  │
│  │  │ - Multi-version isolate │         │ - Providers, Snapshots      │  │  │
│  │  │ - Atomic symlinks       │         │ - Settings & Audit logs     │  │  │
│  │  │ - Registry updater      │         │ - XDG: ~/.local/share/...   │  │  │
│  │  └─────────────────────────┘         └─────────────────────────────┘  │  │
│  └───────────────────────────────────┬───────────────────────────────────┘  │
│                                      │ Child Process Exec / HTTP Loopback   │
│                                      ▼                                      │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                 Official DeepSeek Harness Node Runtime                │  │
│  │  - Cordis Microkernel (@deepseek-ai/dsh)                              │  │
│  │  - HTTP Web UI running on 127.0.0.1:<allocated_port>                  │  │
│  │  - Hot-reload credential & settings watchers                          │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 🧩 2. Core Rust Subsystems (`app/src-tauri/src/`)

### A. Process Supervisor (`dsh/process.rs` & `dsh/health.rs`)
- **Role**: Manages the lifecycle of `@deepseek-ai/dsh` child process.
- **Port Allocation**: Selects an ephemeral loopback port (`5180+`) and verifies loopback availability.
- **Environment Injection**: Injects decrypted credentials via environment variables (`DEEPSEEK_API_KEY`, etc.) and sets `$DSH_HOME`.
- **Log Streaming**: Captures stdout/stderr in real-time, passes output through the **Redaction Engine**, and buffers the most recent 1,000 log events in memory.
- **Graceful Shutdown**: Sends `SIGTERM`, waits up to 5 seconds, and falls back to `SIGKILL` to prevent orphaned background processes.

### B. Provider & Config Syncer (`providers/sync.rs`)
- **Role**: Synchronizes configured AI providers and secret keys into upstream DSH's hot-reload file format.
- **Upstream Watcher Contract**:
  - `$DSH_HOME/.credentials.yaml` (Linux file mode `0600`): Maps environment variable keys to secrets for `@deepseek-ai/dsh-credentials-local`.
  - `$DSH_HOME/settings.yaml` (Linux file mode `0600`): Configures `llm-deepseek` (`baseURL`, `apiKeyEnv: DEEPSEEK_API_KEY`) and `llm-pi-ai` (`providers.<route>: { apiKeyEnv, baseURL, displayName, models }`) for `@deepseek-ai/dsh-settings-file`.
  - `$DSH_HOME/.env` (mode `0600`): Environment variable mappings for subagents and CLI tools.
  - `$DSH_HOME/cordis.patch.yml`: Upstream Cordis patch configuration.

### C. Security & Secret Store (`security/keyring.rs` & `security/redaction.rs`)
- **Linux Secret Service**: Connects over DBus to GNOME Keyring / KWallet to store credentials securely.
- **Encrypted Vault Fallback**: If DBus Secret Service is unavailable (e.g. headless or custom WM), derives an AES-256-GCM encryption key using PBKDF2 (100,000 iterations) from machine-specific hardware IDs (`/etc/machine-id`) to encrypt `dsh_vault.enc`.
- **Redaction Engine**: Regex rules scrub OpenAI, DeepSeek, Anthropic, and generic API keys (`sk-...`, `Bearer ...`) from log events before they reach the UI or disk.

### D. Multi-Version Runtime Manager (`runtime/manager.rs` & `installer.rs`)
- **Version Isolation**: Versions are stored under `~/.local/share/deepseek-harness-linux/runtime/versions/<version>/`.
- **Atomic Symlinks**: 
  - `runtime/current` -> currently active version.
  - `runtime/previous` -> previous working version for instant 1-click rollback.
- **NPM Registry Tracking**: Queries `registry.npmjs.org/@deepseek-ai/dsh` to discover updates across **Stable** (`latest`), **Next** (`next`), and **Dev** channels.

### E. Sandbox & Safe Testing Engine (`sandbox/`)
- **Staging Workspace (`workspace.rs`)**: Creates an isolated temporary copy of the active project in `~/.local/share/deepseek-harness-linux/sandbox_workspaces/`.
- **Automatic Test Runner (`runner.rs`)**: Automatically inspects project root:
  - `package.json` -> Runs `npm test`
  - `Cargo.toml` -> Runs `cargo test`
  - `pytest.ini` / `pyproject.toml` -> Runs `pytest`
- **Unified Diff Engine (`diff.rs`)**: Computes structural diffs between original project and sandbox.
- **Atomic Applier (`applier.rs`)**: Test-gated patch application that copies verified modified files back to the primary workspace.

### F. Git Tracker & Snapshots (`projects/git.rs` & `snapshot.rs`)
- **Git Inspector**: Queries live git status, branch name, dirty file count, and last commit hash.
- **Snapshot Engine**: Creates gzip-compressed tarballs of the workspace before major agent actions, enabling point-in-time state restoration.

### G. Persistent SQLite Database (`storage/db.rs`)
- Stores provider definitions, snapshot metadata, application settings, and update histories in `~/.local/share/deepseek-harness-linux/database/dsh_desktop.sqlite`.

---

## 💻 3. Frontend Architecture (`app/frontend/src/`)

The frontend is built with **React 18, TypeScript, and Vanilla CSS** with custom glassmorphism and modern dark-mode design tokens.

### Key Components
- **`App.tsx`**: Main navigation routing and active view coordinator.
- **`components/Sidebar.tsx`**: Primary navigation bar with DSH status indicators.
- **`components/DshWebView.tsx`**: Embeds the official DSH Web UI in a responsive iframe with auto-reconnect and health polling.
- **`components/ProvidersView.tsx`**: AI provider management (DeepSeek, OpenAI, Anthropic, Gemini, Ollama, OpenRouter).
- **`components/SandboxView.tsx`**: Interactive sandbox staging workspace, automated test runner output, and side-by-side diff viewer.
- **`components/UpdatesView.tsx`**: Version switcher, channel selector, update checker, and rollback controls.
- **`components/SnapshotsView.tsx`**: Project snapshots manager with create and rollback buttons.
- **`components/LogsView.tsx`**: Live streaming log viewer with stream filters (`stdout`, `stderr`, `system`) and diagnostic export.
- **`components/SettingsView.tsx`**: App settings (auto-start, start port, theme, telemetry).
- **`services/tauriApi.ts`**: Strongly-typed TypeScript wrapper around Tauri `invoke()` IPC calls.

---

## 🛠️ 4. How to Extend & Contribute

### A. Adding a New Tauri IPC Command
1. Create or edit the command function in `app/src-tauri/src/commands/<your_module>.rs`:
   ```rust
   #[tauri::command]
   pub async fn my_custom_command(
       state: State<'_, AppState>,
       param: String,
   ) -> Result<MyResponse, String> {
       // Implementation
       Ok(MyResponse { ... })
   }
   ```
2. Export the command in `app/src-tauri/src/commands/mod.rs`.
3. Register the command handler in `app/src-tauri/src/lib.rs` inside `.invoke_handler(tauri::generate_handler![...])`.
4. Add typed wrapper in `app/frontend/src/services/tauriApi.ts`:
   ```typescript
   export async function myCustomCommand(param: string): Promise<MyResponse> {
     return await invoke('my_custom_command', { param });
   }
   ```

### B. Adding a New AI Provider Preset
1. Update `app/src-tauri/src/providers/manager.rs` and `app/frontend/src/components/ProvidersView.tsx` with provider default endpoints and models.
2. Ensure `app/src-tauri/src/providers/sync.rs` maps the provider route correctly into `$DSH_HOME/settings.yaml` under `llm-pi-ai.providers.<route>`.

### C. Adding a New Test Runner Detection
1. Open `app/src-tauri/src/sandbox/runner.rs`.
2. Add detection rule in `detect_project_type()` (e.g. `go.mod` -> `go test ./...`).
3. Add a corresponding unit test in `app/src-tauri/tests/unit_tests.rs`.

---

## 🧪 5. Testing & Validation Workflow

Always verify your changes before submitting a pull request:

```bash
# 1. Run all Rust backend unit and integration tests
cargo test --manifest-path app/src-tauri/Cargo.toml -- --nocapture

# 2. Run React frontend build & TypeScript typecheck
npm --prefix app/frontend run build

# 3. Test release packaging pipeline
chmod +x scripts/build_package.sh
./scripts/build_package.sh

# 4. Launch development application
npm run tauri:dev
```

---

## 🚀 6. Submitting Your Contribution

1. Fork `https://github.com/MSpider3/Deepseek-Harness-Linux-Desktop-App`.
2. Create your feature branch (`git checkout -b feature/my-enhancement`).
3. Commit surgical changes with descriptive messages (`git commit -m "feat(sandbox): add Go test runner support"`).
4. Push to your fork and submit a Pull Request to `main`.
