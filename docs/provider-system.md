# Provider & Model Management System

## 1. Supported Providers

| Provider | Endpoint Pattern | Default Models | Capabilities |
| :--- | :--- | :--- | :--- |
| **DeepSeek** | `https://api.deepseek.com` | `deepseek-chat` (V3), `deepseek-reasoner` (R1) | Reasoning, Streaming, Tools |
| **OpenAI** | `https://api.openai.com/v1` | `gpt-4o`, `gpt-4o-mini`, `o1`, `o3-mini` | Vision, Reasoning, Tools |
| **Anthropic** | `https://api.anthropic.com` | `claude-3-7-sonnet`, `claude-3-5-sonnet` | Hybrid Thinking, Vision, Tools |
| **Google Gemini** | `https://generativelanguage.googleapis.com/v1beta/openai/` | `gemini-2.5-pro`, `gemini-2.5-flash` | Vision, Tools |
| **OpenRouter** | `https://openrouter.ai/api/v1` | Dynamic via `GET /models` | Multi-vendor Routing |
| **Local Ollama** | `http://localhost:11434/v1` | Local models via `GET /models` | Offline / Self-hosted |
| **Custom Gateway** | User defined | User defined / discovered | OpenAI-compatible endpoints |

---

## 2. Configuration Synchronization

When DSH launches, `ProviderConfigSyncer` writes:
`$DSH_HOME/cordis.patch.yml`:
```yaml
- id: llm
  name: '@deepseek-ai/dsh-llm-pi-ai'
  config:
    providers:
      deepseek:
        apiKeyEnv: DEEPSEEK_API_KEY
        baseURL: https://api.deepseek.com
        displayName: DeepSeek
```
And passes secrets in child process memory (`DEEPSEEK_API_KEY=sk-...`), avoiding plaintext disk exposure.
