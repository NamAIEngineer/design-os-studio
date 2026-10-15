# Phase 5 Implementation Plan — Parallel Execution Matrix

## Parallel Workstreams

### Workstream A (`core-figma-parity`) — `designos-doc`, `designos-fig`, `designos-render`, `designos-export`
- [x] **Task A1**: Add `LayoutDirection::Grid`, `GridTrack`, `grid_columns`, `grid_rows`, `col_span`, `row_span` to `crates/designos-doc/src/node.rs` & `spec.rs` (keeping `AutoLayout` and `LayoutChild` `Copy`).
- [x] **Task A2**: Add `PropKind::Slot`, `PropValue::Slot`, `OverrideField::SlotContent` in `node.rs` & `components.rs`, and `Variable::matches_scope` in `variables.rs`.
- [x] **Task A3**: Add `TextStyle::variation_axes` (`wght`, `wdth`, `opsz`, `slnt`) & OpenType feature helpers in `paint.rs`, wire into `designos-render/src/text.rs` and `designos-export/src/{css,tailwind,react}.rs`.
- [x] **Task A4**: Upgrade `crates/designos-fig/src/{fig_export.rs,fig.rs,testkit.rs}` to round-trip `symbolOverrides`, `componentPropDefs`, `componentPropRefs`, `fontVariations`, `toggledOnOTFeatures`, and `stackMode: "GRID"`, verified by `crates/designos-fig/tests/fig_roundtrip.rs`.

### Workstream B (`agent-terminal-and-cmdk`) — `designos-app` Independent Modules
- [x] **Task B1**: Add `portable-pty = "0.8"` and `vt100 = "0.15"` to `crates/designos-app/Cargo.toml` + `terminal.svg` & `grid-2x2.svg`.
- [x] **Task B2**: Implement `crates/designos-app/src/terminal.rs` — real PTY multi-tab runtime (`+ Shell`, `+ Claude CLI`, `+ Codex CLI`, `+ Grok CLI`) with `DESIGNOS_*` env injection, VT100 TrueColor screen renderer, raw keyboard/paste forwarding, Selection & Pin-Comment stdin bridge, and JSON `NodeSpec` stdout extractor.
- [x] **Task B3**: Implement `crates/designos-app/src/cmdk.rs` — floating In-Canvas `Cmd+K` Agent Bar (`synthesize_spec_from_prompt`) + Live VSF/EaseUI Design Gate Audit (`run_design_gate_audit`) with 1-click Auto-Fix.

### Workstream C (`studio-integration-and-qa`) — UI Wiring & GPU Visual Verification
- [x] **Task C1**: Wire `LayoutDirection::Grid` solver into `crates/designos-app/src/layout.rs` and expose Grid / Variable Axes / OpenType / Scoped Variable controls in `crates/designos-app/src/panels/inspector/`.
- [x] **Task C2**: Wire `RailItem::Terminal` (`--panel terminal` + bottom split dock `Ctrl+\``) and `Cmd+K` (`cmdk::show`) into `crates/designos-app/src/{app.rs,shell.rs,cli.rs,icons.rs,panels/rail.rs,panels/left.rs}`.
- [x] **Task C3**: Run full workspace tests (`cargo test --workspace`) and capture GPU screenshots of the Embedded PTY Agent Terminal, In-Canvas `Cmd+K` Design Gate, and 2D CSS Grid Inspector.
