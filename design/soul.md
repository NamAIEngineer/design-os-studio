---
status: ratified
---

# Design Soul — designos-studio

_Declared stance for `design:os studio`. Every `/ui:*` flow and studio agent reads this BEFORE picking personas, tokens, or layouts. Precedence: brief > soul (project > studio > factory) > memory prior > knowledge floors._

## Never

- Never use purple or violet (`#9747FF`, `#8755E9`, `#A259FF`) for component badges, focus rings, or gradients; components are strictly `#06B6D4` Cyan and primary brand actions are `#E36323` Orange — evidence: `crates/designos-app/src/theme.rs`
- Never hardcode inline `Color32::from_rgb` values in panels or views outside the central token registry — evidence: `crates/designos-app/src/theme.rs`
- Never silently fall back to `Recents` when a Home workspace section (`Starred`, `Trash`, `Templates`, `Design Systems`, `Drafts`) has zero matching items — evidence: `crates/designos-app/src/home/mod.rs`
- Never ship decorative stub buttons, dead search bars, or un-wired menu items that violate `Z01..Z12` — evidence: `scripts/zero_fake_ui_gate.py`
- Never concatenate item counts into section title strings like `"Layers (4)"`; always render counts in a separate `Badge/StatusCount` pill (`D85`) — evidence: `plans/reports/audit-261016-vsf-pcp-ui-craft.md`
- Never allow single-key canvas shortcuts (`V`, `F`, `R`, `T`, `Backspace`) to fire while the embedded PTY terminal or a text input has keyboard focus — evidence: `crates/designos-app/src/terminal.rs`

## Always

- Always enforce the 4-step Zinc dark surface hierarchy (`#121215` canvas, `#18181B` panel chrome, `#1E1E22` card/surface, `#232329` hover) separated by `1px` `#27272A` borders — evidence: `crates/designos-app/src/theme.rs`
- Always keep interactive pointer targets at or above `24×24px` (`WCAG 2.5.8 AA`) with `28px` standard control height (`D12`) and uniform `28px` inspector tab strips (`D116`) — evidence: `plans/reports/audit-261016-vsf-pcp-ui-craft.md`
- Always render numeric coordinates, dimensions, hex codes, token paths, and frame telemetry in `JetBrains Mono` (`theme::font_mono()`, `D6`) — evidence: `crates/designos-app/src/theme.rs`
- Always apply concentric border radii (`r_outer = r_inner + padding`, `8px` outer card / `4px` inner preview, `D92`) on file thumbnails and template cards — evidence: `crates/designos-app/src/home/mod.rs`
- Always isolate embedded CLI agents per open document in the `400px` `Panel/AgentTerminal` with procedural box-drawing and 3-tier live status dots — evidence: `crates/designos-app/src/terminal.rs`
- Always preserve the 60fps viewport floor via `SceneIndex` flat SoA culling and incremental `DirtyLog` O(k) GPU sync — evidence: `plans/reports/perf-261016-figma-wasm-re.md`

## Voice

- Direct, technical, zero-filler studio nomenclature (`Frame`, `Component`, `Instance`, `Token`, `DirtyLog`, `Inspect`) with exact unit suffixes (`px`, `%`, `deg`, `ms`) — inherited: `crates/designos-app/src/theme.rs`
- Truthful empty and status states that name the exact section or action required without marketing fluff — evidence: `crates/designos-app/src/home/mod.rs`

## Evidence pointers

- `crates/designos-app/src/theme.rs` — canonical color, font, spacing, and radius constants mirrored to `design/design.tokens.json`
- `crates/designos-app/src/home/mod.rs` — truthful per-section filtering, concentric card thumbnails, and interactive starter templates
- `crates/designos-app/src/terminal.rs` — per-file VTE PTY agent terminal, Ghostty palette, procedural box-drawing, and shortcut suppression
- `scripts/zero_fake_ui_gate.py` — automated `Z01..Z12` static & behavioral gate blocking dead controls
- `plans/reports/audit-261016-vsf-pcp-ui-craft.md` — VSF-PCP 124-rule UI craft audit and remediation ledger
- `plans/reports/perf-261016-figma-wasm-re.md` — Figma C++/WASM reverse-engineering performance architecture report
