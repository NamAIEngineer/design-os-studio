# Phase 5: 100% Figma Design Parity & Embedded PTY Agent Terminal

## Press Release
**DESIGN:OS Studio** achieves 100% 1:1 Figma Design feature parity (2D Auto-Layout CSS Grid, Component Slots, Scoped Variables, Variable Font Axes & OpenType Features, and Lossless `.fig` v106 Binary Roundtrip for Overrides & Component Props) while embedding a **Native PTY Agent Terminal Runtime (`claude`, `codex`, `grok`, `$SHELL`)** and an **In-Canvas `Cmd+K` Agent & Design Gate Bar** directly inside the studio.

## Product Goals
1. **100% 1:1 Figma Design Feature Parity**:
   - **Auto-Layout 2D CSS Grid (`LayoutDirection::Grid`)**: Support 2D Grid track sizing (`Flex(fr)`, `Fixed(px)`, `Hug`), explicit/auto `(row, col)` placement, `row_span` / `col_span`, CSS Grid / Tailwind codegen, and `.fig` `stackMode: "GRID"` roundtrip.
   - **Component Slots (`PropKind::Slot`) & Scoped Variables (`Variable::matches_scope`)**: Preserve user-inserted subtrees inside designated instance Slot containers during `sync_instance`, and filter the inspector Variable Picker by `Variable::scopes`.
   - **Variable Font Axes & OpenType Features**: Continuous variation axes (`wght`, `wdth`, `opsz`, `slnt`) and OpenType feature toggles (`liga`, `tnum`, `smcp`, `ss01`, `ss02`) in `TextStyle`, `designos-render`, `designos-export`, and `.fig` binary R/W.
   - **Lossless `.fig` v106 Binary Export**: Encode `symbolOverrides`, `componentPropDefs`, `componentPropRefs`, `componentPropAssignments`, `fontVariations`, and `toggledOnOTFeatures` into `canvas.fig`.
2. **Native Embedded PTY Agent Terminal (`crates/designos-app/src/terminal.rs`)**:
   - Real POSIX/cross-platform PTY (`portable-pty` + `vt100`) running `$SHELL -l`, `claude`, `codex`, `grok`, and `designos` with full ANSI/24-bit TrueColor, alternate screen buffer, and automatic `$DESIGNOS_BIN` / `$DESIGNOS_FILE` injection.
   - **Bidirectional Canvas <-> Terminal Bridge**: 1-click **Send Selection & Pin-Comments to Terminal** (`stdin` injection) and 1-click **Apply Agent Spec Output to Canvas** (`stdout` JSON `NodeSpec` detector + `<500ms` disk hot-reload).
3. **In-Canvas `Cmd+K` Agent Bar & Live Design Gate (`crates/designos-app/src/cmdk.rs`)**:
   - Floating `Cmd+K` bar on the Canvas for natural-language EaseUI screen/component synthesis and live VSF/EaseUI Design Gate auditing (4px spacing grid, `>=32px` tap targets, token binding, readability) with 1-click Auto-Fix.

## MoSCoW Prioritization
- **MUST**: Native PTY Terminal (`claude`, `codex`, `grok`, `$SHELL`) + Canvas/Spec bridge.
- **MUST**: 2D Auto-Layout CSS Grid (`LayoutDirection::Grid`) in model, solver, inspector, and CSS/Tailwind/React export.
- **MUST**: Lossless `.fig` v106 export for `symbolOverrides`, `componentPropDefs`, OpenType/Variable axes, and `Grid`.
- **SHOULD**: Component Slots (`PropKind::Slot`) + `Variable::matches_scope` filtering in the inspector.
- **SHOULD**: In-Canvas `Cmd+K` Agent Bar + Live Design Gate Audit & 1-click Auto-Fix.
