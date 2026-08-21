# Security Architecture & Secret Management

## 1. Core Security Doctrine

1. **Least Privilege**: The application runs entirely with user-level privileges. It never prompts for `sudo` or requires root execution.
2. **Never Plaintext Secrets**: API keys, tokens, and authorization headers are never written in plaintext to configuration files, database tables, or logs.
3. **Environment Scrubbing**: Sensitive parent environment variables are sanitized before spawning child processes.
4. **Log Redaction**: All stdout, stderr, and system logs pass through regex-based redaction filters (`Redactor::sanitize`) before being stored or displayed in the UI.

---

## 2. Linux Secret Store Implementation

```
┌─────────────────────────────────────────────────────────────┐
│                      Tauri Frontend                         │
└──────────────────────────────┬──────────────────────────────┘
                               │ IPC (Secret identifier ref)
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                   Rust SecretStore                          │
│                                                             │
│  Primary Tier:                                              │
│  - Freedesktop DBus Secret Service API (GNOME Keyring /     │
│    KWallet)                                                 │
│                                                             │
│  Encrypted Fallback Tier:                                   │
│  - PBKDF2-HMAC-SHA1 Key Derivation (10,000 iterations from   │
│    machine-id & user seed)                                  │
│  - AES-256-GCM authenticated encryption                     │
│  - Stored at ~/.local/share/.../data/vault.enc (mode 0600)  │
└──────────────────────────────┬──────────────────────────────┘
                               │ Injected via secure memory
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                 Isolated Child Node Process                 │
│                 (DSH_SECRET_<UUID> in memory)               │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Redaction Engine

The `Redactor` engine actively scrubs:
- `sk-[a-zA-Z0-9_-]{16,}` (OpenAI / DeepSeek / Anthropic API keys)
- `Authorization: Bearer ...` headers
- `password=...`, `secret=...`, `token=...` patterns
- Cookie session secrets

Diagnostic export files (`export_diagnostics`) are guaranteed clean of all private credentials.
