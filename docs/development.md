# Development & Contributor Quickstart

This document provides a quickstart guide for setting up the local development environment. For deep architectural internals and subsystem blueprints, see the **[Developer Architecture Guide](developer-guide.md)** and **[Contributing Guidelines](../CONTRIBUTING.md)**.

---

## 🛠️ Prerequisites

- **Linux Distribution**: Fedora 40+, Ubuntu 22.04+, Debian 12+, Arch Linux
- **Rust Toolchain**: `>= 1.80.0` (`rustc`, `cargo`)
- **Node.js**: `>= 20.0.0` (Node 22 LTS recommended) and `npm`
- **System Libraries**:
  ```bash
  # Debian / Ubuntu / Pop!_OS
  sudo apt install -y libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev libsecret-1-dev libdbus-1-dev build-essential curl wget file libssl-dev

  # Fedora / RHEL
  sudo dnf install -y gtk3-devel webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel libsecret-devel dbus-devel openssl-devel @development-tools
  ```

---

## 🚀 Getting Started

1. **Clone the repository**:
   ```bash
   git clone https://github.com/MSpider3/Deepseek-Harness-Linux-Desktop-App.git
   cd Deepseek-Harness-Linux-Desktop-App
   ```

2. **Install frontend dependencies**:
   ```bash
   npm --prefix app/frontend install
   ```

3. **Run in development mode (with live UI hot-reload)**:
   ```bash
   npm run tauri:dev
   ```

4. **Run test suites**:
   ```bash
   cargo test --manifest-path app/src-tauri/Cargo.toml -- --nocapture
   ```

5. **Build release packages**:
   ```bash
   chmod +x scripts/build_package.sh
   ./scripts/build_package.sh
   ```

---

## 📚 Technical Documentation Index

- **[Developer Architecture & Subsystems Blueprint](developer-guide.md)**
- **[System Architecture](architecture.md)**
- **[Security Doctrine & Keyring Store](security.md)**
- **[Sandbox & Safe Staging Engine](sandbox.md)**
- **[Provider Hot-Reload Syncing](provider-system.md)**
- **[Multi-Version Runtime Manager](update-system.md)**
- **[Packaging Pipeline & Freedesktop Specs](packaging.md)**
- **[Upstream Compatibility Matrix](upstream-capability-matrix.md)**
