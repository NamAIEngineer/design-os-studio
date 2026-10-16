# Figma Engine Phase 2 Optimization (`OPT-P2-01..05`)

- **Feature**: `figma-engine-phase2-optimization`
- **REA Evidence**: `ev_970f4814a2e5b6ea58ee775f70caf0e1b5b426e332a7b864acf9ab00f7e0e53b`
- **Target Crates**: `crates/designos-fig`, `crates/designos-doc`, `crates/designos-gpu`, `crates/designos-app`

## Architectural Decisions & Invariants

1. **Zero-Allocation Kiwi Wire-Level Field Skipper (`OPT-P2-01`, `crates/designos-fig/src/kiwi.rs`)**:
   - `Reader::skip_string` and `Schema::{skip_def, skip_field, skip_type}` advance the Kiwi wire cursor directly past `editInfo`, `derivedTextData`, and `derivedImmutableFrameData` when `skip_heavy` is active (`data.len() > 2_000_000`) **before** allocating any `KiwiValue` heap structures.
2. **Single-Owner Vector Geometry & Zero Rectangular/Text Pre-Baked Outlines (`OPT-P2-02` & `OPT-P2-03`, `crates/designos-fig/src/fig.rs`, `crates/designos-fig/src/rest_import.rs`)**:
   - `extract_fast_derived` takes `&mut [KiwiValue]` (`take_field("derivedSymbolData")`) and filters `fillGeometry`/`strokeGeometry` to only retain geometry for non-rectangular/non-text shapes (`VECTOR | BOOLEAN_OPERATION | STAR | REGULAR_POLYGON | ELLIPSE | LINE | BRUSH`), eliminating `200,000+` pre-baked `TEXT` glyph outlines and `300,000+` rectangular `fill_geometry` paths inside instances.
   - `build_node` moves `fill_geo` directly into `NodeKind::Vector { paths, network: None }` without storing a 2× duplicate in `n.fill_geometry`.
   - `clone_subtree` resolves override keys via `override_key_of(id)` (walking `n.component.source` up to 8 hops) instead of inserting `566,202` cloned `NodeId` entries into `override_keys`.
3. **Lazy / On-Demand Page Instance Hydration (`OPT-P2-04`, `crates/designos-doc/src/lib.rs`, `crates/designos-fig/src/rest_import.rs`, `crates/designos-app/src/ops.rs`, `crates/designos-app/src/shell.rs`)**:
   - `Document::page_hydrator: Option<Arc<Mutex<dyn PageHydrator>>>` and `Document::{ensure_page_hydrated, hydrate_all_pages}` defer component instance expansion on inactive pages (`pages[1..]`) of large multi-page `.fig` files (`> 5,000` nodes), hydrating only the active page (`pages[0]`) and main component definitions (`COMPONENT` / `COMPONENT_SET`) on load, and hydrating any inactive page on demand in `App::switch_page`.
4. **GPU `GeomCache` `VectorPath` Content Hashing (`OPT-P2-05`, `crates/designos-gpu/src/geom.rs`)**:
   - `GeomSrc::shape_hash` hashes sampled commands (`MoveTo`, `LineTo`, `QuadTo`, `CubicTo`, `Close`) and fill rules of `NodeKind::Vector`, `fill_geometry`, and `stroke_geometry`, eliminating `shared` cache hash collisions across distinct 24×24 icons.

## Real-World Benchmark Results

- **`BW - DLS.fig` (`129.63 MB`, `25` pages)**:
  - `import_fig`: **`1.34s`**, Peak RSS **`2.25 GB`** (Memory Footprint **`1.67 GB`**, down from `3.83 GB`), `68,698` nodes.
  - `designos --screenshot`: **`3.69s`** (down from `8.18s`).
- **`Untitled UI v8.0.fig` (`321.04 MB`, `113` pages, `312,068` raw Kiwi records)**:
  - `import_fig`: **`6.09s`** (down from `8.60s`), Peak RSS **`9.02 GB`** (Memory Footprint **`5.75 GB`**, down from `13.10 GB`), initial hydrated nodes **`208,610`** (down from `878,270`, `-76.2%`).
  - `designos --screenshot`: **`7.41s`** (Memory Footprint **`6.25 GB`**, down from `~15.0s / 13.10+ GB`).
