---
id: glossary
type: glossary
title: "Domain Glossary & Banned Wording"
updated: 2026-10-15
---

## Terms

| Term | Anchor | Definition | Source |
|---|---|---|---|
| Spec Format v2 | spec-format-v2 | Canonical 11-stage specification bundle (`spec.manifest.json`, `pipeline.yaml`, `po-spec.md`, `clarifications.md`, `FC-001.md`, `PRD.md`, `US-01..09.md`, `W-01.md`, `P-01.md`, `tokens.md`, `test-plan.md`, `ui-contract.md`, `READY.md`, `review/gates/*.md`). | D3, D5 |
| Copy cho AI | copy-cho-ai | Deterministic Markdown citation block carrying exact source file path, line range, section breadcrumb, reader view, and project revision. | D4 |
| UI Contract | ui-contract | 12-column behavioral selector table mapping `Screen × State` (`S01..S09` × 6 states = 54 rows) to `US`, `Element`, `Components`, and `SCN`. | D5 |
| Kiwi v106 | kiwi-v106 | Figma binary container format (`fig-kiwi` header + DEFLATE schema + Zstd/DEFLATE message) preserving unknown schema fields in `raw_fields` across load/save cycles. | D1, D7 |
| 2D CSS Grid | css-grid-2d | Figma 2025+ Auto-Layout mode (`LayoutDirection::Grid`, `stackMode: "GRID"`) supporting `fr`, `px`, `auto` track sizing and `col_span` / `row_span` placement. | D1, D6 |
| Figma UI3 Tokens | figma-ui3-tokens | Canonical sRGB dark theme palette of `com.figma.Desktop` (`#1E1E1E` canvas, `#2C2C2C` panel, `#383838` input well, `#444444` border, `#0D99FF` selection blue, `#9747FF` component purple, `#24CB71` Dev Mode green). | D6 |
| REA Pipeline | rea-pipeline | Static and binary reverse-engineering workflow (`morluto/rea`) inspecting `/Applications/Figma.app` (`app.asar`) and `.fig` archives to document Kiwi schemas and UI3 tokens in `research/`. | D7 |
| Headless Background Capture | headless-capture | Non-intrusive verification mode (`with_visible(false).with_active(false)`) that renders `--screenshot` PNGs in the background without stealing macOS window focus. | D8 |
| Dual-Layer Auto-Update | dual-layer-auto-update | Two-tier evergreen update engine (`updater.rs` + `designos update`): Layer 1 atomic self-updater with SHA-256 verification and `.bak` rollback; Layer 2 passive read-only `/Applications/Figma.app` (`Info.plist` / `app.asar` / Kiwi `v106`) schema watchdog. | D9 |
| **design**:os studio Wordmark | brand-wordmark | Canonical brand typography rule: always all-lowercase **`design`:os studio** (or **`design`:os**) with **`design`** rendered in Bold (`700`) and `:os studio` rendered in Regular (`400`). | D10 |
| Multi-Tab Project Workspace | multi-tab-workspace | Top `36px` Figma UI3 Desktop Tab Strip (`S10`) managing multiple simultaneously open projects (`ProjectTab`) with isolated `Document`, `History` undo/redo, `Page`, `Selection`, `View` zoom/pan, `Dev Mode`, dirty dot `•`, and pin state. | D11 |

## Ban list

| Banned | Replacement | Reason |
|---|---|---|
| Emoji icons in UI chrome | `crate::icons::Icon` vector Lucide / Figma UI3 glyphs | Violates `es:designer` craft-floor §2 |
| Nested cards-in-cards | `1px` hairline divider (`#2A2B31` / `#3C3C3C`) or `#26272C` table header row | Violates `es:designer` craft-floor §2 |
| Unverified "seamless" claim | Cite concrete `SCN-xx` test and `CURRENT` gate digest | No unmeasured claims |
| Focus-stealing screenshot window | `ViewportBuilder::default().with_visible(false).with_active(false)` | Violates `BR-11` (`D8`) user screen protection rule |
| Unverified in-place binary overwrite | Atomic `.staged` download + `SHA-256` verify + `.bak` backup before rename | Violates `BR-13` (`D9`) zero-brick update safety rule |
| Uppercase or title-case brand (`DESIGN:OS Studio`, `Design OS Studio`, `DESIGN:OS`) | All-lowercase **`design`:os studio** (bold **`design`**, regular `:os studio`) | Violates `BR-14` (`D10`) brand typography directive |
| Shared global Undo/Selection across open tabs | Per-tab `ProjectTab` state isolation (`Document`, `History`, `Page`, `Selection`, `View`) | Violates `BR-15` (`D11`) multi-tab project isolation rule |
