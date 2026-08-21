# Upstream Compatibility & Tracking Guide

## Upstream Target
- **Official Repository**: `https://github.com/deepseek-ai/deepseek-harness`
- **Official Package**: `@deepseek-ai/dsh`
- **Current Audited Upstream Version**: `0.1.0-rc.8` (`latest`: `0.1.0-rc.7`, `next`: `0.1.0-rc.8`)

---

## 1. Upstream Architecture & Contracts

### 1.1 Profile Boot & Layered Composition
Upstream DSH uses the **Cordis** microkernel framework. Configuration is composed as an ordered stack of patch layers:
1. `dsh.profile.bundles`: Base bundles (`@deepseek-ai/dsh-base`, `@deepseek-ai/dsh-web-app`, etc.)
2. Profile root: `cordis.yml` (empty entry list)
3. Profile patch: `cordis.patch.yml`
4. Home patch: `$DSH_HOME/cordis.patch.yml`
5. Overlays: `--patch <path>`

**Wrapper Guarantee**: DeepSeek Harness Linux writes customizations to `$DSH_HOME/cordis.patch.yml` and launches with isolated `$DSH_HOME`, leaving all upstream code untouched.

### 1.2 Web Profile Arguments
`dsh web` is parsed by `apps/cli/src/args.ts` and accepts:
- `--host <ip>` (e.g. `127.0.0.1`)
- `--port <port>` (e.g. `5180` or `0` for dynamic)
- `--no-open` (suppresses external browser launch)
- `--patch <file>` (custom patch overlays)

**Wrapper Guarantee**: The Rust supervisor invokes `node <dsh-bin> web --no-open --port <port> --host 127.0.0.1` and captures the loopback endpoint for embedding.

### 1.3 LLM Seam & Multi-Provider Compatibility
- `@deepseek-ai/dsh-llm`: Core vocabulary (`Message`, `StreamChunk`, `LlmAdapter`).
- `@deepseek-ai/dsh-llm-deepseek`: Direct fetch SSE client for DeepSeek Official.
- `@deepseek-ai/dsh-llm-pi-ai`: Multi-provider router powered by `@earendil-works/pi-ai`, supporting OpenAI, Anthropic, Gemini, OpenRouter, Ollama, and OpenAI-compatible gateways.
- Key reference: `apiKeyEnv: <ENV_VAR>` resolves credentials dynamically without embedding plaintext in configs.

---

## 2. Upstream Tracking Policy

1. **Automated Dist-Tag Queries**:
   - The desktop updater queries `https://registry.npmjs.org/@deepseek-ai%2Fdsh` for `latest` and `next` tags.
2. **Smoke Testing Before Activation**:
   - Every candidate release is installed in an isolated staging folder and verified with `dsh --version` before switching the active symlink.
3. **Fail-Safe Rollback**:
   - If a new upstream release fails to boot or reach healthy status within 15 seconds, the application automatically rolls back the `current` symlink to `previous`.
