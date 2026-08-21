# Contributing to DeepSeek Harness Linux

Thank you for your interest in contributing to **DeepSeek Harness Linux**! We welcome bug reports, feature suggestions, documentation enhancements, and code contributions from the community.

> [!NOTE]
> DeepSeek Harness Linux is a native Linux desktop shell for the official upstream `@deepseek-ai/dsh` runtime. We follow a strict zero-fork doctrine to preserve upstream compatibility while providing native Linux desktop performance, encrypted keyring security, safe sandboxing, and atomic runtime updates.

---

## 🛠️ Development Setup

### System Prerequisites
- **Operating System**: Modern Linux distribution (Fedora 40+, Ubuntu 22.04+, Arch Linux, Debian 12+)
- **Rust Toolchain**: `>= 1.80.0` (with `cargo`)
- **Node.js**: `>= 20.0.0` (Node.js 22 LTS recommended) and `npm`
- **Linux Development Headers**:
  ```bash
  # Debian / Ubuntu / Pop!_OS
  sudo apt install -y libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev libsecret-1-dev libdbus-1-dev build-essential curl wget file libssl-dev

  # Fedora / RHEL
  sudo dnf install -y gtk3-devel webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel libsecret-devel dbus-devel openssl-devel @development-tools

  # Arch Linux
  sudo pacman -S --needed gtk3 webkit2gtk-4.1 libappindicator-gtk3 librsvg libsecret dbus base-devel openssl
  ```

### Getting Started
1. **Clone the Repository**:
   ```bash
   git clone https://github.com/MSpider3/Deepseek-Harness-Linux-Desktop-App.git
   cd Deepseek-Harness-Linux-Desktop-App
   ```

2. **Install Frontend Dependencies**:
   ```bash
   npm --prefix app/frontend install
   ```

3. **Run the Development Server**:
   ```bash
   npm run tauri:dev
   ```

---

## 🧪 Testing & Verification

Before submitting any code changes or opening a pull request, ensure all tests pass cleanly:

### 1. Rust Backend Unit & Integration Tests
```bash
cargo test --manifest-path app/src-tauri/Cargo.toml -- --nocapture
```

### 2. Frontend Build & Typecheck
```bash
npm --prefix app/frontend run build
```

### 3. Packaging Pipeline Verification
```bash
chmod +x scripts/build_package.sh
./scripts/build_package.sh
```

---

## 📐 Architecture & Coding Guidelines

1. **Security First**:
   - Never write plain-text secrets, bearer tokens, or API keys to unencrypted files.
   - Use the `SecretStore` abstraction (`app/src-tauri/src/security/keyring.rs`) to store credentials via Linux Secret Service (GNOME Keyring / KWallet) with PBKDF2 + AES-256-GCM vault fallback.
   - Always sanitize output with the Redaction engine (`app/src-tauri/src/security/redaction.rs`).

2. **Surgical Changes**:
   - Focus pull requests on specific bugs or features.
   - Match existing Rust and React coding style.
   - Remove unused imports or variables introduced by your modifications.

3. **No Unofficial Upstream Forks**:
   - The desktop shell interacts with official `@deepseek-ai/dsh` packages via Node child processes, loopback port routing, and runtime configuration sync (`$DSH_HOME/.credentials.yaml`, `$DSH_HOME/settings.yaml`, and `$DSH_HOME/cordis.patch.yml`).
   - Do not bundle modified forks of `@deepseek-ai/dsh`.

---

## 🚀 Submitting a Pull Request

1. Create a descriptive feature branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```
2. Commit your changes with a clear commit message:
   ```bash
   git commit -m "feat(runtime): add telemetry-free update inspector"
   ```
3. Push to your fork and open a Pull Request against the `main` branch at:
   [https://github.com/MSpider3/Deepseek-Harness-Linux-Desktop-App](https://github.com/MSpider3/Deepseek-Harness-Linux-Desktop-App)
4. Ensure continuous integration (CI) tests pass on GitHub Actions.

---

## 💬 Community & Support

If you have questions, encounter an issue, or want to discuss improvements, please open an issue in the [GitHub Issue Tracker](https://github.com/MSpider3/Deepseek-Harness-Linux-Desktop-App/issues).
