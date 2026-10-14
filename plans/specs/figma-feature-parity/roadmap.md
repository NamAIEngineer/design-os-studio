# Figma vs. DESIGN:OS Studio — Feature Parity & Gap Matrix

**Status**: Authoritative Architecture & Parity Specification  
**Scope**: `designos-doc`, `designos-fig`, `designos-export`, `designos-render`, `designos-gpu`, `designos-app`, `designos-mcp`  
**Target Comparison**: Production Figma (2025/2026) vs. Native Rust DESIGN:OS Studio

---

## Executive Summary Matrix

| Domain | Production Figma (2025/2026) | DESIGN:OS Studio Status | Primary Crate / Module Path |
| :--- | :--- | :--- | :--- |
| **Core Scene & Vector Networks** | Frames, Sections, Groups, Shapes, Vector Networks, Boolean Ops, Shape Builder | **100% Native Rust** | `crates/designos-doc/src/node.rs`, `vector.rs`, `crates/designos-app/src/canvas/builder.rs` |
| **Auto Layout v4 (1D + Wrap)** | Horizontal/Vertical, Wrap, Hug/Fill/Fixed, Min/Max, Absolute, Baseline, Negative Gap | **100% Native Rust** | `crates/designos-app/src/layout.rs`, `crates/designos-doc/src/node.rs` |
| **Components, Variants & Props** | Component, ComponentSet, Instance, 15 OverrideFields, Bool/Text/Variant/InstanceSwap, Libraries | **100% Native Rust** | `crates/designos-doc/src/components.rs`, `library_ops.rs` |
| **Variables & Modes** | Color/Number/String/Boolean, Multi-Mode Collections, Aliases, Cycle Detection, W3C Tokens | **100% Native Rust** | `crates/designos-doc/src/variables.rs`, `variables_apply.rs`, `variables_edit.rs` |
| **Effects & Paints** | Solid, 4 Gradients, Image Fills + Color Grading, Shadows, Blurs, Frosted Glass, Noise/Texture | **100% Native Rust** | `crates/designos-doc/src/paint.rs`, `crates/designos-render/`, `crates/designos-gpu/` |
| **Prototyping & Present Mode** | 11 Triggers (incl. Key/Gamepad), 7 Actions, Smart Animate, Spring/Bezier Easing, Runtime Vars | **100% Native Rust** | `crates/designos-doc/src/prototype.rs`, `crates/designos-app/src/present/` |
| **Multi-Mode Surface & MCP** | Design, FigJam Whiteboard, Slides, Dev Mode (5 languages + Redlines + Diff), 136-Tool MCP | **100% Native Rust** | `crates/designos-app/src/`, `crates/designos-export/src/`, `crates/designos-mcp/` |
| **Binary `.fig` Export** | Native `.fig` Kiwi container read/write | **P0 Gap (Import Only)** | `crates/designos-fig/src/fig.rs`, `kiwi.rs`, `export.rs` |
| **CSS Grid (2D Auto-Layout)** | 2D Grid track sizing (`fr`, row/col span) | **P0 Gap (1D + Wrap Only)** | `crates/designos-app/src/layout.rs` |
| **Real-Time Multiplayer** | Multi-cursor CRDT / WebSocket sync | **P1 Gap (Local + MCP)** | `crates/designos-doc/src/history.rs` |
| **Component Slots & Variable Scopes** | Native Instance Slot holes & binding picker scope filtering | **P1 Gap (Partial)** | `crates/designos-doc/src/components.rs`, `variables.rs` |
| **Variable Expressions AST** | Full arithmetic/string expression formulas in `SetVariable` / `Conditional` | **P1 Gap (Partial)** | `crates/designos-doc/src/prototype_expr.rs`, `crates/designos-app/src/protovars.rs` |
| **OpenType & Variable Font Axes** | Arbitrary OpenType features (`liga`, `ss01`, `tnum`) & continuous `wght`/`opsz`/`wdth` axes | **P2 Gap** | `crates/designos-doc/src/textmodel.rs`, `crates/designos-app/src/fonts.rs` |
| **Native DESIGN:OS AI Bridge** | In-canvas AI prompt bar, EaseUI + Laya/JEV integration, `vsf-reviewer` gate overlay | **P2 Gap** | `crates/designos-app/src/panels/`, `crates/designos-mcp/` |

---

## Section 1: Already Implemented in DESIGN:OS Studio (100% Native Rust)

### 1. Core Scene & Shapes
- **Crate & File Paths**:
  - `crates/designos-doc/src/lib.rs` (`Document`, `NodeId`, `Style`, `StyleMeta`, `walk`, `page_node_count`, `prune_unreachable`)
  - `crates/designos-doc/src/node.rs` (`Node`, `NodeKind`, `FrameProps`, `ArcData`, `BooleanOp`, `BooleanKind`)
  - `crates/designos-doc/src/vector.rs` (`VectorNetwork`, `HandleMirror`, vertices, cubic Bezier segments, per-vertex corner radius, filled regions)
  - `crates/designos-doc/src/vector_fit.rs` (`fit_polyline`, `Anchor` curve fitting for freehand/pencil paths)
  - `crates/designos-app/src/canvas/builder.rs` (Interactive Shape Builder tool for region merging/subtraction)
  - `crates/designos-app/src/vecops.rs` (Planar boolean geometry operations and path offsetting)
- **Implemented Capabilities**:
  - Full scene hierarchy with `Page`, `Frame`, `Section`, `Group`, `Rectangle` (independent 4-corner radii + iOS corner smoothing `0..1`), `Ellipse` (start/sweep angle + inner donut ratio via `ArcData`), `Polygon`, `Star`, `Line`, and `Text`.
  - True non-hierarchical `VectorNetwork` graph representation with per-vertex corner rounding and enclosed region fills (`VectorNetwork::to_paths`).
  - Non-destructive `BooleanOp` (`Union`, `Subtract`, `Intersect`, `Exclude`) plus interactive Shape Builder (`canvas/builder.rs`).
  - Document hygiene and telemetry helpers in `crates/designos-doc/src/lib.rs`:
    - `Document::prune_unreachable(&mut self) -> usize`: mark-and-sweep reachability from `self.pages` across `n.children`, `NodeKind::Instance { component: Some(cid), .. }`, `ComponentLink::source`, and `PropValue::Instance(cid)` / `PropKind::InstanceSwap { preferred }`.
    - `Document::page_node_count(&self, page: NodeId) -> usize`: $O(|T_{\text{page}}|)$ subtree node counter for UI telemetry and lazy page inspection.

### 2. Auto Layout v4
- **Crate & File Paths**:
  - `crates/designos-app/src/layout.rs` (`compute`, `ChildIn`, `LayoutOut`, `relayout_all`)
  - `crates/designos-doc/src/node.rs` (`AutoLayout`, `LayoutDirection`, `Sizing`, `LayoutAlign`, `LayoutChild`, `Constraints`)
- **Implemented Capabilities**:
  - `Horizontal` and `Vertical` primary axis flow with multi-line `wrap` (`AutoLayout::wrap`) and `counter_item_spacing`.
  - `Fixed`, `Hug`, and `Fill` sizing modes (`Sizing`) on both primary and counter axes.
  - Distribution and alignment: `Start`, `Center`, `End`, `SpaceBetween`, and text `Baseline` alignment (`LayoutAlign`).
  - Per-node `min_width`, `max_width`, `min_height`, `max_height` constraints enforced during layout solving.
  - `LayoutChild::absolute` out-of-flow positioning with pin `Constraints`, `reverse_z` (first-on-top stacking), and `stroke_included` box-sizing.

### 3. Components, Variants & Properties
- **Crate & File Paths**:
  - `crates/designos-doc/src/components.rs` (`OverrideField::ALL`, `instance_component`, `instances_of`, component creation, instance sync, detach, reset overrides, variant switching)
  - `crates/designos-doc/src/node.rs` (`ComponentLink`, `OverrideField`, `ComponentPropDef`, `PropKind`, `PropValue`)
  - `crates/designos-doc/src/library.rs` & `crates/designos-doc/src/library_ops.rs` (`LibraryState`, `LibrarySnapshot`, `SnapshotAsset`, `import_asset`, `pending_updates`, `apply_updates`)
  - `crates/designos-app/src/comp_ops.rs` & `crates/designos-app/src/libraries.rs`
- **Implemented Capabilities**:
  - `NodeKind::Component`, `NodeKind::ComponentSet` (variant matrix), and `NodeKind::Instance`.
  - Fine-grained override tracking across all 15 `OverrideField` variants (`Name`, `Visible`, `Locked`, `Opacity`, `Blend`, `Size`, `Transform`, `Fills`, `Stroke`, `Effects`, `CornerRadii`, `Characters`, `TextStyle`, `Layout`, `MainComponent`).
  - Typed component property definitions (`Boolean`, `Text`, `Variant { options }`, `InstanceSwap { preferred }`) and property bindings (`ComponentLink::bindings`).
  - Local and external published library snapshots (`LibrarySnapshot`) with FNV content fingerprinting, dependency resolution (`SnapshotAsset::deps`), hidden `Document::library_page` storage, diffing (`pending_updates`), and non-destructive update application (`apply_updates`).

### 4. Variables & Modes
- **Crate & File Paths**:
  - `crates/designos-doc/src/variables.rs` (`Variable`, `VariableCollection`, `VariableMode`, `VariableKind`, `VariableValue`, `NodeVariables`, `MAX_ALIAS_DEPTH`)
  - `crates/designos-doc/src/variables_apply.rs` (`resolve_node`, mode inheritance, bound property resolution)
  - `crates/designos-doc/src/variables_edit.rs` (CRUD operations, W3C Design Tokens JSON import/export)
  - `crates/designos-fig/src/variables.rs` (`import_variables_json`, `export_variables_json`)
  - `crates/designos-app/src/variables.rs` (Variables modal table, mode columns, inspector binding UI)
- **Implemented Capabilities**:
  - `Color`, `Number`, `String`, and `Boolean` variable types organized into multi-mode `VariableCollection`s.
  - `VariableValue::Alias(String)` chains with deterministic cycle protection (`MAX_ALIAS_DEPTH = 16`).
  - Ancestor mode inheritance (`Document::variable_mode`) via `NodeVariables::modes`.
  - Node property bindings (`NodeVariables::bindings`) covering fills, strokes, dimensions, spacing, corner radii, opacity, visibility, text content, and typography fields.
  - W3C Design Tokens (`$value` / `$type`) and Figma Variables REST JSON import/export.

### 5. Effects & Paints
- **Crate & File Paths**:
  - `crates/designos-doc/src/paint.rs` (`Color`, `BlendMode`, `Paint`, `PaintKind`, `GradientStop`, `ImageScaleMode`, `ImageFilter`, `Stroke`, `Effect`, `EffectKind`)
  - `crates/designos-render/src/` (CPU reference rasterizer and effect pipeline)
  - `crates/designos-gpu/src/` (WGPU hardware-accelerated renderer and shader pipelines)
- **Implemented Capabilities**:
  - 19 `BlendMode` compositing modes (`Normal`, `PassThrough`, `Multiply`, `Screen`, `Overlay`, `SoftLight`, `HardLight`, `ColorDodge`, `ColorBurn`, etc.).
  - `Solid`, `LinearGradient`, `RadialGradient`, `AngularGradient`, and `DiamondGradient` fills and strokes, plus per-side stroke weights (`side_weights`), caps, joins, miter limits, and dash arrays.
  - `Image` fills supporting `Fill`, `Fit`, `Crop`, and `Tile` scale modes with non-destructive `ImageFilter` adjustments (`exposure`, `contrast`, `saturation`, `temperature`, `tint`, `highlights`, `shadows`).
  - Multi-pass `Effect` stack: `DropShadow` (with `spread`), `InnerShadow`, `LayerBlur`, `BackgroundBlur`, `FrostedGlass` refraction, `ProgressiveBlur`, `Noise` (Mono/Duo/Multi), and `Texture` displacement.

### 6. Prototyping & Present Mode
- **Crate & File Paths**:
  - `crates/designos-doc/src/prototype.rs` (`PrototypeProps`, `Interaction`, `Trigger`, `Action`, `NavigateKind`, `Transition`, `TransitionKind`, `Easing`, `Flow`, `PrototypeDevice`)
  - `crates/designos-doc/src/prototype_expr.rs` (Prototype expression types and evaluator)
  - `crates/designos-app/src/present/` & `crates/designos-app/src/proto.rs` & `crates/designos-app/src/protovars.rs`
- **Implemented Capabilities**:
  - 11 Triggers: `OnClick`, `OnDrag`, `WhileHovering`, `WhilePressing`, `MouseEnter`, `MouseLeave`, `MouseDown`, `MouseUp`, `AfterDelay`, `KeyPress`, and `GamepadPress` (`Xbox`, `PlayStation`, `SwitchPro`, `Generic`).
  - Actions: `Navigate` (`NavigateTo`, `SwapOverlay`, `OpenOverlay`, `ScrollTo`), `Back`, `CloseOverlay`, `OpenUrl`, `SetVariable`, and `Conditional` (`if/else` action branches).
  - Transitions & Physics: `Dissolve`, `SmartAnimate` (matching layer names across frames), `MoveIn`, `MoveOut`, `Push`, `SlideIn`, `SlideOut` with `Bezier([f32; 4])` and physical `Spring { mass, stiffness, damping }` solvers.
  - Interactive Present Mode window with device frames (`PrototypeDevice`), flow switcher, overflow scrolling, and live runtime variable inspection.

### 7. Multi-Mode Product Surface
- **Crate & File Paths**:
  - **Design Editor**: `crates/designos-app/src/app.rs`, `canvas/`, `panels/`
  - **FigJam Whiteboard (`DocumentKind::Board`)**: `crates/designos-doc/src/board.rs`, `crates/designos-app/src/board.rs` (`Sticky`, orthogonal routing `Connector`, `Stamp`, `ShapeWithText`)
  - **Slides (`Deck`)**: `crates/designos-doc/src/node.rs` (`SlideProps`), `crates/designos-app/src/decks.rs`, `crates/designos-export/src/pdf.rs` (`export_pdf`)
  - **Dev Mode & Codegen**: `crates/designos-doc/src/dev.rs` (`DevInfo`, `DevStatus`, annotations), `crates/designos-app/src/dev.rs` (redline distance measurements, version diff compare), `crates/designos-export/src/{css,tailwind,swiftui,compose,react}.rs`
  - **136-Tool MCP Server**: `crates/designos-mcp/` (`designos-mcp` binary exposing full scene, layout, variable, component, prototyping, and export tools over JSON-RPC)

---

## Section 2: Missing / Partial Features Compared to Production Figma (2025/2026) — Prioritized Roadmap

### 1. P0 — Binary `.fig` Export (Currently `.fig` Import Only)
- **Current Evidence**:
  - `crates/designos-fig/src/fig.rs` and `crates/designos-fig/src/kiwi.rs` decode binary `.fig` containers (`fig-kiwi` / zip-wrapped `canvas.fig` + embedded `images/`) into `designos_doc::Document` (`import_fig`, `decode_fig_to_json`).
  - `crates/designos-fig/src/export.rs` exports Figma REST API JSON (`export_rest_json`), and `crates/designos-export/src/lib.rs` exports `PNG`, `JPG`, `SVG`, and `PDF`, while `designos-doc` saves native `.designos` JSON.
- **Gap**:
  - Cannot serialize a `designos_doc::Document` back into a binary `.fig` Kiwi container (`fig-kiwi` header + schema chunk + Zstd/Deflate node changes payload + bundled images) for direct drag-and-drop import into desktop/web Figma without a plugin.
- **Implementation Plan**:
  - Add `crates/designos-fig/src/fig_encode.rs` implementing a Kiwi binary encoder matching Figma's `Message` / `NodeChange` schema (`GUID`, `Phase::CREATED`, `parentIndex`, `fillPaints`, `strokePaints`, `symbolData`, `derivedSymbolData`) and zip container bundler (`export_fig(&Document) -> Vec<u8>`).

### 2. P0 — CSS Grid / Auto-Layout 2D Grid Track Sizing (Figma 2025 Grid)
- **Current Evidence**:
  - `crates/designos-app/src/layout.rs` (`compute`) solves 1D `Horizontal` and `Vertical` flex-style layouts plus 1D line wrapping (`AutoLayout::wrap`).
  - `crates/designos-doc/src/node.rs` (`LayoutGrid`) provides visual guide overlays (`Columns`, `Rows`, `Grid`), which do not position children.
- **Gap**:
  - Production Figma (2025+) includes a 2D Auto-Layout `Grid` mode with explicit row/column track definitions (`Fixed(px)`, `Flex(fr)`, `Hug`), cell placement `(row, col)`, and `row_span` / `col_span` on `LayoutChild`.
- **Implementation Plan**:
  - Extend `LayoutDirection` (or `AutoLayout`) in `crates/designos-doc/src/node.rs` with `Grid { columns: Vec<GridTrack>, rows: Vec<GridTrack>, column_gap: f32, row_gap: f32 }` and add `grid_row`, `grid_column`, `row_span`, `col_span` to `LayoutChild`.
  - Implement 2D track sizing and span resolution in `crates/designos-app/src/layout.rs` and emit CSS Grid (`display: grid; grid-template-columns: ...`) in `crates/designos-export/src/css.rs` and `tailwind.rs`.

### 3. P1 — Real-Time Multiplayer Collaboration (CRDT / WebSocket Sync)
- **Current Evidence**:
  - `crates/designos-doc/src/history.rs` and `crates/designos-app/src/history.rs` implement single-user undo/redo command logs (`Command::Insert`, `Remove`, `Update`, `Reparent`, `Batch`) and named version snapshots.
  - `crates/designos-mcp/` provides live local IPC/MCP mutation from AI agents.
- **Gap**:
  - No multi-user network transport, live peer cursors/selections, or Last-Write-Wins (LWW) per-property CRDT map reconciliation (`yrs` or Figma-style fractional-index + property-map sync).
- **Implementation Plan**:
  - Introduce `crates/designos-sync/` with a fractional-index tree ordering + per-node LWW property register clock, WebSocket client/relay protocol, and peer presence overlays (remote cursors, viewport follow, live selection rings) in `crates/designos-app/src/canvas/`.

### 4. P1 — Component Slots & Scoped Variable Publishing
- **Current Evidence**:
  - `crates/designos-doc/src/components.rs` supports `Boolean`, `Text`, `Variant`, and `InstanceSwap` properties (`PropKind`), and `crates/designos-doc/src/variables.rs` stores `Variable::scopes: Vec<String>` and `hidden_from_publishing: bool`.
- **Gap**:
  - **Component Slots**: Instances cannot accept arbitrary user-inserted child subtrees inside designated `Slot` container holes without detaching the instance.
  - **Variable Scope Filtering**: While `Variable::scopes` is stored and round-tripped in `designos-doc`, the inspector variable picker in `crates/designos-app/src/variables.rs` does not yet filter candidate variables by target property scope (`ALL_FILLS`, `STROKE_COLOR`, `CORNER_RADIUS`, `GAP`, `WIDTH_HEIGHT`, `TEXT_CONTENT`, `FONT_SIZE`, etc.).
- **Implementation Plan**:
  - Add `PropKind::Slot` and allow `NodeKind::Instance` children under a slot-bound container frame to bypass main-component structural overwrite during `sync_instance`.
  - Enforce `Variable::scopes` filtering in `crates/designos-app/src/variables.rs` and `library_ops.rs` (`hidden_from_publishing` exclusion option).

### 5. P1 — Variable Expressions & Conditional Logic AST
- **Current Evidence**:
  - `crates/designos-doc/src/prototype.rs` and `crates/designos-doc/src/prototype_expr.rs` support `Action::SetVariable` and `Action::Conditional` with basic expression evaluation.
- **Gap**:
  - Complex multi-operator expressions in Figma Prototyping (nested boolean short-circuiting, string interpolation/concatenation with type coercion, component property reads inside prototype expressions, and multiple sequential actions per trigger without wrapper batches) have partial UX coverage in the prototype inspector (`crates/designos-app/src/proto.rs`).
- **Implementation Plan**:
  - Expand the expression builder UI in `crates/designos-app/src/proto.rs` to provide visual token pills, multi-operand arithmetic/boolean/string formulas, and full type-checking diagnostics in the interaction editor.

### 6. P2 — Advanced Typography & Variable Font Axes (`wght`, `opsz`, `wdth`)
- **Current Evidence**:
  - `crates/designos-doc/src/textmodel.rs` and `crates/designos-app/src/fonts.rs` support rich-text `TextRun` spans, bulleted/numbered lists, `max_lines` truncation, leading trim (`TextTrim`), paragraph spacing/indent, and on-demand Google Fonts / Fontsource downloading.
- **Gap**:
  - No continuous variable-font variation axis overrides (`font-variation-settings`: `wght`, `wdth`, `opsz`, `slnt`, `GRAD`) or granular OpenType feature tag toggles (`liga`, `calt`, `tnum`, `onum`, `ss01`–`ss20`, `smcp`) in `TextStyle` and the typography inspector.
- **Implementation Plan**:
  - Add `opentype_features: BTreeMap<[u8; 4], u32>` and `variation_axes: BTreeMap<[u8; 4], f32>` to `TextStyle` in `crates/designos-doc/src/textmodel.rs`, wire them into cosmic-text / swash shaping in `designos-render`, and expose OpenType / Variable Font sliders in `crates/designos-app/src/panels/`.

### 7. P2 — Native DESIGN:OS AI & Generative UI Bridge (EaseUI + Laya/JEV Integration)
- **Current Evidence**:
  - External AI orchestration works today via `crates/designos-mcp/` (136 MCP tools).
- **Gap**:
  - No native in-canvas `Cmd+K` AI prompt bar or direct integration with the DESIGN:OS Generative UI stack (`EaseUI`, `Laya-MLX` sub-10ms local classifier, `JEV` typed judgment, and `vsf-reviewer` automated visual/design-gate scoring).
- **Implementation Plan**:
  - Add an in-canvas `Cmd+K` AI command bar in `crates/designos-app/src/` supporting:
    1. **Generate Variant / Screen**: streaming Zod-validated component trees directly into `designos_doc::Command` batches.
    2. **Extract Design System**: auto-clustering unstyled colors/type into `VariableCollection`s and `Component`s.
    3. **Bidirectional HTML/React <-> Canvas Cast**: one-click sync with `designos-export`.
    4. **Live `vsf-reviewer` Gate Overlay**: on-canvas badge audit for spacing rhythm, contrast, tap targets, and token compliance.
