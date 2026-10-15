# Wave 6: Unified Phosphor Icons System (`DIR-19`) & 1:1 Figma Visual/Userflow Closure (`DIR-20`)

- **Date**: 2026-10-15
- **Directives**: `DIR-19`, `DIR-20`
- **Epics / Stories / Rules**: `EP-12`, `EP-13`, `EP-14` · `US-12`, `US-13`, `US-14` · `REQ-14..REQ-17` · `BR-19..BR-22` · `D14..D17`

## Summary of Architecture & Decisions
1. **Unified Official Phosphor Icons (`DIR-19` / `BR-22` / `D17`)**:
   - Vendored 108 official `@phosphor-icons/core@2.1.1` regular SVGs (`viewBox="0 0 256 256"`, MIT license) in `crates/designos-app/assets/icons/phosphor/`.
   - Mapped all 116 `Icon` variants in `crates/designos-app/src/icons.rs` via `Icon::phosphor_name(self)` and `PHOSPHOR` table (`resvg` rasterizer with DPR-aware `(Icon, physical_px)` texture cache and `round_to_pixel` snapping), while preserving `Icon::Logo`.
   - Eliminated ad-hoc procedural icon drawing across `rail.rs`, `assets.rs`, `right.rs`, `tabs.rs`, and `terminal.rs`.
2. **1:1 Figma Visual Micro-Gaps & Userflows Closure (`DIR-20`)**:
   - **Gap 1 (`toolbar.rs`)**: Right mode switcher pill upgraded to 3 Figma UI3 segments (`Draw` `Icon::Pencil`, `Design` `Icon::Design`, `Dev mode` `Icon::Dev`).
   - **Gap 2 (`assets.rs`)**: Header `Icon::BookOpen`, `ListView`/`GridView` toggle, and vertical `Created in this file` library card (`52px` `#2C2C2C` preview thumbnail box on top, title + `"N component"` subtitle below) above `"Add more libraries"`.
   - **Gap 3 (`left.rs`)**: Rendered `"Team project"` subtitle under file name in `file_header` and added `"layers-collapse-all"` (`Collapse layers`) button next to search on the `Layers` header row.
   - **Gap 4 (`menus.rs`)**: Added `"AI balance"` and `"Help and account"` to `main_nodes` while keeping `"Help"` path lookup compatibility.
   - **Gap 5 (`rail.rs`)**: Clicking `File` returns in 1 click from any other rail tab (`Agents`, `Assets`, `Specs`) to `File` (`Pages & Layers`), and toggles `app.ui.left_collapsed` when clicked while already on `File`.
   - **Userflows `UF-01..UF-08` (`shell/tests/home.rs`)**: Verified end-to-end by `userflows_uf01_to_uf08_end_to_end_figma_parity` (`822 passed; 0 failed` in `designos-app`, `387/387` = `100.00%` in `scripts/figma_parity_meter.py`).
