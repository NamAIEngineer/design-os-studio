# Brainstorm: Large `.fig` File Performance & Compact Native Storage (`large-fig-performance`)

## 1. Press Release (Amazon Method)
**DESIGN:OS Studio Now Opens, Saves, and Renders 700,000+ Node Figma Files at 60 FPS on Apple Silicon**
Designers and AI agents working with production enterprise Figma files (such as `VSF - PCP.fig`, 17 MB binary / 723,136 expanded nodes) previously experienced multi-second freezes on every pan/zoom gesture and 6.8 GB disk bloat when importing into the local library. Today, DESIGN:OS Studio introduces Compact Native Serialization, Zero-Allocation Revision Hashing, Spatial Hit-Testing via `SceneIndex`, and Virtualized Layer Tree Caching — shrinking library disk usage by ~50x and making pan, zoom, hover, and autosave instantaneous on Apple Silicon Metal GPUs.

## 2. User Stories
- **US-1 (Designer — Import & Disk Footprint):** As a product designer importing a 17 MB enterprise `.fig` file (`723,136` nodes), I want the saved `.designos` / `.designos` library file to stay compact (< 180 MB instead of 6.8 GB) and not re-parse from disk on import, so my SSD does not fill up and import completes quickly.
- **US-2 (Designer — 60 FPS Pan/Zoom & Autosave Probe):** As a designer navigating a 723k-node canvas, I want pan, zoom, and fit-to-screen to never bump the document revision (`rev`) or serialize the document to a multi-GB string on the UI thread, so viewport navigation stays locked at 60 FPS without 300ms post-gesture freezes.
- **US-3 (Designer — Instant Hover, Click & Layers Panel):** As a designer moving my cursor across the canvas with the Layers panel open, I want hit-testing to cull off-cursor subtrees via `SceneIndex` bounds and the Layers panel to cache flattened rows without cloning `Node` geometry, so frame time stays under 16ms.
- **US-4 (Designer — Non-Blocking File Open & Font Loading):** As a designer opening a large file from the Home screen, I want `Library::file_kind` to inspect only the file header and `FontLoader` to pass only `Vec<FontRequest>` to its worker thread rather than cloning all 723k nodes, so switching between Home and Editor is immediate.

## 3. MoSCoW Prioritization

| Priority | Item | Rationale |
|----------|------|-----------|
| **Must (P0)** | Compact JSON serialization (`skip_serializing_if` + `"hex:..."` images) with 100% backward-compatible deserialization | Eliminates the 400x (17 MB -> 6.8 GB) disk & RAM explosion |
| **Must (P0)** | Remove `app.changed()` from view-only pan/zoom/fit & replace `doc_hash` `to_json()` with zero-alloc `HashSerializer` | Eliminates the main-thread freeze 300ms after every pan/zoom and preserves `SceneIndex` across frames |
| **Must (P0)** | `Library::file_kind` 8 KB header probe + `import_file_with_doc` | Eliminates reading 6.8 GB on every Home screen refresh and double-loading on import |
| **Should (P1)** | `SceneIndex`-culled canvas hit-testing (`hit.rs`) & cached/borrowed `LayersState` rows (`layers.rs`) | Eliminates per-frame 723k-node tree walks on mouse move and layer panel render |
| **Should (P1)** | Zero-clone `FontLoader::load_for` (`ensure_font_requests_with`) & higher GPU/CPU `PaintCache` image limits | Eliminates 723k-node deep clone on file open and texture eviction thrashing |
| **Won't** | Changing `.fig` instance expansion architecture (lazy unexpanded instances) | Out of scope for this cycle; requires rewriting all editing commands and layout engine |

## 4. Appetite & No-Gos (Shape Up)
- **Time budget:** Same-session parallel multi-agent sprint.
- **No-Gos:**
  1. Do **not** break backward compatibility with existing `.designos` files or any existing workspace tests (`cargo test --workspace --lib` must pass 100%).
  2. Do **not** introduce new external crate dependencies when `std` + `serde` solve the problem directly.
  3. Do **not** change visual rendering output or break the `designos-gpu` Metal pipeline.

## 5. First-Principles Root-Cause Matrix (Verified on `VSF - PCP.fig`, 723,136 nodes)

| # | Symptom Observed | Root Cause in Code | Fix Strategy |
|---|------------------|--------------------|--------------|
| **RC-1** | 17 MB `.fig` becomes **6.8 GB** `.designos` on disk | `Document::to_json` uses `to_string_pretty`; `images: BTreeMap<String, Vec<u8>>` writes 1 integer per line; `Node` serializes 18 default/empty fields per node across 723,136 nodes | Compact `serde_json::to_string`, `serde_images` (`"hex:..."` with array fallback), and `#[serde(default, skip_serializing_if = "...")]` on `Node`, `ComponentLink`, `TextProps`, `FrameProps` |
| **RC-2** | UI freezes 300ms after every scroll/zoom/pan | `interact.rs` and `ops.rs` call `app.changed()` on view-only changes, bumping `tiles.rev`, which triggers `autosave_tick` -> `doc_hash(&self.doc)` -> `doc.to_json()` (allocating & hashing 6.8 GB on UI thread) AND invalidates `SceneIndex` + `Differ` + `dev_tick` every frame | Remove `app.changed()` from view-only pan/zoom/fit; implement zero-allocation `HashSerializer` for `doc_hash`; cache `dev_tick` `has_tracked` per `rev` |
| **RC-3** | Home screen & file import freeze for 15+ seconds | `Library::file_kind` reads the entire `.designos` file (`std::fs::read`) and parses all 6.8 GB just to read `"kind"`; `import_into_library` calls `lib.load_file` immediately after `lib.import_file` | Read only first 8 KB header in `file_kind` before `"nodes"`; add `import_file_with_doc` returning `(FileEntry, Document)` |
| **RC-4** | Mouse hover & Layers panel lag on 723k-node pages | `hit::hit_in` walks unclipped subtrees without checking `SceneIndex::bounds_of(cid)`; `layers::show` flattens the tree every frame and calls `.cloned()` on visible `Node`s | Pass `Option<&SceneIndex>` into hit-testing to cull by `bounds_of(cid)`; cache `Vec<Row>` in `LayersState` keyed by `(page, rev, expanded_gen, filter)` and borrow `&Node` |
| **RC-5** | Opening a file clones all 723,136 nodes + images; images thrash in GPU cache | `FontLoader::load_for` clones the entire `Document` for the worker thread after already extracting `requests`; `PaintCache` clears all decoded images at `> 256` (GPU) / `> 96` (CPU) | Add `ensure_font_requests_with(&requests, ...)` in `designos-fonts`; raise `PaintCache` image limits to `2048` (GPU) / `1024` (CPU) |
