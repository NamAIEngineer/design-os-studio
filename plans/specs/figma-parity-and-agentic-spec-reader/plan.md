# Implementation Plan: Phase 4 — 1:1 Figma Binary Parity (`.fig` R/W + UI3 Icons) & Native EaseUI Spec Reader

## Architecture & Reverse-Engineering Summary (`morluto/rea` + `VSF - PCP.fig`)

Using `morluto/rea` (`rea analyze-javascript-application`, `rea inspect-artifact`, `rea inspect-rust-binary`) on `/Applications/Figma.app` and binary inspection of `/Users/jangtrinh/Downloads/VSF - PCP.fig` (`fig-kiwi` container version `106`, 653 Kiwi definitions, 155,297 `NodeChange` records), we identified:

1. **`.fig` Binary Container & Kiwi Schema (`crates/designos-fig/src/fig_export.rs`)**:
   - A `.fig` archive is a ZIP containing `canvas.fig` (`b"fig-kiwi"` + `u32` version + DEFLATE-compressed Kiwi `Schema` chunk + Zstd/DEFLATE-compressed Kiwi `Message` chunk), `meta.json` (`client_meta`, `file_name`, `exported_at`), and `images/<hex_hash>`.
   - Root `Message` has `type = Enum("NODE_CHANGES")`, `sessionID = Uint(0)`, `ackID = Uint(0)`, `nodeChanges = Array(Vec<NodeChange>)`, `blobs = Array(Vec<Blob>)`.
   - `path_to_commands_blob(path: &Path) -> Vec<u8>` is the exact binary inverse of `commands_blob_to_path` (`MoveTo = 1`, `LineTo = 2`, `QuadTo = 3`, `CubicTo = 4`, `Close = 0`, little-endian `f32` coordinates).
2. **`.fig` Import Fidelity Gaps (`crates/designos-fig/src/fig.rs` & `rest_import.rs`)**:
   - `parameterConsumptionMap` (id=445 in `NodeChange`) holds Auto-Layout and typography variable bindings (`STACK_SPACING`, `STACK_PADDING_LEFT/TOP/RIGHT/BOTTOM`, `FONT_FAMILY`, `FONT_STYLE`, `FONT_SIZE`) on 631 nodes and 15 `symbolOverrides` in `VSF - PCP.fig`, often without `variableConsumptionMap`.
   - `FONT_STYLE` variable bindings use `variableData.value.fontStyleValue.asFloat` / `asString`.
   - `convert_stack` must support `stackPaddingLeft`/`stackHorizontalPadding`, `stackPaddingTop`/`stackVerticalPadding`, `stackPadding` uniform fallback, `stackWrap` (`"WRAP"`), `bordersTakeSpace` (`strokesIncludedInLayout`), and `stackReverseZIndex` (`itemReverseZIndex`).
   - `convert_override` in `fig.rs` and `apply_override` in `rest_import.rs` must propagate `boundVariables`, clear stale master `fills/*` and `strokes/*` variable bindings when `fills` or `strokes` are overridden, swap nested instances on `overriddenSymbolID` (`"componentId"`), and apply auto-layout overrides (`stackPrimarySizing`, `stackCounterSizing`, `stackHorizontalPadding`, `stackPaddingRight`, `stackVerticalPadding`, `stackPaddingBottom`, `stackSpacing`, `stackCounterSpacing`, `stackPrimaryAlignItems`, `stackCounterAlignItems`, `stackChildAlignSelf`, `stackChildPrimaryGrow`, `stackPositioning`, `stackMode`), `componentPropAssignments`, `individualStrokeWeights`, `rectangleCornerRadii`, and `frameMaskDisabled`.
   - `CANVAS.handoffStatusMap.entries` maps frame `guid` to `handoffStatus.status` (`"BUILD"` / `"READY_FOR_DEV"` -> `ReadyForDev`, `"COMPLETED"` -> `Completed`).

---

## Parallel Sub-Agent Work Breakdown (Strict Non-Overlapping File Ownership)

### Epic 1 (`backend-specialist` Subagent 1): 1:1 `.fig` Import Fidelity + Native `.fig` Binary Writer (`export_fig`)
**Owned Files**:
- `crates/designos-fig/src/fig.rs`
- `crates/designos-fig/src/rest_import.rs`
- `crates/designos-fig/src/fig_export.rs` (new)
- `crates/designos-fig/src/testkit.rs`
- `crates/designos-fig/src/lib.rs`
- `crates/designos-fig/tests/fig_roundtrip.rs` (new)

**Tasks**:
1. Fix `fig.rs`:
   - `bound_variables`: read entries from both `variableConsumptionMap` and `parameterConsumptionMap`; support `fontStyleValue.asFloat` / `asString` in `alias_target`; map `"FONT_STYLE"` to `"fontWeight"`.
   - `convert_stack`: read `stackPaddingLeft` (or `stackHorizontalPadding` or `stackPadding`), `stackPaddingTop` (or `stackVerticalPadding` or `stackPadding`), `stackPaddingRight` (or `stackHorizontalPadding` or `stackPadding`), `stackPaddingBottom` (or `stackVerticalPadding` or `stackPadding`), `stackWrap` (`"WRAP"`), `bordersTakeSpace` (`strokesIncludedInLayout`), and `stackReverseZIndex` (`itemReverseZIndex`).
   - `convert_override`: emit `boundVariables` (`bound_variables(o, lk)`), `overriddenSymbolID` -> `"componentId"`, auto-layout properties (`convert_stack` + `convert_child_layout`), `componentPropAssignments` (`component_props`), `individualStrokeWeights`, `rectangleCornerRadii`, `frameMaskDisabled` -> `"clipsContent"`, `opacity`, `blendMode`, `effects`.
   - `CANVAS.handoffStatusMap`: collect `handoffStatusMap.entries` from `CANVAS` records (`"BUILD"` | `"READY_FOR_DEV"` -> `"READY_FOR_DEV"`, `"COMPLETED"` -> `"COMPLETED"`) and inject `"devStatus"` onto the corresponding node JSON objects.
2. Fix `rest_import.rs`:
   - In `apply_override`:
     - When `set.get("fills").is_some()`, retain only non-`"fills/"` bindings in `n.variables.bindings`.
     - When `set.get("strokes").is_some()`, retain only non-`"strokes/"` bindings in `n.variables.bindings`.
     - Apply `boundVariables` from `set` via `crate::variables::parse_node_bindings(set, &mut n.variables)`!
     - When `set.get("componentId")` is present on a nested instance, call `swap_nested` to swap the nested instance's master component!
     - Apply auto-layout overrides (`layoutMode`, `itemSpacing`, `counterAxisSpacing`, `paddingTop/Right/Bottom/Left`, `primaryAxisAlignItems`, `counterAxisAlignItems`, `primaryAxisSizingMode`, `counterAxisSizingMode`, `layoutSizingHorizontal`, `layoutSizingVertical`, `layoutGrow`, `layoutPositioning`), `clipsContent`, `individualStrokeWeights`, `rectangleCornerRadii`, `componentPropertyReferences`, and `componentProperties`.
3. Implement `crates/designos-fig/src/fig_export.rs`:
   - `pub fn path_to_commands_blob(path: &Path) -> Vec<u8>`
   - `pub fn export_fig(doc: &Document) -> Result<Vec<u8>, FigError>`
   - Serializes `Document` into a `fig-kiwi` container (using an expanded `figma_like_schema()`) wrapped in a `.fig` ZIP (`container::write_zip`) with `canvas.fig`, `meta.json`, and `images/*`.
4. Add unit & integration tests in `crates/designos-fig/tests/fig_roundtrip.rs` verifying `.fig` export + re-import roundtrip AND override variable/layout/swap fidelity.

---

### Epic 2 (`frontend-specialist` Subagent 2): 1:1 Figma UI3 Icon Set
**Owned Files**:
- `crates/designos-app/src/icons.rs`
- `crates/designos-app/assets/icons/lucide/*.svg`
- `crates/designos-app/src/panels/layers.rs`

**Tasks**:
1. Add new `Icon` variants in `crates/designos-app/src/icons.rs` (`Icon::ALL` + `LUCIDE` table):
   - `Icon::Specs` (`specs.svg`)
   - `Icon::AutoLayoutH` (`auto-layout-h.svg`)
   - `Icon::AutoLayoutV` (`auto-layout-v.svg`)
   - `Icon::ComponentSet` (`component-set.svg`)
2. Upgrade SVG assets in `crates/designos-app/assets/icons/lucide/` to match Figma UI3 1:1:
   - `frame.svg` (Figma `#` frame hash grid with clean overhangs)
   - `auto-layout-h.svg` (Figma horizontal auto-layout two vertical bars inside frame)
   - `auto-layout-v.svg` (Figma vertical auto-layout two horizontal bars inside frame)
   - `component.svg` (Figma 4-diamond rosette)
   - `component-set.svg` (Figma 4-diamond variant set inside dashed diamond/frame)
   - `instance.svg` (Figma single hollow diamond)
   - `section.svg` (Figma section frame with top-left title tab)
   - `group.svg` (Figma dashed selection rectangle)
   - `mask.svg` (Figma half-filled circle/moon mask icon)
   - `radius.svg` (Figma single rounded corner arc with endpoint ticks)
   - `padding-h.svg` & `padding-v.svg` (Figma inner pill between parallel side/top-bottom bars)
   - `spacing.svg` (Figma gap icon between two bars)
   - `stroke-sides.svg` (Figma 4 independent border segments)
   - `specs.svg` (EaseUI/Figma product spec blueprint icon)
3. Update `crates/designos-app/src/panels/layers.rs`:
   - In `kind_icon(k: &NodeKind)`:
     - `NodeKind::Frame(f)` -> if `f.auto_layout` is `Some(al)` then `if al.direction == LayoutDirection::Horizontal { Icon::AutoLayoutH } else { Icon::AutoLayoutV }` else `Icon::Frame`!
     - `NodeKind::ComponentSet(_)` -> `Icon::ComponentSet`!
   - Ensure all unit tests in `icons.rs` (`every_icon_has_a_parsable_asset`, `all_lists_every_variant_once`, `no_vendored_svg_is_unused`) pass!

---

### Epic 3 (`fullstack-developer` Subagent 3): Native EaseUI-Styled Spec Reader & `.fig` Save/CLI/MCP Wiring
**Owned Files**:
- `crates/designos-app/src/specs.rs` (new)
- `crates/designos-app/src/panels/rail.rs`
- `crates/designos-app/src/panels/left.rs`
- `crates/designos-app/src/panels/menus.rs`
- `crates/designos-app/src/app.rs`
- `crates/designos-app/src/shell.rs`
- `crates/designos-app/src/io.rs`
- `crates/designos-app/src/cli.rs`
- `crates/designos-app/src/main.rs`
- `crates/designos-app/src/lib.rs`
- `crates/designos-mcp/src/engine.rs`
- `crates/designos-mcp/src/server.rs`

**Tasks**:
1. **Native EaseUI Spec Reader (`crates/designos-app/src/specs.rs`)**:
   - Discovers workspace spec packages under `plans/specs/*/` (`brainstorm.md`, `usecases.md`, `plan.md`, `*.spec.json`) and standalone `examples/specs/*.spec.json`.
   - Provides:
     - **Left Sidebar Spec Explorer (`specs::show_left`)**: when `app.ui.rail == Some(RailItem::Specs)`, displays discovered Spec Packages, `.spec.json` blueprints, and Linked Canvas Frames (with live comment count badges), plus quick actions (`Open Spec Reader`, `Apply .spec.json`, `Export Frame to .spec.json`, `Export .fig`).
     - **EaseUI-Styled Spec Reader Split/Overlay View (`specs::show_reader`)**:
       - Styled with EaseUI BrainWave 2.0 dark slate tokens (`#121215` / `#18181B` / `#27272A`, Orange `#FF6A3D`, Blue `#3B82F6`, Violet `#8B5CF6`, Emerald `#10B981`).
       - Tabs: **Overview (`brainstorm.md`)**, **Use Cases & Gherkin (`usecases.md`)**, **Plan (`plan.md`)**, **UI Spec (`.spec.json`)**, and **Canvas Screens & Agent Feedback**.
       - Structured rendering of Markdown headings, MoSCoW badges (`MUST`, `SHOULD`, `COULD`), User Story cards, Gherkin `Scenario` / `Given` / `When` / `Then` syntax-highlighted blocks, and `.spec.json` preset/token summary.
       - 1-click buttons to **Focus Frame on Canvas**, **Apply Spec to Canvas**, **Export Selected Frame to Spec**, and **Save `.fig` Copy**.
2. **Wire `.fig` Saving (`io.rs`, `menus.rs`, `cli.rs`, `main.rs`, `designos-mcp`)**:
   - `io::save_path`: if `path.extension()` is `"fig"`, call `designos_fig::export_fig(doc)` and write bytes; `pick_save` adds `.fig` filter; add `Action::ExportFig` (`File -> Save local .fig copy…`).
   - `cli.rs` & `main.rs`: add `--export-fig <FILE.fig>` and `--panel <specs|assets|variables|plugins>` CLI flags for headless agent workflows and visual QA screenshots.
   - `designos-mcp`: add `export_fig_file` MCP tool (`{ "output_path": "path/to/file.fig" }`).
