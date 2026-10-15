# **design**:os studio — Master Prioritized Product Backlog (`plans/BACKLOG.md`)

**Canonical Spec Bundle:** [`plans/specs/designos-studio-v2/`](specs/designos-studio-v2/spec.manifest.json)  
**Pipeline Standard:** `product-os-spec-driven` Format v2 (11 Stages · 5 Gates `G1..G5` · 12-Column `ui-contract.md`)  
**Updated:** 2026-10-15 · **Owner:** `po@designos.local` · **Status:** Active Execution (`EP-01..EP-10` · `US-01..US-10` · `S01..S10`)

---

## 1. Executive Summary & Traceability Architecture

Every user directive across the history of **`design`:os studio** is captured in this canonical backlog and traced end-to-end through the **11-stage `product-os-spec-driven` Format v2 pipeline**:

```text
User Directive (Intake: po-spec.md · DIR-01..DIR-13)
  ──► Decisions & Clarifications (D1..D11 · CL-01..CL-10 · 0 blocking OPEN)
  ──► Feature Contract (FC-001: BR-01..BR-15 · Amendments A-01..A-05)
  ──► Product Requirements (PRD.md: REQ-01..REQ-12)
  ──► User Stories (US-01..US-10) & Acceptance Criteria (30 ACs)
  ──► Wireframes (W-01: S01..S10) & Interactive Prototype (P-01)
  ──► Design Tokens (design/tokens.md) & 12-Column UI Contract (ux/ui-contract.md · 60 Rows)
  ──► Automated Scenarios & TDD (tdd/test-plan.md: SCN-01-01..SCN-10-03 · 30 SCNs)
  ──► Gates (G1..G5) & Handoff Manifest (handoff/READY.md)
```

---

## 2. Historical User Directives Log (Chronological Intake Ledger `DIR-01..DIR-13`)

| Directive ID | User Request Summary | Primary Epic | Target `REQ` | Target `US` | Gate / Wave |
|---|---|---|---|---|---|
| **DIR-01** | Reverse-engineer `/Applications/Figma.app` (`app.asar`, Electron main/preload/renderer bundles, Native bindings) and `/Users/jangtrinh/Downloads/VSF - PCP.fig` (`fig-kiwi` v106) using `morluto/rea` (`rea`) to extract exact Kiwi binary schemas, node field IDs, and UI3 CSS design tokens. | `EP-01` | `REQ-09`, `REQ-01` | `US-07`, `US-01` | `G1`, `G3` · `W1` |
| **DIR-02** | Achieve 99% 1:1 `.fig` v106 binary import and lossless `.fig` round-trip export (`export_fig` / `import_fig`), preserving 100% of unknown Kiwi fields (`raw_fields`), `parameterConsumptionMap` (id=445), `symbolOverrides`, nested `overriddenSymbolID` instance swaps, `fontVariations`, OpenType features, and `handoffStatusMap`. | `EP-02` | `REQ-01`, `REQ-02` | `US-01` | `G2`, `G3` · `W1` |
| **DIR-03** | Implement 2D Auto-Layout CSS Grid (`LayoutDirection::Grid`, `stackMode: "GRID"`) with track sizing (`fr`, `px`, `auto`), `col_span` / `row_span`, and bidirectional `.spec.json` (`NodeSpec`) canvas sync. | `EP-02`, `EP-03` | `REQ-01`, `REQ-02`, `REQ-08` | `US-01`, `US-06` | `G3`, `G4` · `W1` |
| **DIR-04** | Deliver 1:1 Figma per-element canvas interactions across all 17+ `NodeKind` types (hover outlines `#0D99FF` / `#9747FF`, double-click hierarchy drill-down, 8 resize handles, corner-radius inner dots, Auto-Layout padding/gap pink handles, `Alt` redline distance guides, and dotted 2D Grid cell overlays with track pills). | `EP-03` | `REQ-08` | `US-06` | `G3`, `G4` · `W2` |
| **DIR-05** | Match Figma UI3 Dark Mode chrome 1:1 (`#1E1E1E` canvas, `#2C2C2C` panels, `#383838` inputs, `#444444` borders, `#0D99FF` selection blue, `#9747FF` component purple, `#24CB71` Dev Mode green; `48px` icon-only left rail; `240px` left sidebar with `Team project` subtitle & `Collapse layers`; `528×48px` bottom floating toolbar with 4-mode `Toolbelt Mode`; `240px` right inspector with Multiplayer avatars & blue `Share` button). | `EP-04` | `REQ-07` | `US-05` | `G4` · `W2` |
| **DIR-06** | Implement 1:1 Figma UI3 Settings, Menus & Popovers: `Type settings` 3-tab popover (`Basic \| Details \| Variable`), bottom-docked `Actions (Cmd+K)` bar (`All \| AI \| Actions \| Libraries`), `Zoom/View` menu with editable `%` input, full Color Picker (`HEX/RGB/CSS/HSL/HSB`), Scoped Variables Picker, and Dev Mode (`</>`) Box Model & 5-language codegen inspector. | `EP-04` | `REQ-07` | `US-05` | `G4` · `W2` |
| **DIR-07** | Build 1:1 Figma UI3 File Manager & Home Screen (`Drafts`, `All folders`, `Resources / Templates`, `Starred`, `Trash`, file-type badges, search/filter/sort, grid/list toggle, and file card context menu). | `EP-05` | `REQ-06` | `US-04` | `G4` · `W2` |
| **DIR-08** | Embed a native multi-tab PTY Agent Terminal (`claude`, `codex`, `grok`, `$SHELL`) via `portable-pty` + `vt100` with `DESIGNOS_*` env injection, `Copy cho AI` citation bridge (`state.last_ai_copy`), and live stdout `NodeSpec` canvas streaming at 120fps. | `EP-06` | `REQ-05` | `US-03` | `G3`, `G4` · `W1` |
| **DIR-09** | Integrate a native `product-os-spec-driven` Format v2 Reader in Studio (`S02`) supporting all 11 pipeline stages, 5 Role Views (`PO View`, `Dev View`, `QA & Coverage`, `Wireframes & UI Contract`, `Gates & Canvas`), 4-Metric KPI Strip, Attention Line, and `Copy cho AI` citations under the `es:designer` Operate-mode craft floor (`#16171B` / `#1F2024` / `#26272C`, zero emoji chrome, no cards-in-cards). | `EP-07` | `REQ-03`, `REQ-04` | `US-02` | `G3`, `G4` · `W1` |
| **DIR-10** | Enforce 100% background headless screenshot & QA verification (`with_visible(false).with_active(false)` / offscreen software/GPU rendering) so automated verification never steals macOS window focus or interrupts the user's screen. | `EP-08` | `REQ-10` | `US-08` | `G4`, `G5` · `W1`–`W2` |
| **DIR-11** | Design and implement a **Dual-Layer Auto-Update Architecture (`EP-09`)**: (1) **Layer 1 — Native App & CLI Atomic Self-Updater (`updater.rs` + `designos update`)** with `Stable \| Beta \| Nightly` channels, SHA-256 verification, atomic `.staged` / `.bak` swap and automatic rollback, and UI3 Update Pill & Modal (`S09`); (2) **Layer 2 — Upstream Figma Desktop (`app.asar` & `.fig` Kiwi Schema) Auto-Sync Watchdog** that passively monitors `/Applications/Figma.app/Contents/Info.plist` (`CFBundleShortVersionString`) & `app.asar` / Kiwi schema (`v106`) in the background (100% read-only, zero GUI activation) and offers 1-click/CLI `Sync Figma Schema`. | `EP-09` | `REQ-11` | `US-09` | `G3`, `G4`, `G5` · `W2` |
| **DIR-12** | **Brand Typography & Wordmark Rule (`BR-14` / `D10`)**: The product brand name must ALWAYS be written in all-lowercase as **`design`:os studio** (or **`design`:os**), with **`design`** rendered in **Bold** (`700` / `egui::TextFormat` bold span) and `:os studio` rendered in Regular (`400`) weight. Uppercase or title-case variants (`"DESIGN:OS Studio"`, `"Design OS Studio"`, `"DESIGN:OS"`) are strictly forbidden in UI chrome and user-facing headings. | `EP-04`, `EP-10` | `REQ-07`, `REQ-12` | `US-05`, `US-10` | `G2`, `G4` · `W2` |
| **DIR-13** | **Multiple Tab Project Workspace (`EP-10` / `REQ-12` / `US-10` / `S10` / `BR-15`)**: Support opening and working across multiple project tabs simultaneously via a top `36px` Figma UI3 Desktop Tab Strip (`Home` icon tab + open project tabs with file-type badge `Design` / `FigJam` / `Slides` / `Specs`, dirty indicator `•`, Pin state, close `×`, and `+` New Tab button), with full per-tab state isolation (`Document`, `History` undo/redo stack, active `Page`, `Selection`, `View` zoom/pan offset, `Dev Mode` toggle, and file path), keyboard shortcuts (`Cmd+T`, `Cmd+W`, `Ctrl+Tab` / `Cmd+Shift+]`, `Cmd+1..9`), and tab context menu (`Duplicate tab`, `Pin/Unpin tab`, `Close other tabs`). | `EP-10` | `REQ-12` | `US-10` | `G3`, `G4`, `G5` · `W2` |

---

## 3. Master Epic Breakdown (`EP-01` .. `EP-10`)

| Epic ID | Epic Title | User Stories | PRD REQs | Screens | Business Rules | Priority / Wave | Status |
|---|---|---|---|---|---|---|---|
| **EP-01** | Reverse Engineering Figma App (`app.asar` & `.fig` v106 Kiwi Binary Schema) | [`US-07`](specs/designos-studio-v2/us/US-07.md), [`US-01`](specs/designos-studio-v2/us/US-01.md) | `REQ-09`, `REQ-01` | `S01`, `S07` | `BR-01`, `BR-10` | `MVP` · `W1` | Verified |
| **EP-02** | 99% 1:1 `.fig` Import & Lossless Roundtrip Rendering | [`US-01`](specs/designos-studio-v2/us/US-01.md) | `REQ-01`, `REQ-02` | `S01`, `S02`, `S03` | `BR-01`, `BR-05` | `MVP` · `W1` | Verified |
| **EP-03** | 1:1 Per-Element Canvas Interactions & 2D CSS Grid Overlay (All 17 Node Types) | [`US-06`](specs/designos-studio-v2/us/US-06.md) | `REQ-08`, `REQ-01` | `S01` | `BR-01`, `BR-09` | `MVP` · `W2` | Verified |
| **EP-04** | 1:1 Figma UI3 Chrome, Settings, Menus, Popovers & Dev Mode Inspector | [`US-05`](specs/designos-studio-v2/us/US-05.md) | `REQ-07` | `S05`, `S06`, `S07`, `S08` | `BR-07`, `BR-08`, `BR-14` | `MVP` · `W2` | Verified |
| **EP-05** | 1:1 Figma UI3 File Manager & Home Screen (`Drafts`, `All folders`, `Starred`, `Trash`, `Templates`) | [`US-04`](specs/designos-studio-v2/us/US-04.md) | `REQ-06` | `S04` | `BR-06`, `BR-14` | `MVP` · `W2` | Verified |
| **EP-06** | Native Embedded PTY Agent Terminal (`claude`, `codex`, `grok`, `$SHELL`) & Bidirectional Canvas Bridge | [`US-03`](specs/designos-studio-v2/us/US-03.md) | `REQ-05` | `S01`, `S02`, `S03` | `BR-04`, `BR-05` | `MVP` · `W1` | Verified |
| **EP-07** | Native `product-os-spec-driven` Format v2 Reader & `es:designer` Craft Floor | [`US-02`](specs/designos-studio-v2/us/US-02.md) | `REQ-03`, `REQ-04` | `S01`, `S02`, `S03` | `BR-02`, `BR-03`, `BR-04` | `MVP` · `W1` | Verified |
| **EP-08** | 100% Background Headless Verification (`with_active(false)`) & Side-by-Side Figma Parity Gate | [`US-08`](specs/designos-studio-v2/us/US-08.md) | `REQ-10` | `S01`..`S10` | `BR-11`, `BR-12` | `MVP` · `W1`–`W2` | Verified |
| **EP-09** | Dual-Layer Auto-Update Architecture (Atomic App/CLI Self-Updater & Upstream Figma Schema Watchdog) | [`US-09`](specs/designos-studio-v2/us/US-09.md) | `REQ-11` | `S09`, `S04`, `S01` | `BR-13` | `MVP` · `W2` | Verified |
| **EP-10** | Multiple Tab Project Workspace & Figma UI3 Desktop Tab Bar (`S10`) | [`US-10`](specs/designos-studio-v2/us/US-10.md) | `REQ-12` | `S10`, `S04`, `S01` | `BR-14`, `BR-15` | `MVP` · `W2` | Specified / Ready for Subagent Dispatch |

---

## 4. User Stories, Acceptance Criteria & Verification Matrix (`US-01` .. `US-10`)

| Story ID | Title | Epic | REQs | Decisions | Screens | Scenarios (`auto` / `visual`) | Primary Verification Test / Command |
|---|---|---|---|---|---|---|---|
| **[`US-01`](specs/designos-studio-v2/us/US-01.md)** | 1:1 Figma `.fig` v106 Binary R/W & 2D CSS Grid | `EP-02`, `EP-01` | `REQ-01`, `REQ-02` | `D1`, `D5` | `S01`, `S02`, `S03` | `SCN-01-01`, `SCN-01-02`, `SCN-01-03` | `cargo test -p designos-fig --test fig_roundtrip` & `cargo test -p designos-app layout::tests` |
| **[`US-02`](specs/designos-studio-v2/us/US-02.md)** | PRODUCT:OS Spec Driven v2 Reader & Role Views | `EP-07` | `REQ-03`, `REQ-04` | `D2`, `D3`, `D4` | `S01`, `S02`, `S03` | `SCN-02-01`, `SCN-02-02`, `SCN-02-03` | `cargo test -p designos-app specs::tests` |
| **[`US-03`](specs/designos-studio-v2/us/US-03.md)** | Embedded PTY Agent Terminal `claude`/`codex` & Live Canvas Bridge | `EP-06` | `REQ-05` | `D4`, `D5` | `S01`, `S02`, `S03` | `SCN-03-01`, `SCN-03-02`, `SCN-03-03` | `cargo test -p designos-app terminal::tests` |
| **[`US-04`](specs/designos-studio-v2/us/US-04.md)** | 1:1 Figma UI3 Home & File Management (`Drafts`, `All folders`, `Trash`, `Starred`, `Templates`) | `EP-05` | `REQ-06` | `D6` | `S04` | `SCN-04-01`, `SCN-04-02`, `SCN-04-03` | `cargo test -p designos-app home::` & `shell::tests` |
| **[`US-05`](specs/designos-studio-v2/us/US-05.md)** | 1:1 Figma UI3 Settings, Menus & Popovers (`Type settings`, `Cmd+K`, `Zoom/View`, `Color/Variables`, `Dev Mode`) | `EP-04` | `REQ-07` | `D6` | `S05`, `S06`, `S07`, `S08` | `SCN-05-01`, `SCN-05-02`, `SCN-05-03` | `cargo test -p designos-app panels::menus::tests` & `cmdk::tests` & `dev::tests` |
| **[`US-06`](specs/designos-studio-v2/us/US-06.md)** | 1:1 Figma Per-Element Canvas Interactions & 2D Grid Overlay (17 Node Kinds) | `EP-03` | `REQ-08`, `REQ-01` | `D1`, `D6` | `S01` | `SCN-06-01`, `SCN-06-02`, `SCN-06-03` | `cargo test -p designos-app canvas::interact::tests` & `canvas::handles::tests` |
| **[`US-07`](specs/designos-studio-v2/us/US-07.md)** | Figma Desktop (`app.asar`) & `.fig` Kiwi Binary Reverse Engineering (REA) Pipeline | `EP-01` | `REQ-09` | `D1`, `D7` | `S01`, `S07` | `SCN-07-01`, `SCN-07-02`, `SCN-07-03` | `cargo test -p designos-fig` & `plans/specs/designos-studio-v2/research/` |
| **[`US-08`](specs/designos-studio-v2/us/US-08.md)** | Headless Background Screenshot Verification (`with_active(false)`) & 99% Parity Gate | `EP-08` | `REQ-10` | `D8` | `S01`..`S10` | `SCN-08-01`, `SCN-08-02`, `SCN-08-03` | `cargo test -p designos-app cli::tests` & `designos --screenshot <out.png>` |
| **[`US-09`](specs/designos-studio-v2/us/US-09.md)** | Dual-Layer Auto-Update (Atomic Self-Updater `updater.rs` + Upstream Figma `app.asar`/Kiwi Watchdog) | `EP-09` | `REQ-11` | `D9` | `S09`, `S04`, `S01` | `SCN-09-01`, `SCN-09-02`, `SCN-09-03` | `cargo test -p designos-app updater::tests` & `designos update --check` |
| **[`US-10`](specs/designos-studio-v2/us/US-10.md)** | Multiple Tab Project Workspace & Figma UI3 Desktop Tab Strip (`36px` Tab Bar, Per-Tab Isolation & Shortcuts) | `EP-10` | `REQ-12` | `D10`, `D11` | `S10`, `S04`, `S01` | `SCN-10-01`, `SCN-10-02`, `SCN-10-03` | `cargo test -p designos-app tabs::tests` & `app::tests` |

---

## 5. 11-Stage `product-os-spec-driven` Format v2 Artifact Map

| Stage # | Stage ID | Owner Role | Artifact Path(s) in `plans/specs/designos-studio-v2/` | Gate | Status |
|---|---|---|---|---|---|
| **1** | `intake` | Product Owner (`po`) | [`intake/po-spec.md`](specs/designos-studio-v2/intake/po-spec.md), [`spec.manifest.json`](specs/designos-studio-v2/spec.manifest.json) | [`g1-intake.md`](specs/designos-studio-v2/review/gates/g1-intake.md) | `APPROVED` (`CURRENT`) |
| **2** | `clarify` | Product Owner (`po`) | [`clarify/clarifications.md`](specs/designos-studio-v2/clarify/clarifications.md), [`decisions/decision-log.md`](specs/designos-studio-v2/decisions/decision-log.md), [`decisions/glossary.md`](specs/designos-studio-v2/decisions/glossary.md) | `clarify` (via `G2`) | `Done` (`0` blocking `OPEN`) |
| **3** | `contract` | Product Owner (`po`) | [`contract/FC-001.md`](specs/designos-studio-v2/contract/FC-001.md) (`BR-01..BR-15`, Amendments `A-01..A-05`) | [`g2-contract.md`](specs/designos-studio-v2/review/gates/g2-contract.md) | `APPROVED` (`CURRENT`) |
| **4** | `prd` | Product Owner (`po`) | [`prd/PRD.md`](specs/designos-studio-v2/prd/PRD.md) (`REQ-01..REQ-12`) | `prd` (via `G3`) | `Approved` (`CURRENT`) |
| **5** | `us` | Product Owner (`po`) | [`us/INDEX.md`](specs/designos-studio-v2/us/INDEX.md), [`us/US-01.md`](specs/designos-studio-v2/us/US-01.md) .. [`us/US-10.md`](specs/designos-studio-v2/us/US-10.md) | [`g3-us.md`](specs/designos-studio-v2/review/gates/g3-us.md) | `APPROVED` (`CURRENT`) |
| **6** | `wireframe` | Product Owner (`po`) | [`wireframe/W-01.md`](specs/designos-studio-v2/wireframe/W-01.md) (`S01..S10` ASCII normal/empty/error/loading) | `wireframe` | `Verified` |
| **7** | `prototype` | Product Owner (`po`) | [`prototype/P-01.md`](specs/designos-studio-v2/prototype/P-01.md) (`S01..S10` interactive flows & state machines) | `prototype` | `Verified` |
| **8** | `design` | Developer (`dev`) | [`design/tokens.md`](specs/designos-studio-v2/design/tokens.md), [`ux/screen-inventory.md`](specs/designos-studio-v2/ux/screen-inventory.md), [`ux/ui-contract.md`](specs/designos-studio-v2/ux/ui-contract.md) | `design` | `Verified` |
| **9** | `tdd` | QA / QC (`qc`) | [`tdd/test-plan.md`](specs/designos-studio-v2/tdd/test-plan.md) (`SCN-01-01..SCN-10-03` test matrix) | `g3-tdd` | `Passed` (`30/30 SCN`) |
| **10** | `uiux` | Product Owner (`po`) | [`ux/screen-inventory.md`](specs/designos-studio-v2/ux/screen-inventory.md), [`ux/ui-contract.md`](specs/designos-studio-v2/ux/ui-contract.md) (`S01..S10` × 6 states = 60 rows) | [`g4-uiux.md`](specs/designos-studio-v2/review/gates/g4-uiux.md) | `PENDING` (PO visual sign-off) |
| **11** | `handoff` | Product Owner (`po`) | [`handoff/READY.md`](specs/designos-studio-v2/handoff/READY.md), [`config/pipeline.yaml`](specs/designos-studio-v2/config/pipeline.yaml) | [`g5-handoff.md`](specs/designos-studio-v2/review/gates/g5-handoff.md) | `APPROVED` (`CURRENT`) |

---

## 6. Subagent Dispatch Waves & Strict File Ownership Matrix

| Wave | Subagent Role | Assigned Epics & Stories | Strict Owned File Boundaries | Verification Gate |
|---|---|---|---|---|
| **Wave 1 (`W1`)** | `figma-rea-specialist` | `EP-01` (`US-07`) | `plans/specs/designos-studio-v2/research/*` | `G1`, `G3` |
| **Wave 1 (`W1`)** | `spec-driven-pipeline-orchestrator` | All 11 Spec Stages (`EP-01..EP-10`, `US-01..US-10`) | `plans/BACKLOG.md`, `plans/specs/designos-studio-v2/{intake,clarify,contract,prd,us,wireframe,prototype,design,tdd,ux,handoff,decisions,review,config,spec.manifest.json}` | `G1..G5` |
| **Wave 1 (`W1`)** | `core-figma-parity` (`backend-specialist`) | `EP-02` (`US-01`) | `crates/designos-doc/`, `crates/designos-fig/`, `crates/designos-render/`, `crates/designos-export/` | `cargo test -p designos-fig --test fig_roundtrip` |
| **Wave 1 (`W1`)** | `agent-terminal-and-cmdk` (`fullstack-developer`) | `EP-06`, `EP-07` (`US-02`, `US-03`) | `crates/designos-app/src/{terminal.rs,cmdk.rs,specs.rs}` | `cargo test -p designos-app specs::tests terminal::tests` |
| **Wave 2 (`W2`)** | `ui3-chrome-and-menus` (`frontend-specialist`) | `EP-03`, `EP-04`, `EP-05`, `EP-08`, `EP-10` (`US-04`, `US-05`, `US-06`, `US-08`, `US-10`) | `crates/designos-app/src/{app.rs,shell.rs,theme.rs,home/,panels/,canvas/,dev/,variables/,widgets/,cli.rs,main.rs}` | `cargo test --workspace` + headless `--screenshot` |
| **Wave 2 (`W2`)** | `auto-update-architect` (`backend-specialist` / `fullstack-developer`) | `EP-09` (`US-09`) | `crates/designos-app/src/updater.rs`, `crates/designos-app/src/{cli.rs,main.rs,app.rs,home/mod.rs,panels/menus.rs,specs.rs}` | `cargo test -p designos-app updater::tests` & `designos update --check` |
| **Wave 3 (`W3` Backlog)** | `multiplayer-sync-architect` | Out-of-Scope / Future Wave 3: Real-time CRDT WebSocket multiplayer server | `crates/designos-sync/` (planned) | Deferred per `FC-001` |
