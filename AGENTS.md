# DESIGN:OS Studio (`design-os-studio`) — Cross-Runtime Agent Instructions (`AGENTS.md`)

This repository is **DESIGN:OS Studio**, the native Rust + `wgpu` (Metal/Vulkan/GL) GPU-accelerated design canvas, `.fig` / `.designos` / `.designos` editor, and headless MCP server (`designos-mcp` / `designos-mcp`) for the **DESIGN:OS / EaseUI** ecosystem.

All agents (Claude Code, Codex, Antigravity, Grok, Orca workers) operating in this repository are bound by the same architectural contracts as [`CLAUDE.md`](file:///Users/jangtrinh/Products/design-os-studio/CLAUDE.md), [`.project-agent.md`](file:///Users/jangtrinh/Products/design-os-studio/.project-agent.md), and [`CONTEXT.md`](file:///Users/jangtrinh/Products/design-os-studio/CONTEXT.md).

---

## 1. Mandatory Skills & Context Loading

1. **Required Skills:**
   - `es:rust` — Idiomatic Rust 2021, zero-clone data flow, lock-order safety, and memory efficiency on 700k+ node graphs.
   - `es:rust-gpu-canvas` — `wgpu` pipeline architecture, `egui-wgpu` render pass integration, `lyon` tessellation budgeting, `SceneIndex` spatial culling, and CPU/GPU reference parity.
   - `es:astra-work` — Evidence-backed execution loop and post-mutation verification.
   - `es:fable-thinking` — Epistemic ledger (`FACT` / `INFERENCE` / `ASSUMPTION`), falsification-first root-cause analysis, and prediction before action.
2. **Project Knowledge Base (`.brv/`):**
   - Read `.brv/context-tree/session-handoffs/latest.md` before starting feature or bugfix work.
   - Read `.brv/context-tree/architecture/crate-map.md` and `.brv/context-tree/architecture/figma-vs-designos-engine.md` before modifying document serialization, `.fig` import, `SceneIndex`, or GPU compilation.

---

## 2. Workspace Crate Map (11 Crates)

| Crate | Directory | Summary |
|---|---|---|
| **`designos-doc`** | `crates/designos-doc` | Scene graph (`Document`, `Node`, `NodeId`, `NodeKind`), compact single-line JSON + `"hex:..."` image serialization, styles, variables, components, prototyping, undo/redo. |
| **`designos-fig`** | `crates/designos-fig` | Figma `.fig` (`fig-kiwi` binary container) and REST JSON importer/exporter, VectorNetwork conversion, component/instance expansion. |
| **`designos-render`** | `crates/designos-render` | Deterministic CPU reference renderer (`tiny-skia`), `SceneIndex` spatial/paint-order index, `TileCache`, `i_overlay` boolean ops, `cosmic-text` layout. |
| **`designos-gpu`** | `crates/designos-gpu` | `wgpu` GPU renderer (`SceneCompiler`, `GeomCache`, `PaintCache`, `Renderer`, WGSL shaders) with `lyon` tessellation and 8ms frame budget. |
| **`designos-app`** | `crates/designos-app` | `egui`/`eframe` desktop app (`designos-studio` / `designos`), EaseUI BrainWave 2.0 dark/light theme, GPU/tile canvas, virtualized Layers panel, Home library, CLI. |
| **`designos-library`** | `crates/designos-library` | On-disk project store (`Library`), 8 KB `file_kind` header probe, single-pass `import_file_with_doc`, thumbnails, trash, version history. |
| **`designos-fonts`** | `crates/designos-fonts` | Bundled Inter + Fontsource on-demand font fetcher, disk cache, and zero-clone `ensure_font_requests_with`. |
| **`designos-svg`** | `crates/designos-svg` | SVG import (`usvg` -> `designos_doc::Node` tree). |
| **`designos-export`** | `crates/designos-export` | PNG/JPG/SVG/PDF export and Dev Mode code generation (`CSS`, `Tailwind`, `React`, `SwiftUI`, `Compose`). |
| **`designos-mcp`** | `crates/designos-mcp` | Stdio JSON-RPC MCP server (`designos-mcp` / `designos-mcp`) for AI agent canvas/document automation. |
| **`designos-cli` / `designos-diff`** | `crates/designos-diff` + `crates/designos-app/src/cli.rs` | Pixel-diff verification CLI (`designos-diff`) and desktop CLI automation interface (`--open-id`, `--screenshot`, `--gpu`, `--fixture`). |

---

## 3. Build, Package & Benchmark Commands

```bash
export PATH="$HOME/.cargo/bin:$PATH"

# 1. Build release desktop app & MCP server
cargo build --release -p designos-app -p designos-mcp
cargo build --release -p designos-app

# 2. Update macOS app bundle at target/release/bundle/DESIGN-OS Studio.app
cp target/release/designos-studio "target/release/bundle/DESIGN-OS Studio.app/Contents/MacOS/designos-studio"
cp target/release/designos "target/release/bundle/DESIGN-OS Studio.app/Contents/MacOS/designos"

# 3. Run the 723,136-node "VSF - PCP" benchmark fixture directly
"target/release/bundle/DESIGN-OS Studio.app/Contents/MacOS/designos-studio" --open-id 18de678a186512c80000

# 4. Run test gates
cargo test --workspace --lib
cargo test -p designos-gpu
cargo test -p designos-mcp
```

---

## 4. Non-Negotiable Performance & Safety Invariants

1. **No `doc.to_json()` in `doc_hash` (`crates/designos-app/src/home/session.rs`):**
   - Always hash documents via the streaming, zero-allocation `HashSerializer`. Never serialize `Document` to a `String` for change detection or autosave probing, and never bump `TileView::rev` (`app.changed()`) on viewport-only pan/zoom/fit gestures.
2. **No `ctx.load_texture` inside `ctx.data_mut` (`crates/designos-app/src/icons.rs`):**
   - `egui::Context::data_mut` holds a non-reentrant `parking_lot::RwLock` write guard. Calling `ctx.load_texture` inside `ctx.data_mut` deadlocks the main thread on Frame 1. Decode and call `ctx.load_texture` outside `ctx.data_mut`.
3. **Preserve `interactive()` Guard in `designos-gpu` (`crates/designos-gpu/src/compile.rs`):**
   - `self.interactive()` (`self.cull && self.geom.budget.is_some()`) separates live interactive rendering from headless test rendering. Keep all LOD approximations (`< 0.75px` culling, `< 3.5px` text greeking, scissor clip fallback, `< 64` `Program` cap) strictly inside `self.interactive()` so headless reference tests (`cargo test -p designos-gpu`) remain bit-exact against `designos-render`.
4. **Cap Offscreen GPU `Program`s & Preserve `FrameKey` Reuse:**
   - Do not allocate new offscreen `Program`s beyond `self.progs.len() < 64` during interactive frames, and preserve `FrameKey` caching (`Cb { frame: None }` when `last_key == Some(key) && stale_hits == 0`) plus `frame_build_nanos` 8ms cumulative tessellation budgeting so idle CPU stays at `0.0%` and `egui-wgpu`'s Metal staging buffer never overflows.
