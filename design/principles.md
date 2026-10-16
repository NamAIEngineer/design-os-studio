# design:os studio — Design Principles (P1–P8)

These 8 binding laws govern every surface, widget, token, and interaction across `design:os studio`. Every change to `crates/designos-app` or `design/` is audited against `P1..P8` and enforced by `ui ds a11y`, `ui ds soul check`, and `scripts/zero_fake_ui_gate.py`.

## P1 — Single Token Authority (`crates/designos-app/src/theme.rs` ↔ `design/design.tokens.json`)
All colors, typography scales, spatial increments, corner radii, and motion durations originate in `crates/designos-app/src/theme.rs` and are mirrored 1:1 into `design/design.tokens.json` (73 DTCG tokens across 5 categories, all 16 canonical roles mapped). Inline `Color32::from_rgb(...)` literals in panels or views outside `theme.rs` are forbidden unless explicitly allowlisted for semantic canvas overlays or GPU shader debug modes.

## P2 — Color Restraint, Zinc Dark Hierarchy & Strict Purple Ban
`design:os studio` uses a disciplined 4-step Zinc dark surface hierarchy (`#121215` Canvas < `#18181B` Chrome/Panel < `#1E1E22` Card/Input < `#232329` Hover < `#2E2E36` Active) with exactly four semantic accents:
- `#E36323` Brand Orange (`primary`, active tabs, focus rings, primary CTAs)
- `#0D99FF` Selection Blue (`info`, canvas bounding boxes, selection handles, snap guides)
- `#06B6D4` Cyan (`component`, reusable components, instances, and component sets)
- `#14AE5C` Dev Mode Emerald (`success`, code inspection, ready badges, live agent status)

**Strict Purple Ban:** Generic AI/Figma violet (`#9747FF`, `#8755E9`, `#A259FF`) is banned across all studio chrome and component badges.

## P3 — 8-pt Spatial Rhythm, `24×24px` Pointer Floor (`WCAG 2.5.8 AA`) & Same-Slot Same-Size (`D12`, `D116`)
All layout padding, gaps, and panel widths snap to the 4px/8px spatial grid (`4, 8, 12, 16, 24, 28, 32, 40, 48px`; Left Rail `56px`, Inspector `240px`, Agent Terminal `400px`). Every interactive target enforces a minimum `24×24px` hit box (`WCAG 2.5.8 AA`), with `28px` standard row/button height (`D12`) and uniform `28px` segmented header tabs (`D116`).

## P4 — Surface Discipline — Unified `#27272A` Borders, Concentric Radii (`D92`), Dedicated `36px` Inspector Footer
Adjacent dark surfaces are separated by crisp `1px` `#27272A` (`colors.border`) hairlines rather than muddy drop shadows. Nested containers obey concentric curvature (`r_outer = r_inner + padding`, e.g., `8px` card outer radius with `4px` inner thumbnail radius per `D92`). Status and zoom telemetry live in a dedicated `36px` right-inspector footer bar rather than floating over the canvas.

## P5 — Typography Scale, Monospace Technical Data (`D6` `theme::font_mono()`), Separate Count Badges (`D85`)
UI chrome uses `Inter` across a strict 6-step ramp (`9px` small badge, `10px` caption, `11px` body, `12px` title, `13px` section heading, `20px` home hero). All numeric coordinates (`X`, `Y`, `W`, `H`, rotation, opacity), hex color codes, token paths, and FPS/DirtyLog counters render in `JetBrains Mono` (`theme::font_mono()`, `D6`) for tabular scannability. Item counts render in separate pill badges (`Badge/StatusCount`, `D85`), never concatenated into label strings like `"Layers (12)"`.

## P6 — Zero Fake UI (`scripts/zero_fake_ui_gate.py` `Z01..Z12`) & Home Data Truth
Every visible button, tab, menu item, filter pill, and search input must execute real stateful logic verified by `scripts/zero_fake_ui_gate.py` (`Z01..Z12`). In the Home workspace (`crates/designos-app/src/home/mod.rs`), each navigation section (`Recents`, `Drafts`, `Starred`, `Templates`, `Design Systems`, `Trash`, `Archive`) queries its own truthful dataset and renders a dedicated empty state when empty — never silently falling back to `Recents`.

## P7 — Terminal-First Per-File Agent Isolation
Every open document owns an isolated PTY agent session inside the `400px` resizable `Panel/AgentTerminal` (`crates/designos-app/src/terminal.rs`), rendered with the Orca Ghostty dark palette, procedural pixel-aligned box-drawing glyphs (`U+2500..U+257F`), bracketed paste (`\x1b[200~..\x1b[201~`), 3-tier live status indicator dots (`Running`, `Idle`, `Exited`), and automatic canvas single-key shortcut suppression whenever the terminal or any text input holds focus.

## P8 — Figma C++/WASM 60fps Performance Floor
The viewport engine preserves a locked 60fps (`16.6ms` frame budget) on 100k+ node scenes via flat Structure-of-Arrays `SceneIndex` spatial indexing, incremental `DirtyLog` O(k) GPU instance buffer synchronization, lazy per-page component instance expansion, and off-main-thread background image decoding.
