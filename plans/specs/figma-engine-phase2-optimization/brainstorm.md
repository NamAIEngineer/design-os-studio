# Brainstorm: Figma Engine Phase 2 Optimization (`figma-engine-phase2-optimization`)

## Phase 0: CLASSIFY
- **Feature**: `"Figma Engine Phase 2 Optimization — Zero-Alloc Kiwi Skip, Streaming Node Conversion, Primitive Geometry Deduplication, Lazy Multi-Page Instance Hydration & Shared GeomCache Hashing"`
- **Signal Scan**:
  - **Scope**: `moderate-to-wide` (6 files across `designos-doc`, `designos-fig`, `designos-gpu`, and `designos-app`: `crates/designos-fig/src/kiwi.rs`, `crates/designos-fig/src/fig.rs`, `crates/designos-fig/src/rest_import.rs`, `crates/designos-doc/src/doc.rs`, `crates/designos-gpu/src/geom.rs`, `crates/designos-app/src/shell.rs`)
  - **Novelty**: `familiar` (extends Phase 1 `.fig` importer, `Document` scene graph, and `GeomCache` deduplication)
  - **Clarity**: `clear` (empirically profiled on `BW - DLS.fig` and `Untitled UI v8.0` + backed by REA `ev_970f4814a2e5b6ea58ee775f70caf0e1b5b426e332a7b864acf9ab00f7e0e53b`)
- **Domain**: `COMPLEX` (`--full` all phases executed end-to-end)

---

## 1. Press Release (Working Backwards)
**Headline**: **design**:os studio Opens 100+ Page Enterprise Figma Design Systems (`Untitled UI v8.0`, `BW - DLS.fig`) in Under 4 Seconds Using 70% Less RAM with On-Demand Page Hydration and Zero-Copy Kiwi Streaming.

**Problem**:
When opening massive multi-page `.fig` files such as `Untitled UI v8.0` (`321 MB` on disk, `113` pages, `312,068` raw Kiwi records, `287,303` component `INSTANCE`s), the Phase 1 pipeline:
1. Allocated and immediately dropped `KiwiValue` trees for `editInfo`, `derivedTextData`, and `derivedImmutableFrameData` inside `Schema::decode_def`.
2. Held all `312,068` `KiwiValue`s in RAM while building `312,068` `serde_json::Value` entries, and cloned fields again inside `convert_node`.
3. Attached redundant pre-baked `fillGeometry` and `strokeGeometry` paths onto rectangular primitives (`FRAME`, `COMPONENT`, `INSTANCE`, `RECTANGLE`) and duplicated `fill_geometry` on `VECTOR` nodes, bloating heap memory and bypassing the GPU renderer's zero-tessellation unit-quad fast path.
4. Eagerly deep-cloned `287,303` component instances across all `113` pages at file open (`878,270` `Node` structs, `13.10 GB` Peak RSS), even though the user views one page at a time.
5. Hashed `NodeKind::Vector` in `GeomCache::shape_hash` using only `(paths.len(), cmds.len())`, causing thousands of distinct 24×24 icons to collide into a single 8-entry bucket.

**Solution**:
1. **Zero-Allocation Kiwi Binary Skip (`kiwi.rs`)**: Advance `Reader::pos` directly over `editInfo`, `derivedTextData`, and `derivedImmutableFrameData` via `Schema::skip_field` without allocating a single `KiwiValue`, `Vec`, or `String`.
2. **In-Place Move `convert_node` & Reverse-Drain `raw_changes` (`fig.rs`)**: Drain `raw_changes` incrementally and move fields out of `Entry.json` (`Map::remove`) inside `convert_node` instead of cloning.
3. **Rectangular Primitive Geometry Pruning & Single-Copy `Vector` Paths (`fig.rs`, `rest_import.rs`)**: Skip redundant pre-baked `fillGeometry`/`strokeGeometry` blobs on rectangular containers/shapes (`FRAME`, `COMPONENT`, `COMPONENT_SET`, `INSTANCE`, `SECTION`, `RECTANGLE`) in large `.fig` imports so they use the analytical GPU fast path, and avoid duplicating `paths` into `n.fill_geometry` on `NodeKind::Vector { network: None, .. }`.
4. **Lazy Multi-Page Instance Hydration (`designos-doc`, `designos-fig`, `designos-app`)**: On large multi-page `.fig` files (`> 25,000` raw nodes and `> 2` pages), eagerly hydrate all master `COMPONENT`s, the initial active page (`pages[0]`), and the largest authored page, while storing remaining pages' pending instances in a compact `DeferredPageHydrator` on `Document` that hydrates any page in `< 20ms` on first selection or inspection.
5. **High-Entropy Vector & Geometry `shape_hash` (`designos-gpu/src/geom.rs`)**: Hash path command coordinates and `fill_geometry`/`stroke_geometry` in `GeomSrc::shape_hash` so repeated icons across component instances hit `GeomCache::shared` in $O(1)$.
