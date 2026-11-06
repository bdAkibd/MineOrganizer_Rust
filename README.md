# MineOrganizer 2.0 (Rust & Svelte 5)

> High-performance transactional Minecraft mod manager with zero-overhead virtualization and automated crash diagnostics.

Built with **Rust**, **Tauri v2**, **Svelte 5** (Runes), and **Tailwind CSS**.

---

## ✨ Features

- ⚡ **Zero-Overhead Mod Virtualization**: Switch between massive mod profiles in under a second using transactional symbolic links and junctions with automatic rollback on error.
- 📦 **Multithreaded Mod Scanning**: Parallel metadata extraction (`fabric.mod.json`, `quilt.mod.json`, `mods.toml`, `neoforge.mods.toml`, `mcmod.info`, Bukkit `plugin.yml`) across all CPU cores via Rayon.
- 🌐 **In-App Modrinth Marketplace**: Search mods, browse versions, and install directly into your mod library with one click.
- 🩺 **Automated Crash & Log Diagnostics**: Instant root-cause identification for Out of Memory, Java class mismatch (8–25), Mixin transformer conflicts, missing dependencies, duplicate mod IDs, and GPU/OpenGL issues.
- 🎮 **Universal Launcher Support**: Auto-detects Prism Launcher, official Minecraft Launcher / TLauncher (`.minecraft`), Legacy Launcher, CurseForge instances, and custom directories.
- 🌍 **Internationalization (i18n)**: 11 built-in languages (`en`, `ru`, `de`, `es`, `fr`, `uk`, `it`, `zh_CN`, `zh_TW`, `ja`, `pt`).
- 🎨 **Vortex-Inspired Dark Theme**: Deep dark palette designed for desktop mod management with smooth transitions.

---

## 🛠️ Development & Building

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (1.78+)
- Windows: Visual Studio C++ Build Tools

### Running in Development

```bash
# Install frontend dependencies
npm install

# Run the app in development mode
npm run tauri dev
```

### Running Tests

```bash
# Run Rust unit and integration tests (22 test suite)
cargo test --manifest-path src-tauri/Cargo.toml

# Build frontend production bundle
npm run build
```

### Building Installers (NSIS / MSI)

```bash
npm run tauri build
```

The resulting installers (`.exe` setup and `.msi`) will be generated in `src-tauri/target/release/bundle/nsis/` and `src-tauri/target/release/bundle/msi/`.

---

## 🚀 Automated GitHub Release

A GitHub Actions workflow is provided at `.github/workflows/release.yml`.
To trigger an automated Windows release build with NSIS/MSI installers attached:

```bash
git tag v2.0.0
git push origin v2.0.0
```

---

## 📄 License

MIT License.
