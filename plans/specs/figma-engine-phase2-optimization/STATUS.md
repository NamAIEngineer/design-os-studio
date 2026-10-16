# STATUS — Figma Engine Phase 2 Optimization (`figma-engine-phase2-optimization`)

- **Feature**: `figma-engine-phase2-optimization`
- **Pipeline**: `es:feature-dev` (`Classify -> Brainstorm -> Plan -> Dev -> Review -> Finalize`)
- **Date**: 2026-10-16
- **Status**: **SHIPPED & VERIFIED**

---

## 1. Delivered Architectural Slices

| Slice | Target Files | What Was Built |
|---|---|---|
| **Slice 1 — Zero-Allocation Kiwi Wire-Level Field Skipper (`OPT-P2-01`)** | `crates/designos-fig/src/kiwi.rs` | Added `Reader::skip_string`, `Schema::{skip_def, skip_field, skip_type}`, and `KiwiValue::get_mut`. Updated `Schema::decode_def` to advance the wire cursor past `editInfo`, `derivedTextData`, and `derivedImmutableFrameData` on large `.fig` files (`> 2 MB`) **before** allocating any `KiwiValue` heap structures. |
| **Slice 2 — Single-Owner Vector Geometry & Zero Rectangular/Text Pre-Baked Outlines (`OPT-P2-02` & `OPT-P2-03`)** | `crates/designos-fig/src/fig.rs`, `crates/designos-fig/src/rest_import.rs` | Filtered `extract_fast_derived` (`take_field("derivedSymbolData")` in-place) to only retain `fillGeometry`/`strokeGeometry` for non-rectangular/non-text shapes (`VECTOR`, `BOOLEAN_OPERATION`, `STAR`, `REGULAR_POLYGON`, `ELLIPSE`, `LINE`, `BRUSH`), eliminating `200,000+` pre-baked `TEXT` glyph outlines and `300,000+` rectangular `fill_geometry` paths inside instances. Moved `fill_geo` directly into `NodeKind::Vector { paths, network: None }` without a 2× duplicate in `n.fill_geometry`. Replaced `566,202` cloned `override_keys` entries with on-demand `override_key_of(id)` chain resolution. |
| **Slice 3 — Lazy / On-Demand Page Instance Hydration (`OPT-P2-04`)** | `crates/designos-doc/src/lib.rs`, `crates/designos-fig/src/rest_import.rs`, `crates/designos-app/src/ops.rs`, `crates/designos-app/src/shell.rs` | Added `PageHydrator` trait (`hydrate_page`, `hydrate_all`, `is_hydrated`) and `Document::{ensure_page_hydrated, hydrate_all_pages}`. On large multi-page `.fig` files (`> 5,000` nodes), `import_fig` expands component instances on the active page (`pages[0]`) and main components (`COMPONENT` / `COMPONENT_SET`), while deferring inactive pages (`pages[1..]`) to `DeferredFigHydrator` for instant on-demand hydration when `switch_page` is called. Also enabled GPU instancing + spatial R-tree culling for `--screenshot` on large documents (`> 5,000` nodes). |
| **Slice 4 — GPU `GeomCache` `VectorPath` Content Hashing (`OPT-P2-05`)** | `crates/designos-gpu/src/geom.rs` | Added `hash_path_slice` in `GeomSrc::shape_hash` to hash sampled path commands (`MoveTo`, `LineTo`, `QuadTo`, `CubicTo`, `Close`) and fill rules for `Vector`, `fill_geometry`, and `stroke_geometry` (plus `Star`, `Polygon`, `Ellipse` parameters), and raised the per-hash shared bucket cap from `8` to `16`. |

---

## 2. Empirical Benchmark Results (Real `.fig` Design Systems)

| Metric | `BW - DLS.fig` (`129.63 MB`, `25` pages) — Before | `BW - DLS.fig` — After Phase 2 | `Untitled UI v8.0.fig` (`321.04 MB`, `113` pages) — Before | `Untitled UI v8.0.fig` — After Phase 2 |
|---|---|---|---|---|
| **`import_fig` Parse Time** | `1.33 s` | **`1.28 s`** | `8.60 s` | **`5.78 s` (`-32.8%`)** |
| **`import_fig` Peak RSS** | `3,830 MB` (`3.83 GB`) | **`2,340 MB` (`2.28 GB`, `-39%`)** | `13,100 MB` (`13.10 GB`) | **`9,180 MB` (`8.96 GB`) / `6.10 GB` footprint** |
| **Initial Hydrated Nodes** | `80,778` nodes | **`68,698` nodes** | `878,270` nodes | **`208,610` nodes (`-76.2%`)** |
| **Full App CLI `--screenshot` (Import + Layout + Render + Save)** | `8.18 s` / `6.34 GB` | **`2.94 s` (`-64.1%`) / `4.42 GB` (`2.70 GB` footprint)** | `~15.0 s` / `13.10+ GB` | **`6.77 s` (`-54.9%`) / `6.10 GB` footprint** |

---

## 3. Verification Evidence

- `cargo test -p designos-doc --lib`: **160 passed; 0 failed**
- `cargo test -p designos-fig --lib`: **28 passed; 0 failed** (including `kiwi::tests::skip_heavy_fields_without_allocation`)
- `cargo test -p designos-fig --tests`: **89 passed; 0 failed** across all 15 integration test suites
- `cargo test -p designos-gpu --lib`: **3 passed; 0 failed** (including `geom::tests::distinct_vector_paths_do_not_collide_in_shape_hash`)
- `python3 /Users/jangtrinh/Products/design-os-github-page/scripts/verify-repo-delivery.py /Users/jangtrinh/Products/design-os-studio`: **PASS (`D1..D5`)**
