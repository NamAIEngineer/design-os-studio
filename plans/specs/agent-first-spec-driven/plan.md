# Phase 3 Specification: Agent-First CLI, Spec-Driven EaseUI Builder & Canvas Feedback Loop

## 1. Vision & Operating Model

**DESIGN:OS Studio** operates as an **Agent-First, Spec-Driven Design System & Review Surface**:
1. **Agents Build & Mutate via CLI & MCP (`designos` CLI / `designos-mcp`):**
   - Agents author, inspect, and update screens declaratively using `NodeSpec` JSON (`--apply-spec` / `designos_apply_spec`, `--export-spec` / `designos_export_spec`) with built-in **EaseUI BrainWave 2.0** tokens (`$easeui.*`), component presets (`easeui-screen`, `easeui-card`, `easeui-button-primary`, `easeui-input`, `easeui-badge`), and **Auto-Layout 5.0** (`Horizontal`/`Vertical`, `Wrap`, `Hug`/`Fill`/`Fixed`, `min_w`/`max_w`/`min_h`/`max_h`).
2. **Humans Review, Interact & Drop Feedback on the Canvas (`designos-studio`):**
   - The desktop GPU Canvas (`wgpu` Metal/Vulkan) acts as the visual review and feedback surface.
   - When an Agent mutates the open `.designos` or library file on disk, the open Canvas **hot-reloads the document automatically within 500ms** (`poll_external_reload` in `home/session.rs`), preserving the user's current page, selection, and viewport pan/zoom.
   - When the human drops a **Comment Pin (`C`)** or **Dev Annotation** on any element on the Canvas, autosave flushes it to disk and the Agent immediately queries `--comments-json` or `designos_list_comments` — which automatically resolves the exact target `Frame` and `Node` under each pin (`resolve_comment_targets`)!
   - After addressing the feedback, the Agent replies to the thread and marks it resolved via `--reply-comment <id> --message "..." --resolve` or `designos_reply_comment`.

---

## 2. Architecture & Epics

### Epic 1: Spec-Driven Engine, Auto-Layout 5.0 Reflow & EaseUI Presets (`crates/designos-doc/src/spec.rs`)
- **`EaseUiTokens` & Presets:**
  - Resolves `$easeui.bg.canvas` (`#121215`), `$easeui.bg.panel` (`#18181B`), `$easeui.bg.surface` (`#27272A`), `$easeui.bg.elevated` (`#303036`), `$easeui.border` (`#27272A`), `$easeui.border.subtle` (`#3F3F46`), `$easeui.text.primary` (`#F4F4F5`), `$easeui.text.secondary` (`#A1A1AA`), `$easeui.text.muted` (`#71717A`), `$easeui.accent.orange` / `$easeui.brand` (`#E36323`), `$easeui.accent.blue` (`#3582FF`), `$easeui.accent.violet` (`#8755E9`), `$easeui.status.success` (`#18B368`), `$easeui.status.warning` (`#F59E0B`), `$easeui.status.danger` (`#EF4444`), and `#RRGGBB[AA]`.
  - Presets: `easeui-screen`, `easeui-panel`, `easeui-card`, `easeui-button-primary`, `easeui-button-secondary`, `easeui-input`, `easeui-badge`, `easeui-row`, `easeui-column`.
- **Declarative `NodeSpec` & `LayoutSpec`:**
  - `Document::apply_spec(&mut self, parent: NodeId, spec: &NodeSpec) -> Result<NodeId, String>`
  - `Document::export_spec(&self, root: NodeId) -> Result<NodeSpec, String>`
  - `Document::relayout_subtree(&mut self, root: NodeId)` (bottom-up Auto-Layout 5.0 solver supporting `Wrap`, `Hug`, `Fill`, `SpaceBetween`, `min_w`/`max_w`/`min_h`/`max_h` clamping, and intrinsic text sizing).
- **Spatial Comment Pin Target Resolution:**
  - `Document::resolve_comment_targets(&self) -> Vec<ResolvedCommentThread>` maps every `CommentThread` to its world coordinates, anchored or hit-tested `target_node` (`id`, `name`, `kind`), and top-level `target_frame` (`id`, `name`).

### Epic 2: Agent-First MCP Server Upgrade (`crates/designos-mcp/src/engine.rs`, `crates/designos-mcp/src/server.rs`)
- New MCP Tools:
  - `designos_list_comments`: Lists comment threads with resolved `target_node` & `target_frame` context, filterable by `resolved`, `page_id`, `node_id`.
  - `designos_add_comment`: Pins a new feedback thread to a `node_id` or `(page_id, x, y)`.
  - `designos_reply_comment`: Appends a reply to `thread_id` and optionally marks it `resolved`.
  - `designos_resolve_comment`: Toggles `resolved` on `thread_id`.
  - `designos_apply_spec`: Builds or patches a subtree from a declarative `NodeSpec` JSON (with EaseUI presets/tokens & Auto-Layout 5.0 reflow) in one undoable command.
  - `designos_export_spec`: Exports a subtree as `NodeSpec` JSON with attached comments & dev annotations.
  - `designos_set_dev_status`: Sets `ReadyForDev`, `Completed`, or clears status + attaches Dev Annotations.
- `designos_save_file`: Supports `overwrite: true` for live-updating the active `.designos` file watched by the desktop Canvas.

### Epic 3: Agent-First Headless CLI & Live Canvas File-Watch Hot-Reload (`crates/designos-app`)
- **Headless JSON CLI (`crates/designos-app/src/cli.rs`, `crates/designos-app/src/main.rs`):**
  - `--inspect-json`
  - `--comments-json`
  - `--add-comment <TEXT> [--select <NAME_OR_ID>]`
  - `--reply-comment <THREAD_ID> --message <TEXT> [--resolve]`
  - `--apply-spec <SPEC.json> [--select <PARENT_NAME_OR_ID>]`
  - `--export-spec <OUT.json> [--select <NODE_NAME_OR_ID>]`
- **Live Canvas File-Watch Hot-Reload (`crates/designos-app/src/home/session.rs`):**
  - Polls active library file or `--file` path `mtime` every `500ms`; when modified externally by CLI/MCP, hot-reloads `self.doc` while preserving viewport `pan`/`zoom`, active `page`, and `selection`.
