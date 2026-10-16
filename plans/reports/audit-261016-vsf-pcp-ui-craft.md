# Audit Report: VSF-PCP 124-Rule UI Craft, Home Data Truth & Zero Fake UI Gate (`261016`)

- **Date:** 2026-10-16
- **Project:** `designos-studio` (`/Users/jangtrinh/Products/design-os-studio`)
- **Auditors:** `curator-jang-designos-studio` & VSF-PCP UI Craft Gate
- **Scope:** `crates/designos-app/src/theme.rs`, `crates/designos-app/src/home/mod.rs`, `crates/designos-app/src/panels/right.rs`, `crates/designos-app/src/terminal.rs`, `scripts/zero_fake_ui_gate.py`

## 1. Executive Summary
A comprehensive 124-rule VSF-PCP UI craft and behavioral truthfulness audit was executed across `design:os studio`. Initial control baselines scored `48–52/100` due to silent Home section fallbacks, inline purple (`#9747FF`) component accents violating the studio Purple Ban, inline count strings (`"Layers (4)"`), cramped `<24px` pointer targets, and floating status chrome. After systematic remediation and token unification in `theme.rs` ↔ `design/design.tokens.json`, treatment screens achieved `95–97/100` with `0` regressions and `12/12` `zero_fake_ui_gate.py` (`Z01..Z12`) checks passing.

## 2. Verbatim Findings & Remediations

### F1 — Strict Purple Ban & Single Token Authority (`P1`, `P2`)
- **Finding:** Component badges and instance pills previously used generic Figma violet (`#9747FF` / `#8755E9`) and scattered `Color32::from_rgb` literals across inspector and layer rows.
- **Fix (`crates/designos-app/src/theme.rs`):** Replaced all component accents with `#06B6D4` Cyan (`theme::ACCENT_COMPONENT` / `colors.component`), preserved `#E36323` Brand Orange (`colors.primary`), `#14AE5C` Dev Mode Emerald (`colors.success`), and `#0D99FF` Selection Blue (`colors.info`). Raised `colors.neutral` to `#6e6e77` so all 15 text×surface pairs and 8 interaction-state pairs pass WCAG AA (`>= 4.5:1`).

### F2 — Home Workspace Section Data Truth (`P6`, `Z01..Z12`)
- **Finding:** Clicking `Drafts`, `Starred`, `Templates`, `Design Systems`, `Trash`, or `Archive` in `crates/designos-app/src/home/mod.rs` previously fell back silently to rendering `Recents` when a section had no dedicated items, misleading the user.
- **Fix (`crates/designos-app/src/home/mod.rs`):** Implemented strict per-section filtering (`HomeSection::Drafts`, `Starred`, `Templates`, `DesignSystems`, `Trash`, `Archive`) with truthful empty-state views and wired `scripts/zero_fake_ui_gate.py` (`Z01..Z12`) to fail CI if any section falls back to `Recents` or leaves a button unwired.

### F3 — Spatial Rhythm, Concentric Radii & Inspector Chrome (`P3`, `P4`, `P5`)
- **Finding:** File cards used mismatched outer/inner corner radii, section headers concatenated item counts (`"Layers (12)"`), inspector tabs varied in height, and coordinates used proportional fonts.
- **Fix (`crates/designos-app/src/panels/right.rs`, `crates/designos-app/src/home/mod.rs`):**
  - Enforced `24×24px` minimum pointer floor (`WCAG 2.5.8 AA`), `28px` standard control height (`D12`), and uniform `28px` segmented header tabs (`D116`).
  - Applied concentric radii (`8px` outer card / `4px` inner thumbnail preview, `D92`).
  - Extracted item counts into separate `Badge/StatusCount` pills (`D85`) rendered in `JetBrains Mono` (`theme::font_mono()`, `D6`).
  - Added a dedicated `36px` right-inspector footer bar for status and zoom telemetry.

### F4 — Per-File Embedded Agent Terminal Craft (`P7`)
- **Finding:** Embedded CLI agent sessions needed strict per-file isolation, crisp box-drawing characters, and keyboard shortcut isolation from the canvas.
- **Fix (`crates/designos-app/src/terminal.rs`):** Built the `400px` resizable `Panel/AgentTerminal` with Orca Ghostty dark palette, procedural pixel-aligned box-drawing (`U+2500..U+257F`), bracketed paste (`\x1b[200~..\x1b[201~`), 3-tier live status dots, and automatic single-key canvas shortcut suppression while typing.
