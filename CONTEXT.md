# DESIGN:OS Studio — Domain Glossary (`CONTEXT.md`)

**Document (`designos_doc::Document`)**:
The canonical in-memory and serialized design/board/deck tree containing pages, `NodeId -> Node` map, shared styles, design variables, flows, comments, library assets, and embedded images.
_Avoid_: file blob, raw kiwi tree, canvas state.

**Document Revision (`TileView::rev`)**:
A monotonically increasing counter (`u64`) that increments ONLY when the `Document` is mutated (`App::changed()`). Never increment on viewport-only changes (pan, zoom, fit, marquee).
_Avoid_: frame counter, view revision.

**SceneIndex (`designos_render::SceneIndex`)**:
Cached spatial and paint-order index of a single page at a specific `(page_id, rev)`, storing world transforms, world bounding boxes (`bounds`), and ancestor clips for fast viewport culling and hit-testing.
_Avoid_: ad-hoc tree walk, per-frame bounds recomputation.

**Compact Native Format (`.designos` / `.designos`)**:
Single-line JSON representation of `Document` with `skip_serializing_if` on default/empty `Node` fields and `"hex:..."` encoding for embedded image bytes, maintaining 100% backward compatibility when loading legacy pretty-printed `.designos` files.
_Avoid_: multi-gigabyte pretty-printed byte arrays.
