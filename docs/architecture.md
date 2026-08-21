# DeepSeek Harness Linux — Architecture Specification

## 1. System Overview

**DeepSeek Harness Linux** is a native Linux desktop application built on **Tauri 2**, **Rust**, **TypeScript**, and **React**. It acts as a robust, secure host and process orchestrator around the official `@deepseek-ai/dsh` runtime.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         DeepSeek Harness Linux                              │
│                            Tauri 2 Desktop                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  TypeScript / React Frontend (Vite)                                         │
│  ┌────────────────────────────────────┐ ┌────────────────────────────────┐ │
│  │ DSH Web Application (Embedded)     │ │ Native Desktop Management UI   │ │
│  │ Chat, Sessions, Tools, Streaming   │ │ Providers, Updates, Sandbox,   │ │
│  │ Session Management, Projections    │ │ Snapshots, Git, Logs, Settings │ │
│  └────────────────────────────────────┘ └────────────────────────────────┘ │
│                                    │                                        │
│                                    ▼ Tauri IPC                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │ Rust Native Backend                                                   │  │
│  │  - DSH Process Manager (Supervisor, Crash Recovery, Redaction)         │  │
│  │  - Runtime Manager (Isolated ~/.local/share/.../runtime/versions/)    │  │
│  │  - Update Manager (Atomic Staging, Smoke Tests, Rollback)             │  │
│  │  - Secret Store (Linux Secret Service / Keyring + AES Fallback)       │  │
│  │  - Provider Manager (OpenAI, Anthropic, DeepSeek, Gemini, Local)      │  │
│  │  - Sandbox & Safe Testing (Workspaces, Diff Generator, Test Gating)   │  │
│  │  - Project & Snapshot Manager (Git status, SQLite metadata)           │  │
│  │  - Health Checker & Diagnostics Exporter                              │  │
│  │  - Linux Desktop Integration (Tray, Notifications, File Dialogs)      │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                    │                                        │
│                                    ▼ Child Process Management               │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │ Node.js Runtime & Isolated @deepseek-ai/dsh Package                   │  │
│  │  - Managed DSH Home ($DSH_HOME)                                       │  │
│  │  - Web Profile Boot (`dsh web --no-open --port <port>`)               │  │
│  │  - JSON-RPC + SSE API Gateway (`/api`) & Built Web UI                │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Directory Layout on Linux

The application adheres strictly to the XDG Base Directory specification (`$XDG_DATA_HOME`, `$XDG_CONFIG_HOME`, `$XDG_STATE_HOME`, `$XDG_CACHE_HOME`):

```
~/.local/share/deepseek-harness-linux/
├── runtime/
│   ├── versions/
│   │   ├── 0.1.0-rc.7/
│   │   │   ├── package.json
│   │   │   ├── node_modules/
│   │   │   └── lib/
│   │   └── 0.1.0-rc.8/
│   │       ├── package.json
│   │       ├── node_modules/
│   │       └── lib/
│   ├── current -> versions/0.1.0-rc.8   (atomic symlink)
│   ├── previous -> versions/0.1.0-rc.7  (fallback symlink)
│   ├── staging/                         (temp download & smoke validation)
│   └── downloads/                       (cached npm tarballs)
├── data/
│   ├── database/
│   │   └── dsh_desktop.sqlite           (app settings, providers, snapshots)
│   ├── dsh-home/                        (isolated $DSH_HOME for DSH profiles)
│   │   ├── cordis.patch.yml
│   │   ├── settings.yaml
│   │   └── sessions/
│   └── logs/
│       ├── app.log
│       ├── dsh_runtime.log
│       ├── updater.log
│       └── sandbox.log
└── sandbox/
    ├── workspaces/
    │   └── ws_<id>/                     (isolated staging project copies)
    └── snapshots/
        └── snap_<id>.tar.gz             (pre-execution project snapshots)
```

---

## 3. Core Rust Subsystems

### 3.1 DshProcessManager
- **Lifecycle Control**: Spawns `node <dsh-bin> web --no-open --port <port>` with isolated environment variables.
- **Port Allocation**: Dynamically discovers free loopback ports (avoiding port conflicts) and listens for the `dsh web: http://127.0.0.1:<port>` startup signal.
- **Health Monitoring**: Periodically polls `/api/host.describe` or HTTP ping to verify liveness.
- **Crash Recovery**: Automatically restarts failing processes up to configured limit with exponential backoff (e.g., 3 attempts in 60s) before alerting the user.
- **Log Streaming & Redaction**: Captures stdout/stderr in memory and disk, stripping API keys, tokens, and Authorization headers via regex filters.
- **Graceful Shutdown**: Sends `SIGTERM`, waits up to 5s, then forces `SIGKILL` and process tree cleanup.

### 3.2 RuntimeManager & UpdateManager
- **Version Isolation**: Stores independent runtime folders under `runtime/versions/<version>/`.
- **Atomic Symlink Update**:
  1. Download npm package `@deepseek-ai/dsh` into `staging/<version>/`.
  2. Verify npm tarball integrity and run npm/pnpm install in staging.
  3. Run smoke test: execute `dsh --version` and verify exit code.
  4. Perform atomic symlink update: point `staging` -> `versions/<version>`, then `current` -> `versions/<version>`, updating `previous` to prior known-good version.
  5. If boot or health check fails after update, automatically switch `current` symlink back to `previous`.

### 3.3 SecretStore (Linux Keyring Integration)
- Uses DBus Secret Service API (`secret-service` / `libsecret`) to securely store API keys in GNOME Keyring / KWallet.
- Encrypted local fallback (AES-256-GCM with PBKDF2 derived machine key) when DBus Secret Service is unavailable (e.g. headless/minimal Linux).
- Exposes only references (`apiKeyEnv: DSH_SECRET_<UUID>`) to configuration files, never raw secrets.

### 3.4 ProviderManager
- Supports:
  - DeepSeek Official (`deepseek-chat`, `deepseek-reasoner`)
  - OpenAI (`gpt-4o`, `o1`, `o3-mini`, etc.)
  - Anthropic (`claude-3-5-sonnet`, `claude-3-7-sonnet`, etc.)
  - Google Gemini (`gemini-2.5-pro`, `gemini-2.5-flash`, etc.)
  - OpenRouter, OmniRoute, Local Ollama, and OpenAI-compatible gateways.
- Automatic Model Discovery via `GET /v1/models` or provider discovery APIs with live connectivity test.
- Writes configurations safely into `$DSH_HOME/cordis.patch.yml` or `$DSH_HOME/settings.yaml`.

### 3.5 Sandbox & Safe Testing Workspace
- Detects project type:
  - **Python**: Detects `.venv`, `pyproject.toml`, `requirements.txt`. Recreates or references isolated virtual environment rather than copying absolute-path `.venv`.
  - **Node.js**: Detects `package.json`, `pnpm-lock.yaml`, `package-lock.json`. Re-links dependencies via hardlinks/symlinks rather than full copy.
  - **Rust**: Detects `Cargo.toml`. Safely manages `target/` directory cache.
- Test-gated execution pipeline:
  `Original Code -> Create Snapshot -> Copy to Sandbox -> Agent Applies Changes -> Run Tests/Linter -> Compute Unified Diff -> Present User Review -> User Approves -> Commit to Original`.

### 3.6 Project Snapshots & Git Integration
- Native Git inspection via `git2` / CLI (branch, commit hash, dirty files, diffs).
- Creates timestamped lightweight tarball snapshots before agent operations.
- Snapshot browser: view diff against current project, restore snapshot, or delete.

### 3.7 Linux Native Integration
- **Window Management**: Custom native titlebar option or native GTK/Wayland window decorations.
- **System Tray**: Menu with Quick Launch, Status, Toggle Window, Check for Updates, and Quit.
- **Desktop Notifications**: Standard freedesktop notification interface via `notify-rust` / Tauri notification API.
- **Native File Dialogs**: File and folder pickers via `rfd` / Tauri dialogs.
- **Packaging**: AppImage and Fedora RPM builds.

---

## 4. Frontend Architecture (React + TypeScript)

The frontend is organized into two integrated views:
1. **DSH Web View**: An embedded `<iframe>` or webview container hosting the official DSH Web application on `http://127.0.0.1:<port>`.
2. **Desktop Management Shell**: A sleek, collapsible sidebar and floating navigation providing:
   - **Status & Health Bar**: Active DSH status, memory, current model, and active workspace.
   - **Provider & Model Manager**: Easy credential setup, model selection, custom endpoints, and connectivity verification.
   - **Runtime & Updates Panel**: Current version, available update channels, update history, and 1-click rollback.
   - **Sandbox & Testing Panel**: Active test workspaces, running test outputs, visual unified diffs, and approve/reject change buttons.
   - **Project Snapshots & Git View**: Commit status, snapshots list, diff comparisons, and instant restore.
   - **Settings & Diagnostics**: System preferences, autostart, logging levels, diagnostic ZIP export.

---

## 5. Persistence (SQLite Database Schema)

SQLite metadata database located at `$XDG_DATA_HOME/deepseek-harness-linux/data/database/dsh_desktop.sqlite`:

```sql
CREATE TABLE IF NOT EXISTS application_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS providers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    provider_type TEXT NOT NULL, -- 'deepseek', 'openai', 'anthropic', 'gemini', 'openrouter', 'ollama', 'custom'
    base_url TEXT NOT NULL,
    secret_ref TEXT,             -- reference key in Secret Store
    is_default BOOLEAN DEFAULT 0,
    compat_mode TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS provider_models (
    id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    model_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    context_window INTEGER,
    max_tokens INTEGER,
    supports_reasoning BOOLEAN DEFAULT 0,
    supports_vision BOOLEAN DEFAULT 0,
    supports_tools BOOLEAN DEFAULT 1,
    discovered_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS runtime_versions (
    version TEXT PRIMARY KEY,
    channel TEXT NOT NULL,       -- 'stable', 'rc', 'dev'
    installed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status TEXT NOT NULL,        -- 'active', 'inactive', 'staging', 'failed'
    integrity_hash TEXT,
    install_path TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS update_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_version TEXT,
    to_version TEXT NOT NULL,
    status TEXT NOT NULL,        -- 'success', 'failed', 'rolled_back'
    error_message TEXT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    project_type TEXT NOT NULL,  -- 'node', 'python', 'rust', 'general'
    last_opened_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS project_snapshots (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    snapshot_path TEXT NOT NULL,
    git_commit TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS sandbox_runs (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    workspace_path TEXT NOT NULL,
    status TEXT NOT NULL,        -- 'running', 'passed', 'failed', 'applied', 'discarded'
    test_command TEXT,
    test_output TEXT,
    diff_content TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```
