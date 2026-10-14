# DESIGN:OS Studio (`design-os-studio`) — Agent Instructions (`CLAUDE.md`)

Native GPU-accelerated Rust design canvas, `.fig` / `.designos` / `.designos` editor, and headless MCP server (`designos-mcp` / `designos-mcp`) for the **DESIGN:OS / EaseUI** ecosystem.

> **Before starting any work**, read:
> 1. `.project-agent.md` — product identity, EaseUI BrainWave 2.0 theme tokens, and verification commands.
> 2. `CONTEXT.md` — canonical domain glossary (`Document`, `TileView::rev`, `SceneIndex`, Compact Native Format).
> 3. `.brv/context-tree/session-handoffs/latest.md` — latest session state, verified metrics, and active invariants.
> 4. `.brv/context-tree/architecture/crate-map.md` & `.brv/context-tree/architecture/figma-vs-designos-engine.md` — crate responsibilities, hot paths, and Figma engine mapping.

---

## 1. Required Skills (Mandatory)

Every agent working on `design-os-studio` MUST load and follow:
- **`es:rust`** — Rust 2021 ownership, borrowing, zero-allocation hot paths, `parking_lot` / `egui` lock discipline, and deterministic error handling.
- **`es:rust-gpu-canvas`** — `wgpu` Metal/Vulkan/GL rendering architecture, `egui-wgpu` callback lifecycle, `lyon` tessellation budgeting, spatial culling, and CPU/GPU parity.
- **`es:astra-work`** — Outer execution loop, outcome contract, and post-mutation verification with real evidence.
- **`es:fable-thinking`** — Epistemic ledger (`FACT` / `INFERENCE` / `ASSUMPTION`), falsification-first debugging, and prediction discipline.

---

## 2. Workspace Crate Map (11 Crates)

| Crate | Path / Binary | Role & Primary Responsibility |
|---|---|---|
| **`designos-doc`** | `crates/designos-doc` | Core document model (`Document`, `Node`, `NodeId`, `NodeKind`), compact JSON serializer (`serde_images` `"hex:..."` + `skip_serializing_if`), styles, variables, components, prototyping, and undo/redo history. |
| **`designos-fig`** | `crates/designos-fig` | Figma `.fig` (`fig-kiwi` binary container + DEFLATE/Zstd) and Figma REST JSON importer/exporter, VectorNetwork converter, and component instance expander. |
| **`designos-render`** | `crates/designos-render` | Deterministic CPU reference renderer (`tiny-skia 0.11`), `SceneIndex` world-bounds spatial & paint-order index, `TileCache`, boolean path ops (`i_overlay`), and `cosmic-text` shaping. |
| **`designos-gpu`** | `crates/designos-gpu` | `wgpu` GPU canvas renderer: `SceneCompiler` (`compile.rs`), zoom-bucket `GeomCache` (`geom.rs`, `lyon_tessellation`), `PaintCache` (`paintconv.rs`), and multi-pass `Renderer` (`exec.rs`, custom WGSL shaders). |
| **`designos-app`** | `crates/designos-app` (`designos-studio`, `designos`) | Desktop editor UI (`egui 0.31` + `eframe 0.31`), EaseUI BrainWave 2.0 theme (`theme.rs`), GPU/tile canvas viewport (`canvas/`), virtualized Layers panel (`panels/layers.rs`), Home library, and CLI (`cli.rs`). |
| **`designos-library`** | `crates/designos-library` | On-disk projects library (`store.rs`): `.designos`/`.designos`/`.fig` storage, 8 KB `file_kind` header probe, single-pass `import_file_with_doc`, thumbnails, version history, and published libraries. |
| **`designos-fonts`** | `crates/designos-fonts` | Bundled Inter font + on-demand Fontsource/Google Fonts HTTP fetcher, disk cache, script fallback, and zero-clone `ensure_font_requests_with`. |
| **`designos-svg`** | `crates/designos-svg` | SVG importer mapping `usvg` trees onto `designos_doc::Node` hierarchies with paint, text, and filter conversion. |
| **`designos-export`** | `crates/designos-export` | PNG/JPEG/SVG/PDF document exporter and Dev Mode codegen (`CSS`, `Tailwind`, `React`, `SwiftUI`, `Jetpack Compose`). |
| **`designos-mcp`** | `crates/designos-mcp` (`designos-mcp`, `designos-mcp`) | Headless stdio JSON-RPC MCP server (`rmcp`) exposing document inspection, creation, layout, export, and library operations to AI agents. |
| **`designos-cli` / `designos-diff`** | `crates/designos-diff` (`designos-diff` CLI) + `crates/designos-app/src/cli.rs` | CLI harness & pixel-diff test suite comparing CPU/GPU renders against reference screenshots (`crates/designos-diff`) plus desktop CLI flags (`--screenshot`, `--open-id`, `--gpu`, `--fixture`). |

---

## 3. Build, Test & Run Commands

Always prepend `$HOME/.cargo/bin` to `PATH` before invoking `cargo` or `rustc`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"

# Build release binaries (desktop editor + MCP server)
cargo build --release -p designos-app -p designos-mcp
cargo build --release -p designos-app

# Sync release binary into the macOS app bundle
cp target/release/designos-studio "target/release/bundle/DESIGN-OS Studio.app/Contents/MacOS/designos-studio"
cp target/release/designos "target/release/bundle/DESIGN-OS Studio.app/Contents/MacOS/designos"

# Launch the packaged macOS app bundle
open "target/release/bundle/DESIGN-OS Studio.app"

# Launch directly into the 723,136-node "VSF - PCP" enterprise benchmark fixture
"target/release/bundle/DESIGN-OS Studio.app/Contents/MacOS/designos-studio" --open-id 18de678a186512c80000

# Run verification suites
cargo test --workspace --lib
cargo test -p designos-gpu
cargo test -p designos-mcp
```

---

## 4. Non-Negotiable Performance & Safety Invariants

1. **NEVER call `doc.to_json()` inside `doc_hash` or on the UI thread (`crates/designos-app/src/home/session.rs`):**
   - `doc_hash(doc: &Document) -> u64` MUST use the zero-allocation `HashSerializer` streaming directly into `DefaultHasher` (and sampling image byte headers/tails). Allocating a JSON string on a 723k-node document stalls the UI thread and spikes memory by gigabytes.
   - Viewport-only operations (pan, zoom, fit-to-screen, marquee selection) MUST NEVER call `app.changed()` or increment `TileView::rev`.

2. **NEVER call `ctx.load_texture` inside `ctx.data_mut` (`crates/designos-app/src/icons.rs` & all `egui` code):**
   - `egui::Context::data_mut` acquires a write lock on `egui`'s internal `parking_lot::RwLock<ContextImpl>`. Calling `ctx.load_texture(...)` while that closure is active attempts to re-lock `ContextImpl`, causing an immediate non-reentrant `RwLock` self-deadlock on Frame 1. Always release `ctx.data_mut` before calling `ctx.load_texture`, then insert the handle in a separate `ctx.data_mut` call.

3. **PRESERVE the `interactive()` guard in `crates/designos-gpu/src/compile.rs`:**
   - `fn interactive(&self) -> bool { self.cull && self.geom.budget.is_some() }` distinguishes live interactive viewport rendering (`budget = Some(8ms)`) from headless reference rendering (`budget = None`).
   - All interactive fast-paths (pre-resolve `< 0.75px` sub-pixel culling, `< 3.5px` text greeking, hardware scissor fallback on rounded clips, and offscreen `Program` caps) MUST stay gated behind `self.interactive()` so `cargo test -p designos-gpu` headless CPU-vs-GPU pixel-diff tests remain bit-exact.

4. **CAP offscreen GPU `Program` allocations (`self.progs.len() < 64`) and reuse `FrameKey` (`crates/designos-app/src/canvas/gpu.rs`):**
   - Unbounded offscreen render passes (`Program`s) on 723k-node pages exhaust `egui-wgpu`'s Metal staging ring buffer and crash the GPU driver. Keep offscreen layer/mask/clip/blend `Program` creation capped at `< 64` in `interactive()` mode (falling back to hardware scissor clips).
   - In `canvas::gpu::paint`, when `last_key == Some(key) && compiler.geom.stale_hits == 0`, pass `Cb { frame: None }` so `Canvas` re-blits the cached `Output` texture at `0.0%` idle CPU without recompiling or re-uploading vertex buffers.
   - `GeomCache::get` MUST track cumulative `frame_build_nanos` against the 8ms per-frame budget (`compiler.geom.budget`) and defer uncached `Text`/`Vector`/`BooleanOp` tessellation across subsequent frames via `stale_hits`.
