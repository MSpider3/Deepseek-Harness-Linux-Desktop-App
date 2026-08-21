#!/usr/bin/env bash
set -euo pipefail

echo "======================================================="
echo " DeepSeek Harness Linux — Packaging Pipeline"
echo "======================================================="

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "==> Step 1: Building Frontend Assets (Vite & TypeScript)..."
npm --prefix app/frontend run build

echo "==> Step 2: Running Test Suites..."
cargo test --manifest-path app/src-tauri/Cargo.toml

echo "==> Step 3: Compiling Production Desktop Application..."
cargo build --release --manifest-path app/src-tauri/Cargo.toml

DIST_DIR="$ROOT_DIR/dist/deepseek-harness-linux-0.1.0-linux-x86_64"
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR/bin"
mkdir -p "$DIST_DIR/share/applications"
mkdir -p "$DIST_DIR/share/icons/hicolor/512x512/apps"
mkdir -p "$DIST_DIR/share/icons/hicolor/128x128/apps"
mkdir -p "$DIST_DIR/share/icons/hicolor/32x32/apps"

cp target/release/deepseek-harness-linux "$DIST_DIR/bin/"
cp app/src-tauri/icons/icon.png "$DIST_DIR/share/icons/hicolor/512x512/apps/ai.deepseek.harness.linux.png"
cp app/src-tauri/icons/128x128.png "$DIST_DIR/share/icons/hicolor/128x128/apps/ai.deepseek.harness.linux.png"
cp app/src-tauri/icons/32x32.png "$DIST_DIR/share/icons/hicolor/32x32/apps/ai.deepseek.harness.linux.png"

cat << 'EOF' > "$DIST_DIR/share/applications/ai.deepseek.harness.linux.desktop"
[Desktop Entry]
Name=DeepSeek Harness Linux
GenericName=AI Coding Harness & Workspace
Comment=Native Linux Desktop App for official DeepSeek Harness
Exec=deepseek-harness-linux
Icon=ai.deepseek.harness.linux
Terminal=false
Type=Application
Categories=Development;IDE;Utility;
StartupNotify=true
Keywords=deepseek;ai;coding;agent;harness;
EOF

cat << 'EOF' > "$DIST_DIR/install.sh"
#!/usr/bin/env bash
set -e
PREFIX="${PREFIX:-$HOME/.local}"
echo "Installing DeepSeek Harness Linux to $PREFIX..."
mkdir -p "$PREFIX/bin" "$PREFIX/share/applications" "$PREFIX/share/icons/hicolor/512x512/apps"
cp bin/deepseek-harness-linux "$PREFIX/bin/"
cp share/icons/hicolor/512x512/apps/ai.deepseek.harness.linux.png "$PREFIX/share/icons/hicolor/512x512/apps/"
cp share/applications/ai.deepseek.harness.linux.desktop "$PREFIX/share/applications/"
echo "Installation complete! You can run 'deepseek-harness-linux' or launch it from your desktop app menu."
EOF
chmod +x "$DIST_DIR/install.sh"

echo "==> Creating Portable Distribution Tarball..."
cd "$ROOT_DIR/dist"
tar -czvf "deepseek-harness-linux-0.1.0-linux-x86_64.tar.gz" "deepseek-harness-linux-0.1.0-linux-x86_64"

echo "======================================================="
echo " Package successfully built at:"
echo " dist/deepseek-harness-linux-0.1.0-linux-x86_64.tar.gz"
echo "======================================================="
