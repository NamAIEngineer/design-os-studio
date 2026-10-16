---
id: UX-FP-001
type: ux-feature-pillars
title: "Figma Changelog 4 Feature Pillars — REA Evidence, State-First Specification & Document Engine Architecture (FP-01..FP-04)"
status: current
trace:
  req: [REQ-05, REQ-07, REQ-11, REQ-14, REQ-19]
  us: [US-07, US-09, US-14, US-18]
  skill: [es:micro-interactions, reverse-engineer-anything]
---

# Figma Changelog 4 Feature Pillars — REA Evidence & State-First Behavior Specification (`FP-01` .. `FP-04`)

> **Methodology:** `Research (REA + Figma Changelog) -> Spec (State-First) -> Build -> Verify`  
> **Target Parity Baseline:** Figma Desktop `126.9.13` (`compiled_wasm.wasm` + `app.asar`) & Figma Changelog (Auto-Layout 5.0 Suggest Auto-Layout, Non-Destructive Boolean Groups, Variables Phase 2 Geometry/Typography Bindings, Advanced Prototyping Expressions & Interactive Components)  
> **Target Runtime:** `design:os studio` (`crates/designos-doc/src/changelog_features.rs`)

---

## Layer 1: REA + Figma Changelog Evidence Ledger

### 1.1 Reverse-Engineered Symbols & Changelog Provenance (`FACT`)

| Pillar ID | Feature Pillar | Figma `compiled_wasm.wasm` C++ Units & Symbols | Figma Changelog Capability |
|---|---|---|---|
| **`FP-01`** | **Suggest Auto-Layout (`Cmd+Shift+A` / `Ctrl+Shift+A`)** | `fullscreen/lib/scenegraph/FGStackFacet.cpp`, `WrapInStackBehavior`, `SuggestedAutoLayout`, `_StackFacetTsApiGenerated_setStackSpacing`, `_StackFacetTsApiGenerated_setStackPadding*` | Spatial 1D/2D clustering of selected unstacked sibling layers into Horizontal row, Vertical column, or nested 2D Column-of-Rows Auto-Layout 5.0 frames with automatic gap and padding inference. |
| **`FP-02`** | **Live Non-Destructive Boolean Groups (`Union / Subtract / Intersect / Exclude`)** | `fullscreen/lib/scenegraph/FGBooleanOperationNode.cpp`, `BooleanOperationType`, `computeBooleanGeometry`, `fillGeometry` | Non-destructive boolean parent container (`NodeKind::BooleanOp`) that preserves editable child shapes in the scene tree and dynamically recomputes its tight composite bounding box and local-space `fill_geometry` (`Vec<Path>`) whenever any child moves or resizes. |
| **`FP-03`** | **Variable Bindings on Numeric Geometry/Text & Subtree Mode Switching** | `fullscreen/lib/variables/FGVariableConsumption.cpp`, `boundVariables`, `explicitVariableModes`, `resolveVariableForNode` | Binding `VariableId` to numeric geometry (`Width`, `Height`, `PaddingTop/Right/Bottom/Left`, `ItemSpacing`, `CornerRadius`, `Opacity`), booleans (`Visible`), and `Text` characters, plus `apply_variable_mode_to_subtree` that re-evaluates all bound fills, numbers, and strings across a subtree and triggers `relayout_subtree`. |
| **`FP-04`** | **Interactive Prototype Runtime (Overflow Scroll, Variant Swap & Conditional Variables)** | `fullscreen/lib/prototyping/FGPrototypeRuntime.cpp`, `_PrototypingEditorState_Internal_*`, `ScrollBehavior`, `ChangeToAction`, `ConditionalAction`, `SetVariableAction` | Stateful `PrototypeRuntime` supporting `OverflowScrollMode::{None, Horizontal, Vertical, Both}` with viewport scroll-offset clamping, interactive component `ChangeTo` variant swapping, and `SetVariable` / `Conditional` (`If / Else`) expression evaluation. |

---

## Layer 2: State-First Specifications (`FP-01` .. `FP-04`)

### Pillar 1 (`FP-01`): Suggest Auto-Layout (`Cmd+Shift+A` / `Ctrl+Shift+A`)

#### 1. Intent & Spatial Clustering Algorithm
- **Intent:** Convert a loose selection of unstacked sibling nodes (or an unstacked container Frame's children) into a structured 1D or 2D Auto-Layout 5.0 hierarchy in a single action without manual row-by-row framing.
- **Spatial Clustering Rules:**
  1. **Row Band Partitioning:** Sort candidate sibling nodes by vertical top edge `y`, breaking ties by `x`. Group nodes into horizontal row bands: a node joins the current row band if its vertical interval `[y, y + h]` overlaps the band's vertical interval by at least `30%` of `min(node.h, band.h)` (or its vertical center lies within the band's vertical span).
  2. **Classification:**
     - **`HorizontalRow` (1D):** `row_bands.len() == 1` and `nodes.len() >= 1`. Wraps nodes in a single `LayoutDirection::Horizontal` frame sorted by ascending `x`.
     - **`VerticalColumn` (1D):** `row_bands.len() == nodes.len()` (every band has `1` node). Wraps nodes in a single `LayoutDirection::Vertical` frame sorted by ascending `y`.
     - **`NestedColumnOfRows` (2D):** `row_bands.len() >= 2` and at least one band contains `>= 2` nodes. Each multi-node row band is wrapped into an inner `LayoutDirection::Horizontal` frame (`Sizing::Hug` on both axes) with inferred horizontal `item_spacing`; all row frames are wrapped into an outer `LayoutDirection::Vertical` frame with inferred vertical `item_spacing` between row bands.
  3. **Gap & Padding Inference:**
     - **Horizontal Gap:** Median of `max(0.0, next.x - (prev.x + prev.w)).round()` across consecutive items in a row.
     - **Vertical Gap:** Median of `max(0.0, next_row.y - (prev_row.y + prev_row.h)).round()` across consecutive rows in a column.
     - **Padding:** When converting an existing parent Frame, infer `[top, right, bottom, left]` from the outer margins between the children union box and the parent Frame bounds; when wrapping loose siblings on a Page/Frame, tight-wrap with `[0.0; 4]` padding at the selection's bounding-box origin `(union.x, union.y)`.

#### 2. State Transition Table (`FP-01`)

| From State | Event | Guard | To State | Side Effect | Feedback |
|---|---|---|---|---|---|
| `UnstackedSelection` | `SuggestAutoLayout` (`Cmd+Shift+A`) | `selected.len() >= 2` & shared parent & single row band | `AutoLayoutRowCommitted` | Insert `Horizontal` Auto-Layout Frame at union origin; reparent sorted children; set inferred `item_spacing`; run `relayout_subtree` | Selection switches to new Auto-Layout row frame |
| `UnstackedSelection` | `SuggestAutoLayout` (`Cmd+Shift+A`) | `selected.len() >= 2` & shared parent & single item per row band | `AutoLayoutColumnCommitted` | Insert `Vertical` Auto-Layout Frame at union origin; reparent sorted children; set inferred `item_spacing`; run `relayout_subtree` | Selection switches to new Auto-Layout column frame |
| `UnstackedSelection` | `SuggestAutoLayout` (`Cmd+Shift+A`) | `selected.len() >= 3` & `row_bands >= 2` with multi-item rows | `NestedColumnOfRowsCommitted` | Create inner `Horizontal` frames per row band + outer `Vertical` frame; infer row & column gaps; run `relayout_subtree` | 2D card/form grid becomes a nested Column-of-Rows Auto-Layout tree |

---

### Pillar 2 (`FP-02`): Live Non-Destructive Boolean Groups (`Union / Subtract / Intersect / Exclude`)

#### 1. Intent & Non-Destructive Geometry Pipeline
- **Intent:** Combine 2+ sibling vector/primitive shapes into a `NodeKind::BooleanOp(BooleanOp)` container while keeping every child node alive and editable in the document hierarchy, dynamically recomputing the boolean group's bounding box and `fill_geometry` (`Vec<Path>`) whenever any child moves, resizes, or changes visibility.
- **2D Polygon Clipper & Bounding-Box Recomputation:**
  1. Convert each visible child (`Rectangle`, `Ellipse`, `Polygon`, `Star`, `Vector`, or nested `BooleanOp`) into closed 2D polygon contours in parent space.
  2. Fold visible children left-to-right using the group's `BooleanOp`:
     - `BooleanOp::Union`: $P_{\text{acc}} \cup P_i$
     - `BooleanOp::Subtract`: $P_0 \setminus \bigcup_{i \ge 1} P_i$
     - `BooleanOp::Intersect`: $\bigcap_{i} P_i$
     - `BooleanOp::Exclude`: $P_{\text{acc}} \oplus P_i = (P_{\text{acc}} \setminus P_i) \cup (P_i \setminus P_{\text{acc}})$
  3. Subdivide polygon edges at all pairwise intersection parameters $t \in (0, 1)$, classify each sub-segment midpoint against the opposite operand via even-odd / non-zero winding, stitch kept directed segments into closed `Path` loops, and compute the tight bounding box `Rect` of the resulting geometry.
  4. Update the boolean group's `transform` to `(bounds.x, bounds.y)` and `size` to `(bounds.w, bounds.h)`, shift child local transforms by `-delta_origin` so children stay stationary in world space, and write local-space `Vec<Path>` into `node.fill_geometry`.

#### 2. State Transition Table (`FP-02`)

| From State | Event | Guard | To State | Side Effect | Feedback |
|---|---|---|---|---|---|
| `SiblingShapesSelected` | `CreateBooleanGroup(op)` | `child_ids.len() >= 2` & shared container parent | `LiveBooleanActive` | Insert `NodeKind::BooleanOp(op)`; reparent children preserving world coordinates; compute `fill_geometry` & tight bounds | Boolean group node created with non-destructive children intact |
| `LiveBooleanActive` | `ChildMovedOrResized(child_id, new_rect)` | `child.parent` (or ancestor) is `NodeKind::BooleanOp` | `LiveBooleanRecomputed` | Update child transform/size; walk up ancestor `BooleanOp` chain and recompute composite `fill_geometry` & tight `size`/`transform` | Composite boolean outline and bounding box update in `< 1ms` |

---

### Pillar 3 (`FP-03`): Variable Bindings on Numeric Geometry/Text & Subtree Mode Switching

#### 1. Intent & Subtree Evaluation Contract
- **Intent:** Allow any node property (`Width`, `Height`, `PaddingTop/Right/Bottom/Left`, `ItemSpacing`, `CounterAxisSpacing`, `CornerRadius`, `Opacity`, `Visible`, `TextContent`, `Fill(i)`, `Stroke(i)`) to bind to a `VariableId`, and re-evaluate an entire component/frame subtree when switching `VariableModeId` (`Light` ↔ `Dark`, `Compact` ↔ `Spacious`).
- **Evaluation Pipeline (`apply_variable_mode_to_subtree`):**
  1. Set `root.variables.modes.insert(collection_id, mode_id)` on `root_id`.
  2. Walk `root_id` subtree in pre-order; for every node with `variables.bindings`, resolve each bound variable via `doc.resolve_variable(node_id, var_id)` (inheriting nearest ancestor mode override and following `VariableValue::Alias` chains up to `MAX_ALIAS_DEPTH = 16`).
  3. Apply resolved values to concrete node properties (`size.x`, `size.y`, `auto_layout.padding`, `auto_layout.item_spacing`, `corner_radii`, `opacity`, ` fills`, `strokes`, `TextProps::content`).
  4. Run `relayout_subtree(doc, root_id)` and recompute any affected `BooleanOp` nodes so layout positions and container Hug/Fill dimensions immediately reflect updated token numbers.

#### 2. State Transition Table (`FP-03`)

| From State | Event | Guard | To State | Side Effect | Feedback |
|---|---|---|---|---|---|
| `UnboundProperty` | ` bind_node_variable(node, prop, var_id)` | Variable kind matches `prop` (`Number`, `String`, `Color`, `Boolean`) | `PropertyBound` | Record `node.variables.bindings[prop.key()] = var_id`; resolve in current effective mode & apply to node; relayout parent if geometry changed | Property value reflects bound variable token |
| `SubtreeInMode(M1)` | `apply_variable_mode_to_subtree(root, coll, M2)` | `coll` contains mode `M2` | `SubtreeInMode(M2)` | Set explicit mode `M2` on `root`; re-evaluate all bound fills, numbers, booleans, and text strings in `root` subtree; call `relayout_subtree(root)` | Entire subtree switches theme/density and reflows Auto-Layout |

---

### Pillar 4 (`FP-04`): Interactive Prototype Runtime (Overflow Scroll, Variant State Swap & Conditional Variables)

#### 1. Intent & Runtime Architecture
- **Intent:** Execute interactive prototypes directly against a `Document` with stateful scroll containers, interactive component variant swapping (`Action::ChangeTo`), and Turing-safe conditional variable expressions (`Action::SetVariable`, `Action::SetVariableMode`, `Action::Conditional`).
- **Subsystems:**
  1. **Overflow Scroll Clamping (`OverflowScrollMode`):**
     - Supports `None`, `Horizontal`, `Vertical`, `Both`.
     - Computes `max_scroll = (content_extent - viewport_size).max(0.0)` over non-fixed visible children.
     - Clamps scroll delta along permitted axes; children listed in `prototype.fixed_children` remain pinned to viewport coordinates while scrollable children shift by `-scroll_offset`.
  2. **Variant State Swap (`trigger_variant_interaction` / `Action::ChangeTo`):**
     - Matches `Trigger` (`OnClick`, `WhileHovering`, `WhilePressing`, `MouseEnter`, `MouseLeave`, `KeyPress`, etc.) on the target node or its enclosing component instance.
     - Executes `Action::ChangeTo { dest }` by swapping the instance to sibling variant `dest` in the `ComponentSet`, preserving instance overrides, updating variant property values, and reflowing parent Auto-Layout.
  3. **Conditional Variable Evaluation (`Action::SetVariable` & `Action::Conditional`):**
     - Evaluates `Expression` AST against runtime variable state (`PrototypeRuntime::variable_overrides` layered over `Document::variables`).
     - Executes `If / Else` branches in `Action::Conditional`, mutates runtime variables via `Action::SetVariable`, and syncs bound node properties (`TextContent`, `Visible`, `Width`, `Fills`, etc.) across the active frame subtree.

#### 2. State Transition Table (`FP-04`)

| From State | Event | Guard | To State | Side Effect | Feedback |
|---|---|---|---|---|---|
| `PrototypeIdle` | `scroll_by(container, delta)` | `container.prototype.overflow != OverflowScrollMode::None` | `ScrolledClamped` | Clamp `offset + delta` to `[0, max_scroll_x] × [0, max_scroll_y]` per allowed axis | Non-fixed children shift by `-scroll_offset`; fixed children stay pinned |
| `VariantInstance(V1)` | `trigger_variant_interaction(node, trigger)` | Matching `Reaction` has `Action::ChangeTo { dest: V2 }` | `VariantInstance(V2)` | Swap instance main component to `V2`, preserve overrides, sync variant props, run `relayout_subtree` | Instance swaps to hover/active/toggled variant in place |
| `PrototypeActive` | `trigger_variant_interaction(node, trigger)` | Matching `Reaction` has `Conditional` / `SetVariable` | `VariablesUpdated` | Evaluate `Expression` condition; run matching branch `SetVariable` / `SetVariableMode`; re-evaluate bound properties on active frame | Bound text, visibility, and dimensions update live |

---

## Layer 3: Verification Matrix

| Test ID | Pillar | Verification Scenario | Target Test |
|---|---|---|---|
| `T-FP01-1D-ROW` | `FP-01` | 3 horizontal sibling rectangles -> `suggest_auto_layout` infers `Horizontal` direction and exact `16px` gap | `crates/designos-doc/tests/changelog_pillars.rs::pillar1_suggest_auto_layout_1d_row_and_column` |
| `T-FP01-2D-GRID` | `FP-01` | 4 cards arranged in a 2×2 grid -> `suggest_auto_layout` clusters into 2 inner `Horizontal` row frames inside 1 outer `Vertical` column frame | `crates/designos-doc/tests/changelog_pillars.rs::pillar1_suggest_auto_layout_2d_column_of_rows` |
| `T-FP02-LIVE-BOOL` | `FP-02` | Live `Union`, `Subtract`, `Intersect`, `Exclude` boolean groups preserve children and dynamically update composite `size` and `fill_geometry` when a child moves/resizes | `crates/designos-doc/tests/changelog_pillars.rs::pillar2_live_non_destructive_boolean_groups` |
| `T-FP03-VAR-MODE` | `FP-03` | Bind `Width`, `Height`, `Padding*`, `ItemSpacing`, `CornerRadius`, `Opacity`, `TextContent`, `Fill(0)` to variables and switch subtree mode (`Compact/Light` -> `Spacious/Dark`) | `crates/designos-doc/tests/changelog_pillars.rs::pillar3_variable_bindings_and_subtree_mode_switching` |
| `T-FP04-PROTO-RT` | `FP-04` | `PrototypeRuntime` clamps `OverflowScrollMode` offsets, swaps component variants on `OnClick`/`WhileHovering`, and evaluates `Conditional` + `SetVariable` expressions | `crates/designos-doc/tests/changelog_pillars.rs::pillar4_interactive_prototype_runtime` |
