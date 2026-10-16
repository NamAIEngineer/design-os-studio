# harvest-extract-v1

You are extracting durable lessons from a finished piece of work. You did not do this
work and have no stake in it looking good — be a skeptical outside reader, not a
cheerleader.

You will be given one or more end-of-phase reports, each fenced between
`--- REPORT: <path> ---` and `--- END REPORT: <path> --- `.

## Output

Respond with **raw JSON only** — no prose before or after, no markdown fence required but
allowed. Shape:

```json
{"v": 1, "candidates": [
  {
    "kind": "insight",
    "text": "...",
    "evidence": "...",
    "source": "<path exactly as given in the REPORT header>",
    "durable": true,
    "actionable": true,
    "confidence": 0.8,
    "target": "taste-rubric.md#motion",
    "gapKind": "rubric-gap"
  }
]}
```

`target` and `gapKind` are required when `kind` is `"gap"`; omit them for `"insight"`.

## Fields

- **`evidence` must be copied VERBATIM from the report you read it in** — a candidate whose
  `evidence` is not a literal quote of the source text will be discarded by a deterministic
  gate before it ever reaches a human. This is not a soft preference; quote exactly.
- **`durable`**: would this help on a *different* project, not just this one? A one-off
  fact about this run ("took 2 hours", "ran npm test, all green") is not durable.
- **`actionable`**: does it change what someone would DO differently next time? A
  restated todo-list item is not actionable — it is a description, not a lesson.
- **`kind`**: `"insight"` is a durable lesson worth remembering. `"gap"` is knowledge
  missing from the studio's shared `knowledge/` — set `target` to `<file>.md[#section]`
  and `gapKind` to one of: `rubric-gap`, `persona-gap`, `recipe-gap`, `benchmark-stale`,
  `guardrail-lesson`. A taste observation is a `gap` with `gapKind: "rubric-gap"`.
- **`confidence`**: 0.0–1.0, your honest estimate that this candidate survives scrutiny.

## Discipline

- **Prefer none to noise.** An empty `candidates` array is a valid, respected answer — do
  not manufacture a lesson to have something to say.
- **Never** write files, never run commands, never emit prose outside the JSON object.

--- REPORT: plans/reports/audit-261016-vsf-pcp-ui-craft.md ---
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

--- END REPORT: plans/reports/audit-261016-vsf-pcp-ui-craft.md ---

--- REPORT: plans/reports/perf-261016-figma-wasm-re.md ---
# Performance & Architecture Report: Figma C++/WASM & Orca Terminal Reverse Engineering (`261016`)

- **Date:** 2026-10-16
- **Project:** `designos-studio` (`/Users/jangtrinh/Products/design-os-studio`)
- **Scope:** `crates/designos-core`, `crates/designos-canvas`, `crates/designos-app/src/terminal.rs`

## 1. Figma C++/WASM Engine Reverse-Engineering Lessons (`P8`)
By inspecting production Figma WASM/WebGPU symbols and frame-pipeline invariants, we distilled four core architectural mechanisms into `design:os studio` to hold a locked 60fps (`16.6ms` frame budget) on large documents:

1. **Flat SoA Spatial Indexing (`SceneIndex`):**
   - Instead of pointer-chasing deep Rust tree nodes during pan/zoom culling, `SceneIndex` packs world-space AABBs (`min_x`, `min_y`, `max_x`, `max_y`), visibility flags, and page membership into contiguous Structure-of-Arrays (SoA) slices for cache-friendly frustum culling.
2. **Incremental `DirtyLog` O(k) GPU Synchronization:**
   - Canvas mutations append node IDs to a bounded `DirtyLog` rather than rebuilding the full vertex/instance buffer every frame. Frame preparation updates only the `k` dirty node slots (`O(k)` vs `O(N)`).
3. **Lazy Per-Page Component Instance Expansion:**
   - Master component changes invalidate dependent instances lazily on the active page first; background pages defer instance tree reconciliation until activated.
4. **Off-Main-Thread Image Decode & Texture Upload:**
   - Raster fills decode asynchronously on background worker threads and stream into the GPU texture atlas without stalling the egui/wgpu UI thread.

## 2. Orca Ghostty Terminal Reverse-Engineering Lessons (`P7`)
By analyzing Orca's native Ghostty terminal integration, we upgraded `crates/designos-app/src/terminal.rs`:
- **Per-File Session Isolation:** Each document tab owns its own `TerminalSession` map keyed by document ID so multi-agent workflows never cross-contaminate state or working directories.
- **Procedural Box-Drawing (`U+2500..U+257F`):** Font glyphs for TUI borders often leave subpixel seams at fractional DPI; rendering box-drawing ranges procedurally as crisp pixel-snapped quads eliminates gaps in Claude Code and `design-os` CLI tables.
- **Canvas Shortcut Suppression:** When `Panel/AgentTerminal` holds keyboard focus, single-key tool shortcuts (`V`, `F`, `R`, `T`, `Backspace`, `Space`) are routed exclusively to the PTY input stream.

--- END REPORT: plans/reports/perf-261016-figma-wasm-re.md ---
