---
id: decision-log
type: decisions
title: "Decision Ledger & Open Items Tracker (D1..D11)"
status: Current to D11
updated: 2026-10-15
ledger: true
trace:
  amendments: [A-01, A-02, A-03, A-04, A-05]
topics:
  - id: architecture
    name: Architecture, REA, Binary Format, Auto-Update & Multi-Tab State
    note: Figma binary v106, REA pipeline, Dual-Layer Auto-Update, Multi-Tab Project isolation, and Spec Format v2 decisions
    ids: [D1, D3, D5, D7, D9, D11]
  - id: craft
    name: UI3 Design Craft, Brand Typography, Agent Citations & Headless QA
    note: Figma UI3 tokens, design:os studio lowercase+bold brand rule, Operate-mode tokens, Copy cho AI, and background capture
    ids: [D2, D4, D6, D8, D10]
---

## 1. Settled Decisions (D1..D11)

| Id | Decision | Source and detail |
|---|---|---|
| D1 | Preserve 100% unknown Kiwi fields (`raw_fields`), `parameterConsumptionMap` (id=445), `symbolOverrides`, `fontVariations`, and `toggledOnOTFeatures` in `.fig` v106 round-trip, and support 2D CSS Grid (`fr`/`px`/`auto`, `col_span`/`row_span`) | Owner round R0, 2026-10-14: see [contract](../contract/FC-001.md) BR-01, [US-01](../us/US-01.md), [US-07](../us/US-07.md). |
| D2 | Strictly enforce `es:designer` Operate-mode tokens (`#16171B` / `#1F2024` / `#26272C`), zero emoji icons in chrome, and no cards-in-cards inside Spec Reader v2 (`S02`) | Owner round R1, 2026-10-14: amendment A-01, [contract](../contract/FC-001.md) BR-02, [US-02](../us/US-02.md). |
| D3 | Compute all 4 KPI strip metrics (`REQs · US`, `ACs · SCNs`, `Screens · Contract Rows`, `Gates Approved`) from a single loaded spec model | Owner round R1, 2026-10-14: see [contract](../contract/FC-001.md) BR-03, [US-02](../us/US-02.md). |
| D4 | Standardize `Copy cho AI` citation block with `File: <path>:<start>-<end>`, `Mục`, `Trang reader`, `Project`, and persist to `last_ai_copy` for PTY Terminal injection | Owner round R1, 2026-10-15: amendment A-02, [contract](../contract/FC-001.md) BR-04, [US-02](../us/US-02.md), [US-03](../us/US-03.md). |
| D5 | Auto-discover `plans/specs/designos-studio-v2`, `product-os-spec-driven/fixtures/v2/full`, and `reference/baseline-v0.2` alongside legacy v1 specs and map all 10 screens (`S01..S10`) in `ui-contract.md` | Owner round R1, 2026-10-15: see [contract](../contract/FC-001.md) BR-05, [US-02](../us/US-02.md), [US-03](../us/US-03.md). |
| D6 | Enforce 1:1 Figma UI3 Dark Mode palette (`#1E1E1E` canvas, `#2C2C2C` panel, `#383838` input well, `#444444` border, `#0D99FF` selection blue, `#9747FF` component purple, `#24CB71` Dev Mode green), `48px` icon-only rail, Home screen (`S04`), Popovers (`S05..S08`), and 17-node canvas interactions | Owner round R2, 2026-10-15: amendment A-03, [contract](../contract/FC-001.md) BR-06..BR-09, [US-04](../us/US-04.md), [US-05](../us/US-05.md), [US-06](../us/US-06.md). |
| D7 | Execute `morluto/rea` reverse engineering on `/Applications/Figma.app` (`app.asar`) and `/Users/jangtrinh/Downloads/VSF - PCP.fig`, storing architectural artifacts in `plans/specs/designos-studio-v2/research/` and round-trip tests in `crates/designos-fig/` | Owner round R2, 2026-10-15: amendment A-03, [contract](../contract/FC-001.md) BR-10, [US-07](../us/US-07.md). |
| D8 | Mandate 100% background headless window creation (`with_visible(false).with_active(false)`) for all `--screenshot` verification commands so automated QA never steals macOS focus | Owner round R2, 2026-10-15: amendment A-03, [contract](../contract/FC-001.md) BR-11..BR-12, [US-08](../us/US-08.md). |
| D9 | Implement Dual-Layer Auto-Update Architecture (`updater.rs` + `designos update`): Layer 1 atomic App/CLI self-updater (`Stable \| Beta \| Nightly`, SHA-256 verification, `.staged` → `.bak` atomic swap & rollback, UI3 Update Modal `S09`) and Layer 2 passive read-only Upstream Figma Desktop (`app.asar` & `.fig` Kiwi schema `v106`) Auto-Sync Watchdog | Owner round R3, 2026-10-15: amendment A-04, [contract](../contract/FC-001.md) BR-13, [US-09](../us/US-09.md). |
| D10 | Enforce lowercase brand typography wordmark **`design`:os studio** (or **`design`:os**): always all-lowercase with **`design`** rendered in Bold (`700` / `egui::TextFormat` bold span) and `:os studio` rendered in Regular (`400`); ban uppercase/title-case variants (`DESIGN:OS Studio`, `Design OS Studio`, `DESIGN:OS`) across UI chrome | Owner round R4, 2026-10-15: amendment A-05, [contract](../contract/FC-001.md) BR-14, [US-05](../us/US-05.md), [US-10](../us/US-10.md). |
| D11 | Implement Multiple Tab Project Workspace (`S10`) with a top `36px` Figma UI3 Desktop Tab Strip (`Home` icon tab + open project tabs with file-type badge, dirty dot `•`, pin state, `×` close, `+` new tab), full per-tab state isolation (`Document`, `History` undo/redo, active `Page`, `Selection`, `View` zoom/pan, `Dev Mode`, `file_path`), keyboard shortcuts (`Cmd+T`, `Cmd+W`, `Ctrl+Tab`/`Cmd+Shift+]`, `Cmd+1..9`), and tab context menu | Owner round R4, 2026-10-15: amendment A-05, [contract](../contract/FC-001.md) BR-15, [US-10](../us/US-10.md). |

## 2. Open Items (`OPEN` / `OPEN[blocking]` Tracker)

| ID | Kind | Scope | Summary | Status |
|---|---|---|---|---|
| OPEN-01 | OPEN | G4 / UIUX | Confirm final PO sign-off on Gate G4 (`g4-uiux.md`) after visual inspection of all 10 screens `S01..S10` and 5 Role Views (0 blocking items remaining). | non-blocking · awaiting PO |
