# Stage 11: Developer Handoff & Readiness Packet (`READY.md`)

> **Feature:** `designos-studio-v2` — **`design`:os studio** Native Rust GPU Design Studio, Multi-Tab Projects, 11-Stage Spec-Driven OS, Pure PTY Chat Wrapper (`Claude Code` / `Codex` / `Antigravity CLI`), 5-Pillar Parity Meter, Dual-Layer Auto-Update, Cross-Platform Release Packaging (`macOS` / `Windows` / `Linux`) & Unified Phosphor Icons  
> **Stage:** `11 / handoff`  
> **Gate:** `g5-handoff` (`GATE: APPROVED`)  
> **Updated:** `2026-10-15T14:55:00Z`

---

## 1. End-to-End Traceability Matrix (`REQ` -> `US` -> `Screen` -> `Decision` -> `BR` -> `SCN` -> `Code / Test`)

| REQ ID | User Story | Screen(s) | Decision | Business Rule | Test Scenarios | Primary Rust Module & Test Target |
|---|---|---|---|---|---|---|
| `REQ-01` | `US-01`, `US-06` | `S01` | `D1` | `BR-01` | `SCN-01-01..02` | `crates/designos-fig/` (`.fig` Kiwi v106 lossless R/W) + `crates/designos-app/src/layout.rs` (2D CSS Grid) |
| `REQ-02` | `US-01` | `S01`, `S02`, `S03` | `D1`, `D5` | `BR-01`, `BR-05` | `SCN-01-03` | `crates/designos-app/src/specs.rs` (`apply_node_spec`, `.spec.json` <-> Canvas bridge) |
| `REQ-03` | `US-02`, `US-11` | `S02` | `D2`, `D3` | `BR-02`, `BR-03` | `SCN-02-01` | `crates/designos-app/src/specs.rs` (11-stage Spec Format v2 loader & KPI Strip) |
| `REQ-04` | `US-02`, `US-12` | `S02`, `S12` | `D2`, `D4`, `D14` | `BR-02`, `BR-04`, `BR-19` | `SCN-02-02..03`, `SCN-12-03` | `crates/designos-app/src/specs.rs` (5 Role Views, `format_copy_for_ai`, `approve_gate`, Parity Meter card) |
| `REQ-05` | `US-03`, `US-11`, `US-15` | `S03`, `S11`, `S13` | `D4`, `D5`, `D12`, `D13`, `D18` | `BR-04`, `BR-05`, `BR-16`, `BR-17`, `BR-23` | `SCN-03-01..03`, `SCN-11-01..03`, `SCN-15-01..03` | `crates/designos-app/src/terminal.rs` (`portable-pty` + `vt100` multi-tab agent terminal, `<50ms` hot-apply & Pure Chat UI Wrapper) |
| `REQ-06` | `US-04` | `S04` | `D6` | `BR-06` | `SCN-04-01..03` | `crates/designos-app/src/home/mod.rs` (`Drafts`, `All folders`, `Resources`, `Starred`, `Trash`) |
| `REQ-07` | `US-05`, `US-14` | `S05`, `S06`, `S07`, `S08` | `D6`, `D10`, `D17` | `BR-07`, `BR-08`, `BR-14`, `BR-22` | `SCN-05-01..03`, `SCN-14-01..03` | `crates/designos-app/src/panels/menus.rs`, `cmdk.rs`, `dev/mod.rs`, `variables/mod.rs`, `icons.rs` |
| `REQ-08` | `US-06` | `S01` | `D1`, `D6` | `BR-09` | `SCN-06-01..03` | `crates/designos-app/src/canvas/` (`interact.rs`, `handles.rs`, `grids.rs`, `snap.rs` across 17 `NodeKind`s) |
| `REQ-09` | `US-07` | `S01`, `S07` | `D7` | `BR-10` | `SCN-07-01..03` | `plans/specs/designos-studio-v2/research/` + `crates/designos-fig/tests/fig_roundtrip.rs` |
| `REQ-10` | `US-08` | `S01..S13` | `D8` | `BR-11`, `BR-12` | `SCN-08-01..03` | `crates/designos-app/src/cli.rs` & `main.rs` (`with_visible(false).with_active(false)` background capture) |
| `REQ-11` | `US-09` | `S09`, `S04`, `S01` | `D9` | `BR-13` | `SCN-09-01..03` | `crates/designos-app/src/updater.rs` (`Dual-Layer Auto-Update` + `FigmaSchemaWatchdog`) |
| `REQ-12` | `US-10` | `S10`, `S04`, `S01` | `D10`, `D11` | `BR-14`, `BR-15` | `SCN-10-01..03` | `crates/designos-app/src/tabs.rs` / `app.rs` (`36px` Desktop Tab Strip, per-tab state isolation, **`design`:os studio** wordmark) |
| `REQ-13` | `US-11` | `S11`, `S03`, `S06` | `D12`, `D13` | `BR-16`, `BR-17`, `BR-18` | `SCN-11-01..04` | `crates/designos-app/src/cmdk.rs`, `terminal.rs` (6 Figma AI Actions, 5 CLI Runtimes, `<50ms` Hot-Apply, Gaps A–F) |
| `REQ-14` | `US-12` | `S12`, `S02` | `D14` | `BR-19` | `SCN-12-01..03` | `scripts/figma_parity_meter.py` + `research/parity-scorecard.json` + `crates/designos-app/src/specs.rs` (`--parity-report`) |
| `REQ-15` | `US-13`, `US-15` | `S13`, `S01`, `S06`, `S07` | `D15`, `D18` | `BR-20`, `BR-23` | `SCN-13-01..02`, `SCN-15-01..03` | `crates/designos-app/src/panels/rail.rs`, `left.rs`, `assets.rs`, `toolbar.rs`, `cmdk.rs`, `right.rs`, `inspect.rs`, `prototype.rs`, `properties.rs` (`DIR-17`, `DIR-18`, `DIR-21`) |
| `REQ-16` | `US-13` | `S13`, `S01`, `S04`, `S10` | `D16` | `BR-21` | `SCN-13-03` | `crates/designos-app/src/shell.rs`, `panels/left.rs`, `tabs.rs` (`UF-01..UF-08` 1:1 userflow state machines) |
| `REQ-17` | `US-14` | `S01..S13` | `D17` | `BR-22` | `SCN-14-01..03` | `crates/designos-app/src/icons.rs` + `crates/designos-app/assets/icons/phosphor/*.svg` (`DIR-19` Unified Phosphor Icons System) |
| `REQ-18` | `US-15` | `S13`, `S03`, `S01` | `D18` | `BR-23` | `SCN-15-01..03` | `crates/designos-app/src/terminal.rs` + `crates/designos-app/src/panels/left.rs` (`DIR-21` Pure Chat UI Wrapper over PTY `claude` / `codex` / `agy`, `AgentChatTurn`, `sync_chat_from_pty`, `strip_ansi_codes`) |
| `REQ-19` | `US-09` | `S09`, `S04`, `S01` | `D19`, `D9` | `BR-24`, `BR-13` | `SCN-09-01..03` | `scripts/build-release.sh`, `scripts/package-macos.sh`, `scripts/package-windows.ps1`, `scripts/package-linux.sh`, `packaging/windows/installer.nsi`, `.github/workflows/release.yml` (`DIR-22` Cross-Platform Release Installers) |
| `REQ-20` | `US-08`, `US-15` | `S01..S13` | `D20`, `D8` | `BR-25`, `BR-11`, `BR-12` | `SCN-08-01..03`, `SCN-15-01..03` | `scripts/capture-visual-proof.sh`, `docs/screenshots/*.png` (16 Retina 2x captures), `README.md` (12-Section AI-Ready README), `docs/index.html` (9-Tier GitHub Pages · `DIR-23`) |

---

## 2. Pre-Release & Verification Commands

```bash
# 1. Verify full workspace compilation and unit/integration tests (including specs::tests, icons::tests & terminal::tests)
PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace

# 2. Verify 78 UI contract rows (S01..S13 x 6 states)
grep -E '^\| S(0[1-9]|1[0-3]) \|' plans/specs/designos-studio-v2/ux/ui-contract.md | wc -l

# 3. Verify 5-Pillar Quantitative Figma Parity Scorecard
python3 scripts/figma_parity_meter.py

# 4. Regenerate 16-capture Retina 2x headless visual proof suite (with_visible(false).with_active(false))
./scripts/capture-visual-proof.sh

# 5. Build Cross-Platform Release Installers & SHA256SUMS (macOS .app/.dmg, Windows -setup.exe, Linux .deb/.tar.gz)
./scripts/build-release.sh
```

---

## 3. Handoff Sign-Off Checklist
- [x] All 23 directives (`DIR-01..DIR-23`) traced to `EP-01..EP-17`, `REQ-01..REQ-20`, and `US-01..US-15`.
- [x] Brand typography rule (`DIR-12` / `BR-14` / `D10`): always all-lowercase **`design`:os studio** with bold **`design`** (`700`) and regular `:os studio` (`400`).
- [x] Multiple Tab Project Workspace (`DIR-13` / `EP-10` / `REQ-12` / `US-10` / `S10` / `BR-15`): `36px` Desktop Tab Strip with 100% per-tab state isolation.
- [x] 1:1 Figma AI Agent Suite (`DIR-14`, `DIR-15` / `US-11` / `S11`) & Quantitative 5-Pillar Figma Parity Scorecard (`DIR-16` / `US-12` / `S12`).
- [x] 1:1 Figma UI3 `Agents` Left Side Panel (`DIR-17` / `S13`), 9 Visual Deltas, 5 Visual Micro-Gaps (`Gap 1` 3-way Bottom Toolbar `Draw | Design | </>`, `Gap 2` `Assets` vertical library card & `Icon::BookOpen`, `Gap 3` `Team project` subtitle & `Collapse layers` button, `Gap 4` Main Menu `AI balance` & `Help and account`, `Gap 5` Left Rail 1-click `File` return & collapse/expand toggle) & End-to-End Userflows `UF-01..UF-08` (`userflows_uf01_to_uf08_end_to_end_figma_parity` in `crates/designos-app/src/shell/tests/home.rs`, `DIR-18`, `DIR-20` / `US-13`).
- [x] Unified Phosphor Icons System (`DIR-19` / `EP-14` / `REQ-17` / `US-14` / `BR-22` / `D17`): 100% `crate::icons::Icon` variants (except `Icon::Logo`) mapped to `@phosphor-icons/core` SVGs (`256×256`) in `crates/designos-app/assets/icons/phosphor/*.svg` and rasterized via `resvg` with DPR-aware pixel snapping across `S01..S13`.
- [x] Pure Chat UI Wrapper Over Native PTY Terminal Shell (`DIR-21` / `EP-15` / `REQ-18` / `US-15` / `BR-23` / `D18`): `RailItem::Agents` (`S13`) wraps live `Claude Code (claude)`, `Codex (codex)`, and `Antigravity CLI (agy)` PTY shells with 1-click runtime pills, ANSI-stripped stdout chat bubbles (`AgentChatTurn` / `sync_chat_from_pty` / `strip_ansi_codes`), inline `NodeSpec` Canvas apply badge, Selection + Pin-Comment context attachment (`+`), and instant Chat/VT100 terminal toggle.
- [x] Final Cross-Platform Release Build & User-Installable Packaging Workflow (`DIR-22` / `EP-16` / `REQ-19` / `US-09` / `BR-24` / `D19`): macOS `.app` + `.dmg` + `install-cli.command`, Windows NSIS `-setup.exe` + `install.ps1`, Linux `.deb` + `.tar.gz` + `install.sh`, `scripts/build-release.sh`, and `.github/workflows/release.yml` generating `dist/release-manifest.json` + `SHA256SUMS`.
- [x] 16-Capture Retina 2x Visual Proof Suite & `design-os-github-page` Showcase (`DIR-23` / `EP-17` / `REQ-20` / `US-08`, `US-15` / `BR-25` / `D20`): `scripts/capture-visual-proof.sh` regenerates all 16 Retina 2x PNGs in `docs/screenshots/*.png` headlessly (`with_visible(false).with_active(false)`), and `README.md` (12-Section AI-Ready README, 5-Gate `D1..D5`) + `docs/index.html` (9-Tier GitHub Pages, SEO/AEO/GEO `G1..G5`) pass all `https://github.com/jangtrinh/design-os-github-page` gates.
- [x] Wave 8 (`W8 Complete`) Verified: `387/387` (`100.00%`) 5-Pillar Quantitative Figma Parity Meter (`DIR-20`..`DIR-23`).
- [x] Contract `FC-001.md` Locked (`BR-01..BR-25`, Amendments `A-01..A-12`).
- [x] Decision Log `D1..D20` recorded with zero blocking `OPEN` items.
- [x] Screen Inventory (`S01..S13`) and UI Contract (`78` rows = `13 screens × 6 states`) complete.
- [x] Test Plan (`46` scenarios `SCN-01-01..SCN-15-03`) mapped 1:1 to acceptance criteria.
- [x] All 5 Quality Gates (`g1-intake` .. `g5-handoff`) audited with SHA-256 artifact digests and 6-criteria `2/2` rubric scores.
