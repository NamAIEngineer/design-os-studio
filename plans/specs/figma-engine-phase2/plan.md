# Phase 2 Figma-Engine Architecture Specification

**Target Workspace**: `design-os-studio` (`designos-doc`, `designos-render`, `designos-gpu`, `designos-app`)  
**Benchmark Workload**: 723,000+ nodes across 31 pages (`Simple Design System` + enterprise `.fig` stress corpus)  
**Design Inspiration**: Figma C++/WebGPU Canvas Engine (lazy page chunking, spatial R-tree + incremental dirty propagation, symbol/instance DAG geometry sharing, analytical SDF uber-shader)

---

## Executive Summary & Phase 1 Baseline

Phase 1 eliminated full-tree per-frame clones (`Arc<Document>`), cached visible layer rows, threaded `SceneIndex` bounds into canvas hit-testing/marquee/guide snap, skip-culled hidden subtrees in `SceneIndex::build`, and decoupled `GpuScene` geometry upload from viewport pan/zoom.

To scale cleanly to **700k–1M+ nodes** with sub-frame (`<4ms`) interaction latency and `<30ms` time-to-first-frame on cold open, Phase 2 introduces four architectural epics modeled after Figma's production C++/WebGPU engine:

| Epic | Subsystem | Primary Bottleneck Solved | Target SLA |
| :--- | :--- | :--- | :--- |
| **Epic 1** | `designos-doc` | Monolithic parse of 723k nodes blocks initial canvas paint | `<30ms` active page open (`~25k` nodes); background stream remaining 30 pages |
| **Epic 2** | `designos-render`, `designos-app` | $O(N_{\text{page}})$ index rebuild on single-node drag & linear child scan | `<0.3ms` incremental `update_subtree(dirty_id)`; $O(\log N + K)$ spatial queries |
| **Epic 3** | `designos-doc`, `designos-gpu` | Duplicate Lyon tessellation across thousands of `Instance` nodes | $\ge 70\%$ reduction in vertex/index buffer memory and build time on DS files |
| **Epic 4** | `designos-gpu` | Lyon CPU tessellation for rects/rrects & offscreen MSAA clip render passes | $0$ CPU tessellation for Rect/RRect/Stroke; single-pass analytical SDF clip stack |

---

## Epic 1: Binary Chunked / `mmap` Document Format (`.designos` v2)

### 1. Problem Statement
Opening a 723k-node, 31-page document currently deserializes every page's node tree upfront before the first frame can render. Even at high throughput, parsing all 723k nodes takes hundreds of milliseconds, whereas the user only views **one active page** (`~25k` nodes) on launch and needs shared `Component` / `ComponentSet` definitions and document-level variables/styles.

### 2. File Layout (`.designos` v2 Container)
The v2 container is a 64-byte-aligned binary format mapped via `memmap2::Mmap` (with an in-memory slice fallback for WASM/tests):

```text
+-------------------------------------------------------------------+
| Header (64 B): magic b"DSGM2\0\0\0", version, flags, toc_offset   |
+-------------------------------------------------------------------+
| Chunk 0: Document Manifest & Page Directory (zstd / raw)          |
|   - Page entries: [PageId, name, node_count, chunk_id, byte_span] |
|   - Active page hint, canvas backgrounds                          |
+-------------------------------------------------------------------+
| Chunk 1: Shared Definitions Chunk                                 |
|   - Variable collections, modes, tokens, paint/text styles        |
|   - Master `Component` / `ComponentSet` subtrees (cross-page DAG) |
|   - Image blob table (content-addressed SHA-256 -> byte range)    |
+-------------------------------------------------------------------+
| Chunk 2..=P+1: Per-Page Node Chunks (1 chunk per Page)            |
|   - Contiguous SoA/AoS binary records for page subtree            |
|   - Decodable independently in <30ms for ~25k nodes               |
+-------------------------------------------------------------------+
```

### 3. Runtime Representation & Background Streaming (`designos-doc`)
- **`PageState` Lifecycle**:
  ```rust
  pub enum PageResidency {
      Unloaded { chunk_offset: u64, chunk_len: u32, node_count: u32 },
      Loading,
      Loaded,
  }
  ```
- **Cold Open Path (`<30ms`)**:
  1. `mmap` file and read `Header` + `Chunk 0 (Manifest)` + `Chunk 1 (Shared Definitions)`.
  2. Immediately decode the active page chunk (`~25k` nodes) on the main thread or high-priority worker.
  3. Hand `Arc<Document>` to `designos-app` for immediate first-frame GPU render.
- **Background Page Streaming**:
  - Spawn background rayon/thread tasks to decode remaining 30 page chunks in priority order (adjacent pages first, or on-demand if the user clicks an unloaded page tab).
  - Each decoded `PageChunk` merges into `Document` via an atomic revision bump (`doc.merge_loaded_page(page_chunk)`), without invalidating the active page's `SceneIndex` or `GpuScene`.

### 4. Verification Gate
- Unit & roundtrip test: serialize a multi-page document with cross-page component masters into `.designos` v2, open with lazy loader, verify active page renders identically before background pages finish, and verify full document parity once all chunks settle.
- Benchmark gate: cold-open active page of 723k-node corpus in `<30ms` wall-clock on Apple Silicon M-series.

---

## Epic 2: Hierarchical R-Tree / Grid Spatial Index & Incremental Dirty Subtree Rebuild

### 1. Problem Statement
`SceneIndex::ensure_prev` currently walks the entire active page (`~25k` nodes) whenever `rev` increments, and `Differ::diff` iterates all `new.entries` to compare against `self.snap`. During a 120Hz interactive drag of a single node or frame, rebuilding 25k entries and diffing 25k nodes wastes `2–6ms` per frame. Furthermore, viewport culling and hit-testing still walk top-level container children linearly.

### 2. Incremental Subtree Update (`SceneIndex::update_subtree`)
- **Explicit Dirty Tracking in `Document`**:
  - Record mutated `NodeId`s per revision in a compact `DirtySet`:
    ```rust
    pub struct DirtySet {
        pub rev: u64,
        pub modified_roots: SmallVec<[NodeId; 8]>,
        pub structure_changed: bool,
        pub variables_changed: bool,
    }
    ```
- **Bottom-Up / Subtree Bounds Propagation**:
  - `pub fn update_subtree(&mut self, doc: &Document, dirty_id: NodeId, rev: u64)`:
    1. Recompute `world_transform` for `dirty_id` by walking its ancestor chain (`O(depth)`, typically `depth <= 12`).
    2. Re-run `self.build(doc, dirty_id, parent_ts, parent)` **only** for `dirty_id`'s subtree (stopping immediately at hidden children).
    3. Walk upward from `dirty_id.parent` to `page`, recomputing each ancestor's `Entry.bounds` as the union of its own local bounds and its children's cached `Entry.bounds` (respecting `clips_content`). Stop ascending as soon as an ancestor's `bounds` is unchanged.
  - Complexity drops from $O(N_{\text{page}})$ (`25,000` nodes) to $O(|T_{\text{dirty}}| + \text{depth})$ (`1–50` nodes for typical drags, `<0.15ms`).

### 3. Hierarchical Spatial Acceleration (`GridRTree`)
- Augment `SceneIndex` with a two-level spatial accelerator over top-level and mid-level containers:
  - **Level 1 (Uniform Loose Grid / STR R-Tree)**: Buckets page-space AABBs into `512x512` world-unit cells (or a flat packed 4-ary Sort-Tile-Recursive R-Tree array `Vec<RTreeNode>`).
  - **Fast Queries**:
    - `SceneIndex::query_rect(rect, &mut out)` runs in $O(\log N + K)$ time instead of $O(N)$.
    - `canvas::hit::hit_test_with_index`, `marquee_select_with_index`, and `guides::candidates` query the R-Tree directly to candidate-filter top-level subtrees in `<50µs`.

### 4. Verification Gate
- Property test: for 100 random single-node and container transforms/edits, assert `SceneIndex::update_subtree` produces byte-identical `Entry` bounds and `Differ::Changes` rects as a full `SceneIndex::ensure_prev` rebuild.
- Benchmark gate: single-node drag step on a 25k-node page completes index + diff update in `<0.3ms`.

---

## Epic 3: Component `Symbol` / `Instance` DAG Geometry Deduplication

### 1. Problem Statement
In large design systems (such as `Simple Design System` with 723k nodes), `60%–85%` of leaf nodes live inside `NodeKind::Instance` subtrees cloned from master `Component`s. Currently, `designos-doc::resolve_node` resolves overrides per node and `designos-gpu` tessellates and stores a separate `BuiltNode` mesh for every single instance node, multiplying CPU Lyon tessellation work and GPU vertex buffer size by $10\times$–$50\times$.

### 2. Content-Addressed Canonical Geometry Cache (`designos-gpu`)
- **Geometry Fingerprint (`GeomKey`)**:
  - Separate **intrinsic geometry** (shape kind, unscaled `size`, corner radii, stroke weight/align/join/cap/dash, boolean path, vector network) from **instance placement & paint** (world `Affine` transform, solid fill color override, opacity, blend mode).
  - Define a 128-bit canonical key:
    ```rust
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GeomKey(pub u128);
    ```
    computed from `(source_component_node_id, quantized_size, corner_radii_bits, stroke_params_hash, vector_hash)`.
- **Shared `Arc<BuiltGeom>` Pool**:
  ```rust
  pub struct SharedMesh {
      pub fill_vbo_slice: Range<u32>,
      pub fill_ibo_slice: Range<u32>,
      pub stroke_vbo_slice: Range<u32>,
      pub stroke_ibo_slice: Range<u32>,
      pub local_bounds: (f32, f32, f32, f32),
  }
  ```
  - When building `GpuScene`, if an `Instance` child has no geometry-altering override (only transform, visibility, or color/token override), it looks up `GeomKey` in `MeshPool` (`IdMap` / `HashMap<GeomKey, Arc<SharedMesh>>`).
  - All matching instances emit lightweight **GPU Draw Commands** referencing the shared `vbo_slice` / `ibo_slice` with per-instance `(world_transform, fill_rgba, stroke_rgba, clip_id)` uniform/instance attributes.

### 3. Verification Gate
- Visual parity test: render a page with 500 instances of a complex icon/button component (with color, text, and nested visibility overrides) and assert pixel-for-pixel equivalence against uncached tessellation.
- Memory & build gate: $\ge 70\%$ reduction in Lyon tessellation invocations and GPU vertex buffer bytes on the `Simple Design System` benchmark file.

---

## Epic 4: GPU SDF Uber-Shader for Rects, RRects & Clips

### 1. Problem Statement
Across real-world Figma files, **over 80% of rendered primitives** are axis-aligned or rotated Rectangles, Rounded Rectangles (with uniform or per-corner radii), Frames, Pills, and Borders, plus rounded-rectangle `clips_content` masks. Tessellating rounded corners and strokes on the CPU with Lyon generates hundreds of triangles per node, and nested rounded-rect clipping requires expensive GPU stencil or offscreen MSAA render target switches.

### 2. Analytical Instanced SDF Primitive Pipeline (`designos-gpu`)
- **Zero-Tessellation `SdfRectInstance` (96 bytes, 16-byte aligned)**:
  ```rust
  #[repr(C)]
  #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
  pub struct SdfRectInstance {
      /// Columns 0..2 of 2D affine transform (local -> world) + translation
      pub transform: [f32; 6],
      /// Half-extents (w/2, h/2) in local space
      pub half_size: [f32; 2],
      /// Per-corner radii: (top_left, top_right, bottom_right, bottom_left)
      pub radii: [f32; 4],
      /// Premultiplied fill color (or gradient lut index)
      pub fill_color: [f32; 4],
      /// Premultiplied stroke color
      pub stroke_color: [f32; 4],
      /// (stroke_weight, stroke_align_offset, clip_index, flags)
      pub params: [f32; 4],
  }
  ```
- **Fragment Shader Evaluation**:
  - Evaluates the exact 2D rounded-box signed distance field:
    $$d(p) = \|\max(|p| - b + r, 0)\|_2 + \min(\max(|p|_x - b_x + r, |p|_y - b_y + r), 0) - r$$
  - Computes analytical screen-space anti-aliasing via `fwidth(d)` (`smoothstep(-0.5 * fw, 0.5 * fw, d)`), eliminating MSAA requirement for rect/rrect edges while producing razor-sharp 1x/2x DPI edges at any zoom level.
  - Evaluates inner, center, and outer strokes analytically in the same single quad draw via `abs(d - stroke_offset) - stroke_half_width`.

### 3. Analytical GPU Clip Stack (`ClipBuffer`)
- Upload active container clip frames (`clips_content == true`) into a GPU storage/uniform buffer `Vec<GpuClipRect>` where each entry stores `(inv_transform, half_size, radii, parent_clip_index)`.
- Any primitive (SDF quad, Lyon mesh, or glyph quad) carries a `clip_index: u32` pointing to its innermost clipping frame.
- The fragment shader walks at most `K <= 4` nested rounded-rect clips analytically and multiplies coverage by the intersection of their SDF coverages—eliminating render-pass breaks for 99% of frame clips.

### 4. Verification Gate
- Pixel-diff gate: compare SDF-rendered rounded rects (uniform radii, independent 4-corner radii, inside/center/outside strokes, rotated clipped frames) against the `tiny-skia` software reference renderer (`PSNR >= 42 dB`).
- Performance gate: 50,000 rounded rectangles + borders draw in a single instanced draw call in `<1.2ms` GPU frame time at 4K resolution.

---

## Rollout Order & Dependency DAG

```text
[Epic 2: Incremental SceneIndex + R-Tree]  --->  Immediate 120Hz drag/hit-test win
                |
                v
[Epic 4: GPU SDF Uber-Shader + Clip Stack] --->  Eliminates 80% of Lyon tessellation
                |
                v
[Epic 3: Symbol/Instance Mesh Pool]        --->  Deduplicates remaining 20% vector/boolean meshes
                |
                v
[Epic 1: .designos v2 Chunked mmap Loader]  --->  Sub-30ms cold start on 723k-node files
```
