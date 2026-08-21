# Atomic DSH Update & Rollback System

## 1. Directory Structure

```
~/.local/share/deepseek-harness-linux/runtime/
├── versions/
│   ├── 0.1.0-rc.7/
│   └── 0.1.0-rc.8/
├── current -> versions/0.1.0-rc.8   (Active atomic symlink)
├── previous -> versions/0.1.0-rc.7  (Rollback target symlink)
├── staging/                         (Temporary download and smoke testing)
└── downloads/
```

---

## 2. Atomic Update Workflow

1. **Check for Updates**: Queries npm registry for `@deepseek-ai/dsh` dist-tags (`latest` for Stable, `next` for RC, versions map for Dev).
2. **Staging Download**: Installs candidate into `runtime/staging/staging_<version>/`.
3. **Smoke Test**: Executes `node <staging>/node_modules/@deepseek-ai/dsh/lib/bin.js --version` to confirm exit code 0.
4. **Move to Versions**: Renames staging directory into `runtime/versions/<version>/`.
5. **Atomic Symlink Swap**:
   - Updates `previous` -> prior active version.
   - Symlinks `.tmp_current_<uuid>` -> `versions/<version>`.
   - Renames `.tmp_current_<uuid>` -> `current` (POSIX atomic rename).
6. **Liveness Verification**: Starts candidate DSH process; if health check fails within 15 seconds, automatically swaps `current` back to `previous`.

---

## 3. Update Channels

- **Stable**: Follows npm tag `latest` (recommended for production).
- **Release Candidate (RC)**: Follows npm tag `next` (recommended for testing newest DSH capabilities).
- **Development**: Tracks newest semver release or specific chosen version.
