# MineOrganizer (MOrg)

> A modern desktop Minecraft mod and environment manager with virtualized libraries, complete modpack sharing, and automated diagnostics.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows-informational)](#)
[![Internationalization](https://img.shields.io/badge/Languages-11%20Locales-brightgreen)](#)

---

## 📖 About MineOrganizer

**MineOrganizer** is a free, open-source desktop application designed to make managing Minecraft mods, game environments, and collections simple, safe, and clean.

Instead of copying hundreds of megabytes of mod files back and forth between different launcher directories, MineOrganizer maintains a centralized mod library and uses safe, transactional filesystem links to connect mods directly to your game instances. Switch between different mod setups, game versions, or mod loaders in under a second with zero disk space wasted.

---

## ✨ Features

- **📦 Centralized Mod Library & Drag-and-Drop Import**:
  - Drag and drop `.jar` mods, `.zip` archives, or `.mrpack` / `.morgpack` bundles directly into the app.
  - Automatic metadata inspection (`fabric.mod.json`, `mods.toml`, `quilt.mod.json`, `neoforge.mods.toml`, `mcmod.info`, Bukkit `plugin.yml`).
  - Disk-persistent caching for instant subsequent launches.

- **⚡ Zero-Overhead Profile Virtualization**:
  - Switch between massive mod profiles instantly using managed symbolic links and directory junctions.
  - Transactional deployment with clear safety preview and one-click rollback.
  - Regular user files and folders are never overwritten.

- **🌐 Integrated Modrinth Catalog**:
  - Search and discover thousands of mods directly inside MineOrganizer.
  - Filter by Minecraft version and loader (Fabric, Forge, NeoForge, Quilt).
  - Page-by-page browsing with direct one-click downloads into your library.
  - Sha1/sha512 hash-based update checker for installed mods.

- **🎒 Complete Modpack Sharing (`.morgpack`)**:
  - Export and import complete modpacks containing mods, configuration files, resource packs, shaders, and world saves.
  - Choose exactly which components and world saves to package.
  - Compatible with standard `.mrpack` modpacks.

- **🩺 Automated Crash & Log Diagnostics**:
  - Automatic detection of `crash-reports/` and `logs/latest.log`.
  - Instant root-cause identification: Out of Memory, Java class mismatches (Java 8–25), Mixin transformer conflicts, missing dependencies, duplicate mod IDs, and GPU/OpenGL issues.
  - Actionable recommended solutions and one-click diagnostics sharing.

- **🎮 Universal Launcher Support**:
  - Automatically detects Prism Launcher, official Minecraft Launcher / TLauncher (`.minecraft`), Legacy Launcher, and custom game directories.

- **🌍 11 Built-in Languages & In-App User Manual**:
  - English, Русский, Deutsch, Español, Français, Italiano, Português, Українська, 日本語, 简体中文, and 繁體中文.
  - Built-in user manual reader accessible directly from the app.

---

## 🚀 Quick Start Guide

### 1. Select or Add an Environment
Upon launch, MineOrganizer automatically detects your installed Minecraft instances. You can also click **Add Folder** in the Environments view to link any custom game directory.

### 2. Populate Your Mod Library
- Drag and drop `.jar` or `.zip` files from your computer directly into the **Mod Library** tab.
- Or click **Modrinth Browser** to find and install popular mods with a single click.

### 3. Create a Profile
Navigate to **Profiles** and click **New Profile**:
- Link it to your target game instance.
- Click **Mods** (or right-click the profile card) to choose which mods, resource packs, and shaders to enable.

### 4. Deploy to Game
Click **Deploy** on the profile card (or click **Deploy to Game** in the top bar):
- Review the planned filesystem links in the virtualization preview.
- Confirm deployment. Your game instance is now fully configured!

### 5. Export and Share
Right-click your profile and select **Export as Modpack (.morgpack)**. Choose whether to include configurations, resource packs, shader packs, and worlds to share your complete build with friends.

---

## 🛠️ Building from Source

### Prerequisites
- Node.js (v18 or higher)
- Rust toolchain (stable)
- Windows: Visual Studio C++ Build Tools

### Running in Development
```bash
# Install frontend dependencies
npm install

# Run the development application
npm run tauri dev
```

### Running Tests
```bash
# Run unit and integration tests
cargo test --manifest-path src-tauri/Cargo.toml

# Validate frontend types
npm run check
```

### Packaging an Installer
```bash
npm run tauri build
```
The compiled installer (`.exe` setup) will be created in `src-tauri/target/release/bundle/nsis/`.

---

## ⚠️ Disclaimer

MineOrganizer is provided **"as is", without warranty of any kind, express or implied**.

The authors and contributors make no representations or warranties regarding the operation, reliability, or fitness of the software for any particular purpose. While MineOrganizer utilizes transactional and non-destructive filesystem links, users are strongly advised to maintain backups of important worlds, game settings, and mod files before applying virtualization links or restoring configuration archives.

---

## ⚖️ Minecraft Trademark Notice

Minecraft and related assets, names, and logos are trademarks of Mojang AB and Microsoft Corporation. MineOrganizer is an independent community open-source project and is not affiliated with, endorsed by, or sponsored by Mojang AB or Microsoft.

---

## 📄 License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for complete terms and copyright notices.
