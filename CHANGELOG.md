# Changelog (`design-os-studio`)

All notable changes to `design-os-studio` (**DESIGN:OS Studio**) are documented in this file.

## [0.2.0] - 2026-10-15

### Added
- **Cross-Platform Installers (`DIR-22`)**: Added one-command release builder (`scripts/build-release.sh`) producing macOS `.dmg` (`designos.app` + `install-cli.command`) + `.tar.gz`, Windows NSIS `-setup.exe` + `.zip` (`install.ps1`/`uninstall.ps1`), Linux Debian `.deb` + portable `.tar.gz` (`install.sh`), `dist/release-manifest.json`, and `dist/SHA256SUMS`.
- **Native Agentic PTY Chat UI Wrapper (`DIR-21`)**: Added structured User/Assistant chat turn bubbles (`AgentChatTurn` / `AgentChatRole`) inside the `Agents` Left Rail (`RailItem::Agents`) wrapping live `portable-pty` + `vt100` sessions for Claude Code (`claude`), Codex (`codex`), and Antigravity CLI (`agy`) with inline `NodeSpec` detection and 1-click Canvas apply.
- **100% Unified Phosphor Icons System (`DIR-19`)**: Unified all UI3 chrome, toolbars, left rail, inspector, and Spec Reader surfaces on official Phosphor Icons (`@phosphor-icons/core` `256x256` SVG rasterized via `resvg`).
- **11-Stage `PRODUCT:OS Spec Driven v2` Reader (`DIR-18`)**: Full in-app Spec Reader (`--panel specs` / `--rail specs`) rendering `US-01..US-15` User Stories, Gherkin `Given/When/Then` scenarios, and `g1..g5` verification gates.
- **1:1 Figma `126.9.13` UI3 & `.fig` v106 Parity (`387 / 387 — 100.00%`)**: Complete visual chrome, typography, Auto-Layout 5.0, Variables, Components, Prototype flows, Dev Mode codegen, and native `.fig` v106 binary read/write parity verified by `823` automated tests.
- **AI-Ready Repository Delivery & 9-Tier GitHub Pages (`DIR-23`)**: Added `.claude-plugin`, `.codex-plugin`, `.agents/plugins`, `.factory-plugin`, `skills/design-os-studio/SKILL.md`, `/doctor`, `scripts/self_check.py`, 12-section editorial `README.md`, and 9-tier `docs/index.html` with 15 visual proof captures.
