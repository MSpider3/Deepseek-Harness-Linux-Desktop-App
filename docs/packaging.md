# Linux Packaging Guide (RPM, AppImage, Binary Distribution)

This guide documents the build and packaging pipelines for **DeepSeek Harness Linux** targeting modern Linux distributions (Fedora, RHEL, CentOS Stream, Ubuntu, Debian, Arch).

---

## 1. Native Distribution Targets

- **AppImage**: Portable, self-contained single-executable bundle suitable for any Linux distribution.
- **RPM**: Native package for Fedora / Red Hat Enterprise Linux / openSUSE with automatic system menu integration.
- **Deb**: Native package for Debian / Ubuntu / Pop!_OS.
- **Standalone Tarball**: Portable archive with launcher script.

---

## 2. Build Scripts

### 2.1 Build AppImage / Linux Bundles via Tauri CLI
```bash
# Build production React frontend
npm --prefix app/frontend run build

# Build Linux native Tauri bundle
cargo tauri build --manifest-path app/src-tauri/Cargo.toml
```
The resulting binaries and packages will be located in:
`app/src-tauri/target/release/bundle/`

### 2.2 Automated Packaging Helper (`scripts/build_package.sh`)
```bash
#!/usr/bin/env bash
set -euo pipefail

echo "==> Building React Web Assets..."
npm --prefix app/frontend run build

echo "==> Compiling Tauri 2 Rust Desktop Application..."
cargo build --release --manifest-path app/src-tauri/Cargo.toml

DIST_DIR="dist/deepseek-harness-linux"
mkdir -p "$DIST_DIR/bin"
mkdir -p "$DIST_DIR/share/applications"
mkdir -p "$DIST_DIR/share/icons/hicolor/512x512/apps"

cp target/release/deepseek-harness-linux "$DIST_DIR/bin/"
cp app/src-tauri/icons/icon.png "$DIST_DIR/share/icons/hicolor/512x512/apps/ai.deepseek.harness.linux.png"

cat << 'EOF' > "$DIST_DIR/share/applications/ai.deepseek.harness.linux.desktop"
[Desktop Entry]
Name=DeepSeek Harness
Comment=Native Linux Desktop App for DeepSeek Harness
Exec=deepseek-harness-linux
Icon=ai.deepseek.harness.linux
Terminal=false
Type=Application
Categories=Development;IDE;Utility;
StartupNotify=true
EOF

echo "==> Package ready in $DIST_DIR"
```

---

## 3. Desktop Entry Specification

- **File**: `~/.local/share/applications/ai.deepseek.harness.linux.desktop`
- **Icon**: `ai.deepseek.harness.linux.png` installed in `~/.local/share/icons/hicolor/512x512/apps/`
- **MIME Types / Categories**: `Development;IDE;Utility;`
