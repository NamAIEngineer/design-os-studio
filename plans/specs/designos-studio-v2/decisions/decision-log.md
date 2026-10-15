---
id: decision-log
type: decisions
title: "Decision Ledger & Open Items Tracker (D1..D5)"
status: Current to D5
updated: 2026-10-15
ledger: true
trace:
  amendments: [A-01, A-02]
topics:
  - id: architecture
    name: Architecture & Format
    note: Figma binary v106 and Spec Format v2 decisions
    ids: [D1, D3, D5]
  - id: craft
    name: Design Craft & Agent Citations
    note: Operate-mode tokens, zero-emoji floor, and Copy cho AI
    ids: [D2, D4]
---

## 1. Settled Decisions (D1..D5)

| Id | Decision | Source and detail |
|---|---|---|
| D1 | Preserve 100% unknown Kiwi fields (`raw_fields`) in `.fig` v106 round-trip and support 2D CSS Grid (`fr`/`px`/`auto`, `col_span`/`row_span`) | Owner round R0, 2026-10-14: see [contract](../contract/FC-001.md) BR-01, [US-01](../us/US-01.md). |
| D2 | Strictly enforce `es:designer` Operate-mode tokens (`#16171B` / `#1F2024` / `#26272C`), zero emoji icons in chrome, and no cards-in-cards | Owner round R1, 2026-10-14: amendment A-01, [contract](../contract/FC-001.md) BR-02, [US-02](../us/US-02.md). |
| D3 | Compute all 4 KPI strip metrics (`REQs · US`, `ACs · SCNs`, `Screens · Contract Rows`, `Gates Approved`) from a single loaded spec model | Owner round R1, 2026-10-14: see [contract](../contract/FC-001.md) BR-03, [US-02](../us/US-02.md). |
| D4 | Standardize `Copy cho AI` citation block with `File: <path>:<start>-<end>`, `Mục`, `Trang reader`, `Project`, and persist to `last_ai_copy` | Owner round R1, 2026-10-15: amendment A-02, [contract](../contract/FC-001.md) BR-04, [US-02](../us/US-02.md). |
| D5 | Auto-discover `plans/specs/designos-studio-v2`, `product-os-spec-driven/fixtures/v2/full`, and `reference/baseline-v0.2` alongside legacy v1 specs | Owner round R1, 2026-10-15: see [contract](../contract/FC-001.md) BR-05, [US-02](../us/US-02.md), [US-03](../us/US-03.md). |

## 2. Open Items (`OPEN` / `OPEN[blocking]` Tracker)

| ID | Kind | Scope | Summary | Status |
|---|---|---|---|---|
| OPEN-01 | OPEN | G4 / UIUX | Confirm final PO sign-off on Gate G4 (`g4-uiux.md`) after visual inspection of all 5 Role Views (0 blocking items remaining). | non-blocking · awaiting PO |
