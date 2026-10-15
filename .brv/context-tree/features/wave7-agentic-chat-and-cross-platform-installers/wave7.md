# Wave 7: Pure Agentic Chat Wrapper over PTY Shell (`DIR-21`) & Final Cross-Platform Installers (`DIR-22`)

- **Date**: 2026-10-15
- **Directives**: `DIR-21`, `DIR-22`
- **Epics / Stories / Rules**: `EP-15`, `EP-16` · `US-15` · `REQ-18`, `REQ-19` · `BR-23`, `BR-24` · `D18`, `D19`

## 1. Pure Chat UI Wrapper Over Native PTY Terminal Shell (`DIR-21`)
- **File**: `crates/designos-app/src/terminal.rs`, `crates/designos-app/src/cli.rs`, `crates/designos-app/src/shell.rs`
- **Architecture**:
  - `AgentChatRole` (`User`, `Assistant`) and `AgentChatTurn` (`id`, `role`, `runtime`, `text`, `shell_cmd`, `tab_id`, `log_start_offset`, `log_end_offset`, `extracted_spec_json`, `applied_to_canvas`) in `TerminalState`.
  - `28px` CLI Runtime Selector Bar at the top of the `Agents` panel (`S13`): `"agents-runtime-claude"` (`Claude`), `"agents-runtime-codex"` (`Codex`), `"agents-runtime-agy"` (`Antigravity`), and `"agents-runtime-pty-toggle"` (`Shell`).
  - `submit_chat_prompt` executes prompts inside the active `portable-pty` shell tab (`$SHELL -l`) wrapped by `DESIGNOS_CHAT_BEGIN_<id>` / `DESIGNOS_CHAT_END_<id>` markers, and `sync_chat_from_pty` + `strip_ansi_codes` stream clean stdout directly into the `Assistant` chat bubble (`render_agent_chat_thread`) with 1-click `"Apply"` (`"agents-chat-apply-spec"`) for `NodeSpec` output.
  - CLI screenshot flag `--agents-chat` (`qa_wave7_agents_chat.png`) and unit test `terminal::tests::agents_chat_wrapper_executes_in_pty_shell_and_streams_clean_output`.

## 2. Final Cross-Platform Build & User-Installable Packaging Workflow (`DIR-22`)
- **All 4 Binaries Bundled**: `designos-studio`, `designos`, `designos-mcp`, `designos-diff`.
- **macOS (`scripts/package-macos.sh`, `packaging/macos/`)**:
  - Supports both Universal (`arm64` + `x86_64` via `lipo`) and single-arch native host builds.
  - Lowercase `design:os studio` display name in `Info.plist.in` (`os.design.studio`, `.designos`, `.fig`, `.spec.json`), ad-hoc `codesign --verify --deep --strict`, `install-cli.command` DMG helper (clears Gatekeeper quarantine & symlinks CLI binaries to `~/.local/bin`), producing `dist/designos-<version>-macos.dmg` and `.tar.gz`.
- **Windows (`packaging/windows/`, `scripts/package.sh`)**:
  - `packaging/windows/installer.nsi` (NSIS Modern UI 2 per-user `-setup.exe` installer under `$LOCALAPPDATA\Programs\designos`, Start Menu & Desktop shortcuts, `.designos`/`.fig` associations, `Uninstall.exe`).
  - `packaging/windows/install.ps1` & `uninstall.ps1` bundled inside `.zip`.
- **Linux (`packaging/linux/`, `scripts/package.sh`)**:
  - `dpkg-deb --root-owner-group` builds `dist/designos_<version>_<arch>.deb` (`amd64`/`arm64`) alongside `.tar.gz` with `install.sh`, `uninstall.sh`, `AppRun`, `designos.desktop`, `designos-mime.xml`, and hicolor icons.
- **Unified Orchestrator & CI (`scripts/build-release.sh`, `.github/workflows/release.yml`, `docs/install.md`)**:
  - Generates `dist/release-manifest.json` + `dist/SHA256SUMS` compatible with `updater::ReleaseManifest`.
