# DeepSeek Harness — Upstream Capability Matrix

This matrix documents the inspection of the official upstream repository (`https://github.com/deepseek-ai/deepseek-harness` and `@deepseek-ai/dsh` v0.1.0-rc.8) against the requirements for the **DeepSeek Harness Linux** desktop application.

Status Legend:
- **FULL**: Upstream provides complete, production-ready implementation that can be reused directly without modification.
- **PARTIAL**: Upstream provides foundational abstractions, services, or partial support; wrapper extends or configures it without breaking upstream contracts.
- **ABSENT**: Not provided upstream; must be implemented entirely within the native Linux wrapper.
- **UNKNOWN**: Feature behavior requires dynamic runtime verification or varies across releases.

---

## Capability Matrix

| Feature | Upstream Status | Existing Implementation | Reuse? | Wrapper Work Required |
| :--- | :--- | :--- | :--- | :--- |
| **DSH CLI Entry Points** | `FULL` | `apps/cli/src/bin.ts` (`dsh` binary entrypoint), `args.ts` (Commander parser supporting `--profile`, `--patch`, `web`, `dump-config`, `plugin`). | **YES** | Launch via Rust process manager with arguments (`dsh web --no-open --port <port>`). |
| **DSH Web Profile & Server** | `FULL` | `@deepseek-ai/dsh-web-app` bundle, `webserver` (`node:http`), `apiproxy` (JSON-RPC + SSE gateway), `@deepseek-ai/dsh-web-frontend` (Vite dist). | **YES** | Spawn DSH process, parse stdout for URL/port, monitor health via HTTP endpoint, embed inside Tauri desktop webview. |
| **DSH Headless Profile** | `FULL` | `packages/bundle/headless`, `packages/preset/agent-presets` for non-interactive single-task execution. | **YES** | Can be invoked by sandbox runner and test automation. |
| **Bundle & Profile Architecture** | `FULL` | `@deepseek-ai/cordis` plugin system + `@deepseek-ai/dsh-app-boot` layered patch stack (`cordis.yml`, `cordis.patch.yml`, `--patch`). | **YES** | Generate non-destructive profile overlays (`cordis.patch.yml`) for custom provider configs and settings. |
| **Plugin System (Cordis)** | `FULL` | Extensible microkernel service/lifecycle architecture across 50+ packages. | **YES** | Extend runtime behavior via standard Cordis plugins/patches if needed. |
| **Configuration Files & `$DSH_HOME`** | `FULL` | `@deepseek-ai/dsh-home-paths` resolves `DSH_HOME` env var, fallback to `~/.dsh`. Reads `cordis.patch.yml`, `.credentials.yaml`, `settings.yaml`. | **YES** | Manage isolated `$DSH_HOME` under `~/.local/share/deepseek-harness-linux/data/dsh-home` per version/profile. |
| **Model / Provider Architecture** | `FULL` | `@deepseek-ai/dsh-llm` registry + `@deepseek-ai/dsh-llm-deepseek` + `@deepseek-ai/dsh-llm-pi-ai` (supports OpenAI, Anthropic, Gemini, OpenRouter, Ollama, OpenAI-compatible). | **YES** | Provide native desktop Provider Manager UI that generates and syncs provider configs and model catalogs into DSH settings. |
| **API Key Handling** | `PARTIAL` | `@deepseek-ai/dsh-credentials` (credential references via `apiKeyEnv` or `$DSH_HOME/.credentials.yaml`). Plaintext on disk if written directly. | **PARTIAL** | Store actual secrets in Linux Secret Service / system keyring; inject via isolated process environment or encrypted credential bridge. |
| **Model Discovery** | `PARTIAL` | `ctx.llm.discoverModels` endpoint discovery in `dsh-llm`; dynamic querying on supported providers. | **PARTIAL** | Add native model discovery coordinator in Rust/Tauri with test connection and manual model input fallbacks. |
| **Web UI & Chat Interface** | `FULL` | `@deepseek-ai/dsh-web-frontend` (React 18 + Vite), session views, chat, tools, streaming deltas, file viewing. | **YES** | Embed official Web UI in Tauri webview window; augment with native Linux framing, sidebar, and status overlays. |
| **Session Management** | `FULL` | `@deepseek-ai/dsh-session`, JSONL session logs, SessionStore, session projections, session forks, rename, archive. | **YES** | Read session metadata for desktop dashboards, session search, and history tracking. |
| **Filesystem & Project Tools** | `FULL` | `dsh-fs-local`, `dsh-tool-fs`, `dsh-tool-str-replace-editor`, `host.pickDirectory`, `host.openPath`. | **YES** | Bridge to native Linux file dialogs (`rfd`), directory pickers, and desktop file managers (`xdg-open`). |
| **Terminal & Tool Execution** | `FULL` | `@deepseek-ai/dsh-tool-bash`, `@deepseek-ai/dsh-terminal-bash`, `@deepseek-ai/dsh-jobs-local`. | **YES** | Supervised by Rust process manager; environment scrubbed before child process execution. |
| **MCP (Model Context Protocol)** | `FULL` | `@deepseek-ai/dsh-mcp-client` supporting `stdio` and `streamable-http` transports with dynamic tool naming (`mcp__<server>__<tool>`). | **YES** | Allow configuring MCP servers via desktop UI and mounting into DSH profiles. |
| **Process Sandboxing (Linux)** | `PARTIAL` | `@deepseek-ai/dsh-sandbox-local` wraps shell execution with `bwrap` (Bubblewrap) or Landlock (`danger-full-access`, `read-only`, `workspace-write`). | **PARTIAL** | Upstream sandbox covers bash execution. Wrapper adds project-level staging workspaces, dependency isolation, diff reviews, and test gating. |
| **DSH Version & Runtime Isolation** | `ABSENT` | Upstream relies on global/workspace npm/pnpm installation; no multi-version manager. | **NO** | Implement Rust `RuntimeManager` maintaining isolated versions in `~/.local/share/deepseek-harness-linux/runtime/versions/`. |
| **Atomic Updates & Rollback** | `ABSENT` | Upstream has no self-updater or rollback mechanism. | **NO** | Implement Rust `UpdateManager` (staging directory, integrity validation, smoke test, atomic symlink swap, automatic rollback on boot failure). |
| **Update Channels** | `ABSENT` | Upstream publishes npm dist-tags (`latest`, `next`, specific version tags). | **NO** | Implement channel resolver (`Stable` -> `latest`, `Release Candidate` -> `next`, `Development` -> specific version). |
| **Native Linux Desktop Integration** | `ABSENT` | Upstream is a Node CLI / browser application. | **NO** | Implement Tauri 2 native desktop app: system tray, desktop notifications, window lifecycle, single instance, `.desktop` launcher, RPM/AppImage packaging. |
| **Secure Keyring Secret Storage** | `ABSENT` | Upstream relies on environment variables or flat yaml files. | **NO** | Implement Rust native secret store (`secret-service` / Linux Keyring with AES-GCM encrypted fallback). |
| **Temporary Safe Test Workspaces** | `ABSENT` | Upstream edits files directly in place in the current working directory. | **NO** | Implement temporary staging workspaces, environment preservation (venv / node_modules / cargo target handling), and test execution. |
| **Test-Gated Change Application** | `ABSENT` | Upstream does not gate filesystem commits on automated tests or diff approvals. | **NO** | Implement diff generator, test execution runner, and interactive user approval gate before writing to user project. |
| **Project Snapshots & VCS Integration** | `ABSENT` | Upstream relies on external git repository state. | **NO** | Implement Git status detection, automatic pre-change snapshots, snapshot browser, diff, and rollback manager in SQLite + disk. |
| **Health Monitoring & Auto-Recovery** | `PARTIAL` | Upstream has internal process shutdown hooks and unhandled rejection guards. | **PARTIAL** | Rust process monitor with crash detection, health polling endpoint, exponential backoff restart limits, and port conflict resolution. |
| **Desktop Settings & First-Run Setup** | `ABSENT` | Upstream requires CLI commands, manual npm installs, and raw yaml editing. | **NO** | Build first-run onboarding wizard, comprehensive settings UI, and diagnostic bundle exporter. |

---

## Strategic Summary

1. **Maximum Upstream Reuse**:
   - Upstream DSH contains 50+ battle-tested TypeScript packages covering LLM streaming, token metering, tools, MCP bridges, session logging, bash execution, and the full browser UI.
   - We reuse the official `@deepseek-ai/dsh` runtime package directly via Node without forking or reimplementing DSH logic.

2. **Native Desktop Superpowers via Rust & Tauri 2**:
   - The desktop shell owns runtime isolation, atomic updates, Secret Service integration, Bubblewrap/sandbox boundaries, Git checkpoints, and Linux desktop integration.
   - The frontend integrates the official DSH Web application inside a native desktop frame with custom panels for Provider Management, Updates, Sandbox Workspaces, Snapshots, and System Settings.
