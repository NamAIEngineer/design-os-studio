# Technical Plan: Figma Engine Phase 2 Optimization (`figma-engine-phase2-optimization`)

## Agreed Seams Under Test
1. `designos_fig::import_fig(&[u8]) -> Result<Document, FigError>`
2. `designos_doc::Document::{ensure_page_hydrated, hydrate_all_pages, is_page_hydrated}`
3. `designos_gpu::SceneCompiler::compile` & `GeomCache::get`
4. Real-file benchmark harness: `/Users/jangtrinh/Downloads/benchmark_fig_load.py` on `BW - DLS.fig` and `Untitled UI v8.0`

---

## Vertical Slices

### Slice 1: Zero-Allocation `Schema::skip_field` & In-Place Move Conversion (`crates/designos-fig/src/kiwi.rs`, `crates/designos-fig/src/fig.rs`)
- **What it delivers**: Zero-heap-allocation binary skipping of `editInfo`, `derivedTextData`, and `derivedImmutableFrameData` in `kiwi.rs`, plus reverse-chunked draining of `raw_changes` so `Vec<KiwiValue>` memory is released before `entries_to_rest`.
- **Blocked by**: None.

### Slice 2: Rectangular Primitive Geometry Pruning, Single-Copy Vector Outlines & Zero-Alloc Clone Override Lookup (`crates/designos-fig/src/fig.rs`, `crates/designos-fig/src/rest_import.rs`)
- **What it delivers**:
  1. Skips redundant pre-baked `fillGeometry`/`strokeGeometry` on `FRAME`, `COMPONENT`, `COMPONENT_SET`, `INSTANCE`, `SECTION`, and `RECTANGLE` on large `.fig` imports (`> 5,000` nodes), unlocking the zero-tessellation GPU unit-quad fast path.
  2. Avoids duplicating `Vec<Path>` into `n.fill_geometry` on `NodeKind::Vector { network: None, .. }` during large `.fig` imports.
  3. Eliminates `self.override_keys.insert(id, k)` on cloned instance children inside `clone_subtree` by resolving `override_keys` through `n.component.source` in `node_matches` and `build_subtree_lookup`.
- **Blocked by**: Slice 1.

### Slice 3: On-Demand / Lazy Multi-Page Instance Hydration (`crates/designos-doc/src/doc.rs`, `crates/designos-fig/src/rest_import.rs`, `crates/designos-app/src/shell.rs`)
- **What it delivers**:
  1. Adds `PageHydrator` trait (`Send + Sync`) and `Document::ensure_page_hydrated(&mut self, page: NodeId) -> bool` + `Document::hydrate_all_pages(&mut self)` to `designos-doc`.
  2. In `rest_import.rs::finish`, when `doc.nodes.len() > 25_000 && doc.pages.len() > 2`:
     - Hydrates all master `COMPONENT`s (`expand_nested`), `doc.pages[0]`, and the largest authored page immediately.
     - Stores unexpanded pages' instance hydration state in a `DeferredFigHydrator` attached to `Document`.
  3. Hooks `app.doc.ensure_page_hydrated(app.page)` into `designos-app` whenever a page is selected or rendered.
- **Blocked by**: Slice 2.

### Slice 4: High-Entropy `GeomCache::shape_hash` for Vector & Imported Geometry (`crates/designos-gpu/src/geom.rs`)
- **What it delivers**: Hashes path command coordinates (`MoveTo`, `LineTo`, `QuadTo`, `CubicTo`) and `fill_geometry`/`stroke_geometry` in `GeomSrc::shape_hash`, eliminating hash collisions among same-size icons and maximizing `shared_hits`.
- **Blocked by**: None.
