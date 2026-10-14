# Use Cases & Acceptance Criteria: Large `.fig` File Performance (`large-fig-performance`)

## Seams Under Test
1. **Serialization Seam (`designos_doc::Document::to_json` / `from_json`)**: Public JSON round-trip contract for `Document`, `Node`, and `images`.
2. **Library Storage Seam (`designos_library::Library::file_kind` / `import_file_with_doc`)**: Public library metadata inspection and file import contract.
3. **Font Loading Seam (`designos_fonts::ensure_font_requests_with`)**: Public font resolution entrypoint accepting `&[FontRequest]` without requiring `&Document`.
4. **Canvas & Session Seam (`designos_app`)**: Viewport pan/zoom revision stability (`tiles.rev()`), `doc_hash` determinism, `SceneIndex`-assisted `hit_test`, and `LayersState` row caching.

---

## UC-1: Compact & Backward-Compatible Document Serialization
**Actor:** Editor / Library Store / MCP Server
### Gherkin Acceptance Criteria
```gherkin
Scenario: Default Node fields and embedded images serialize compactly and round-trip losslessly
  Given a Document containing nodes with default properties and embedded binary images
  When Document::to_json() is called
  Then the resulting JSON omits default empty fields ("source_id", "children", "fills", "effects", "style_refs", "fill_geometry", "stroke_geometry")
  And embedded images are encoded as compact "hex:..." strings instead of multi-line integer arrays
  And Document::from_json() restores an identical Document equal to the original

Scenario: Legacy .designos files with byte-array images and full fields still load
  Given a legacy JSON string where "images" values are JSON byte arrays [137, 80, 78, 71]
  When Document::from_json() parses the string
  Then the embedded image bytes [137, 80, 78, 71] are restored without error
```

## UC-2: Viewport Pan/Zoom Revision Stability & Zero-Allocation `doc_hash`
**Actor:** Designer panning and zooming on the canvas
### Gherkin Acceptance Criteria
```gherkin
Scenario: Panning, zooming, and fitting the viewport do not increment document revision
  Given an open document in App with initial tile revision R
  When the user pans, zooms (`zoom_to`, `zoom_on_content`), or fits (`fit_all`, `fit_selection`)
  And a frame is painted (`take_rev`)
  Then `app.canvas.tiles.rev()` remains equal to R
  And `autosave_tick` does not schedule a document probe on view-only navigation

Scenario: Zero-allocation doc_hash detects real document mutations accurately
  Given two identical Documents D1 and D2
  When `doc_hash(&D1)` and `doc_hash(&D2)` are computed without allocating a JSON String
  Then `doc_hash(&D1) == doc_hash(&D2)`
  And mutating any node property or image in D2 changes `doc_hash(&D2)`
```

## UC-3: Fast Library Header Probe & Single-Pass Import
**Actor:** Designer opening Home screen or importing a `.fig` file into a project
### Gherkin Acceptance Criteria
```gherkin
Scenario: Library::file_kind reads DocumentKind from the 8 KB file header
  Given a saved library file of kind Design, Board, or Deck
  When `lib.file_kind(&entry.id)` is called
  Then it returns the exact `DocumentKind` by parsing only the header before `"nodes"`

Scenario: Importing a file into the library returns both FileEntry and Document
  Given an external `.fig` or `.designos` file path
  When `lib.import_file_with_doc(path, project, folder)` is called
  Then the file is saved into the project and `(FileEntry, Document)` is returned without re-reading the saved file from disk
```

## UC-4: Spatial Hit-Testing, Cached Layer Rows & Zero-Clone Font Requests
**Actor:** Designer hovering/clicking on canvas, scrolling Layers panel, or opening a file
### Gherkin Acceptance Criteria
```gherkin
Scenario: SceneIndex-culled hit_test matches exact hit results while skipping distant subtrees
  Given a populated page and its compiled `SceneIndex`
  When `hit::hit_test` and `hit::deep_hit_test` are queried at any canvas point `p`
  Then they return the exact same topmost `NodeId` as unindexed hit-testing while skipping subtrees whose world bounds do not contain `p`

Scenario: FontLoader::load_for resolves fonts from &[FontRequest] without cloning Document
  Given a Document with text nodes requesting font families
  When `ensure_font_requests_with(&requests, &db, fetcher)` is called
  Then all requested fonts are loaded identically to `ensure_fonts_with(&doc, &db, fetcher)`
```
