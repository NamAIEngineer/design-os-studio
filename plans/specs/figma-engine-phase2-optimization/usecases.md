# Use Cases & Acceptance Criteria: Figma Engine Phase 2 Optimization

## Use Case 1 (`UC-01`): Zero-Allocation Kiwi Binary Skip (`kiwi.rs`)
- **Given** a `.fig` message payload `> 2 MB` containing `editInfo`, `derivedTextData`, or `derivedImmutableFrameData` fields
- **When** `Schema::decode_message_for_import` decodes the message
- **Then** `Schema::skip_field` advances the byte reader past those fields with `0` heap allocations and produces the exact same `KiwiValue` output as decoding and dropping them.

## Use Case 2 (`UC-02`): Rectangular Primitive Geometry Pruning & Single-Copy Vector Outlines (`fig.rs`, `rest_import.rs`)
- **Given** a large `.fig` file (`> 5,000` nodeChanges) containing `FRAME`, `COMPONENT`, `INSTANCE`, `RECTANGLE`, and `VECTOR` nodes
- **When** `import_fig` imports the document
- **Then** rectangular nodes derive their geometry analytically from `size`, `corner_radii`, `corner_smoothing`, and `stroke` (unlocking the zero-tessellation GPU unit-quad fast path in `SceneCompiler`), `VECTOR` nodes with `network: None` store their paths once in `NodeKind::Vector { paths }` without duplicating them into `n.fill_geometry`, and `BOOLEAN_OPERATION` / `STAR` / `REGULAR_POLYGON` / `ELLIPSE` nodes retain their imported geometry.

## Use Case 3 (`UC-03`): Lazy Multi-Page Instance Hydration (`designos-doc`, `designos-fig`, `designos-app`)
- **Given** a multi-page enterprise `.fig` file (`> 25,000` raw nodes and `> 2` pages, such as `Untitled UI v8.0` with `113` pages or `BW - DLS.fig` with `25` pages)
- **When** `import_fig` opens the file
- **Then** all master `COMPONENT`s, `pages[0]`, and the largest authored page are hydrated immediately, while inactive background pages defer `INSTANCE` subtree cloning until `doc.ensure_page_hydrated(page_id)` is called (automatically triggered on page switch in `designos-studio` or on `doc.hydrate_all_pages()`), cutting Peak RSS on `Untitled UI v8.0` from `13.10 GB` to `< 5.5 GB` and parse time from `8.6s` to `< 4.5s`.

## Use Case 4 (`UC-04`): High-Entropy Vector & Geometry `GeomCache` Deduplication (`designos-gpu`)
- **Given** a page with hundreds of distinct vector icons instantiated across component instances
- **When** `SceneCompiler` queries `GeomCache::get`
- **Then** `GeomSrc::shape_hash` hashes vector command coordinates (`MoveTo`/`LineTo`/`CubicTo`) and `fill_geometry`/`stroke_geometry` so distinct icons do not collide into the same 8-entry bucket, and identical icon instances share the same `Rc<NodeGeom>`.
