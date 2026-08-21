# Sandbox & Safe Testing Architecture

## 1. Overview

When DeepSeek Harness agents make code changes, executing untested modifications directly on the user's workspace risks breaking builds or corrupting files.

**DeepSeek Harness Linux** provides a **Test-Gated Safe Staging Pipeline**:

```
Original Project (~/Projects/my-app/)
  │
  ▼ [1. Create Staging Workspace]
Staging Workspace (~/.local/share/.../sandbox/workspaces/<id>/)
  │
  ▼ [2. Agent Modifies Code]
  │
  ▼ [3. Run Automated Tests (npm test / pytest / cargo test)]
  ├── Tests FAIL ──► Discard or prompt agent to fix
  │
  └── Tests PASS
        │
        ▼ [4. Compute Unified Diff (similar engine)]
        │
        ▼ [5. User Visual Diff Review]
        ├── Rejected ──► Discard Staging
        │
        └── Approved
              │
              ▼ [6. Auto-Create Safety Snapshot]
              ▼ [7. Atomic Apply to Original Project]
```

---

## 2. Project-Type Detection & Dependency Preservation

| Project Type | Detection Indicators | Dependency Preservation Strategy |
| :--- | :--- | :--- |
| **Node.js** | `package.json`, `pnpm-lock.yaml` | Symlinks/hardlinks `node_modules` from original project for fast test execution without duplicating disk space. |
| **Python** | `pyproject.toml`, `requirements.txt` | Does **NOT** copy `.venv` (avoids broken absolute paths); runs isolated virtualenv tests. |
| **Rust** | `Cargo.toml`, `Cargo.lock` | Symlinks `target/` cache directory when safe to accelerate compilation. |
| **General** | Standard directories | Pure file copying with exclusion of `.git`, `.venv`, and transient caches. |

---

## 3. Sandboxing Controls

- Confinement policies: `read-only`, `workspace-write`, `danger-full-access`.
- Linux Bubblewrap (`bwrap`) support integration.
- Controlled working directory and environment variables.
