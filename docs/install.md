# Installing **design:os studio**

Download the latest installer for your platform from [GitHub Releases](https://github.com/jangtrinh/design-os-studio/releases) or build a local release package directly from source.

Every release ships all **4 binaries**:
- `designos-studio` — Native GPU (`wgpu` Metal / DirectX 12 / Vulkan) + CPU (`tiny-skia`) Studio GUI
- `designos` — GUI + headless CLI renderer, exporter, and batch processor
- `designos-mcp` — Model Context Protocol (MCP) server for Claude Code, Codex, and Antigravity
- `designos-diff` — Visual & `.fig` structural diff CLI tool

---

## 1. macOS (Apple Silicon & Intel)

### Option A: Drag-and-Drop `.dmg` Installer (Recommended)

1. Download `designos-<version>-macos.dmg` from [GitHub Releases](https://github.com/jangtrinh/design-os-studio/releases).
2. Double-click the `.dmg` and drag **`designos.app`** into **`Applications`**.
3. *(Optional — CLI & Gatekeeper Setup)* Double-click **`install-cli.command`** inside the mounted DMG window. This script:
   - Clears the macOS Gatekeeper quarantine flag (`xattr -dr com.apple.quarantine /Applications/designos.app`)
   - Symlinks `designos-studio`, `designos`, `designos-mcp`, and `designos-diff` into `~/.local/bin` (or `/usr/local/bin`)

If you launch `designos.app` without running `install-cli.command`, clear the quarantine attribute once in Terminal:

```bash
xattr -dr com.apple.quarantine /Applications/designos.app
```

### Option B: Tarball Archive (`.tar.gz`)

```bash
tar -xzf designos-<version>-macos-universal.tar.gz
cp -R designos-<version>-macos-universal/designos.app /Applications/
./designos-<version>-macos-universal/install-cli.command
```

---

## 2. Windows (x64)

### Option A: NSIS Setup Installer (`-setup.exe`, Recommended)

1. Download `designos-<version>-x86_64-pc-windows-msvc-setup.exe`.
2. Run the installer (if Windows SmartScreen prompts, click **More info → Run anyway**).
3. The installer performs a **per-user installation** under `%LOCALAPPDATA%\Programs\designos` (no Administrator UAC prompt required):
   - Installs `designos-studio.exe`, `designos.exe`, `designos-mcp.exe`, and `designos-diff.exe`
   - Creates Start Menu (`designos studio`) and Desktop shortcuts
   - Associates `.designos` and `.fig` files with **design:os studio**
   - Registers an uninstaller in Windows **Installed Apps / Add or Remove Programs**

### Option B: Portable `.zip` + PowerShell Installer (`install.ps1`)

1. Download and extract `designos-<version>-x86_64-pc-windows-msvc.zip`.
2. Run the bundled PowerShell installer to copy the 4 binaries to `%LOCALAPPDATA%\Programs\designos\bin`, add them to your User `PATH`, create shortcuts, and register `.designos` / `.fig` file associations:

```powershell
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

To uninstall later:

```powershell
powershell -ExecutionPolicy Bypass -File "$env:LOCALAPPDATA\Programs\designos\uninstall.ps1"
```

---

## 3. Linux (x64 / ARM64)

### Option A: Ubuntu / Debian Package (`.deb`, Recommended on Debian-based distros)

```bash
sudo apt install ./designos_<version>_amd64.deb
```

This installs `/usr/bin/{designos-studio,designos,designos-mcp,designos-diff}`, the desktop launcher (`/usr/share/applications/designos.desktop`), hicolor icons, and `.designos` / `.fig` MIME types.

### Option B: Universal Linux Tarball (`install.sh`, No Root Required)

```bash
tar -xzf designos-<version>-x86_64-unknown-linux-gnu.tar.gz
cd designos-<version>-x86_64-unknown-linux-gnu
./install.sh
```

By default, `./install.sh` installs under `~/.local/bin` and `~/.local/share` (override with `PREFIX=/usr/local ./install.sh`). To remove:

```bash
./uninstall.sh
```

---

## 4. Building & Packaging Releases Locally (`scripts/build-release.sh`)

To compile all 4 release binaries (`designos-studio`, `designos`, `designos-mcp`, `designos-diff`) and build platform installers + `dist/release-manifest.json` + `dist/SHA256SUMS` in one command:

```bash
# Build & package for your current host OS (e.g., macOS .dmg + .tar.gz, Linux .deb + .tar.gz, Windows -setup.exe + .zip)
./scripts/build-release.sh 0.2.0 --local-host

# Build & package a Universal (arm64 + x86_64) macOS release
./scripts/build-release.sh 0.2.0 --universal-macos

# Package already-built release binaries in target/release without recompiling
./scripts/build-release.sh 0.2.0 --local-host --skip-build
```

Outputs are written to `dist/`:
- `dist/designos-<version>-macos.dmg` & `dist/designos-<version>-macos-*.tar.gz` (macOS)
- `dist/designos-<version>-<target>-setup.exe` & `dist/designos-<version>-<target>.zip` (Windows)
- `dist/designos_<version>_<arch>.deb` & `dist/designos-<version>-<target>.tar.gz` (Linux)
- `dist/release-manifest.json` (compatible with the Layer-1 Atomic Self-Updater in `updater.rs`)
- `dist/SHA256SUMS`

---

## 5. GPU Rendering

**design:os studio** renders with `wgpu` (Metal on macOS, DirectX 12 / Vulkan on Windows, Vulkan / GL on Linux) and automatically falls back to the deterministic `tiny-skia` CPU renderer when needed. Pass `--gpu` on the CLI to force GPU rendering in headless `--screenshot` mode.
