# Performance & Architecture Report: Figma C++/WASM & Orca Terminal Reverse Engineering (`261016`)

- **Date:** 2026-10-16
- **Project:** `designos-studio` (`/Users/jangtrinh/Products/design-os-studio`)
- **Scope:** `crates/designos-core`, `crates/designos-canvas`, `crates/designos-app/src/terminal.rs`

## 1. Figma C++/WASM Engine Reverse-Engineering Lessons (`P8`)
By inspecting production Figma WASM/WebGPU symbols and frame-pipeline invariants, we distilled four core architectural mechanisms into `design:os studio` to hold a locked 60fps (`16.6ms` frame budget) on large documents:

1. **Flat SoA Spatial Indexing (`SceneIndex`):**
   - Instead of pointer-chasing deep Rust tree nodes during pan/zoom culling, `SceneIndex` packs world-space AABBs (`min_x`, `min_y`, `max_x`, `max_y`), visibility flags, and page membership into contiguous Structure-of-Arrays (SoA) slices for cache-friendly frustum culling.
2. **Incremental `DirtyLog` O(k) GPU Synchronization:**
   - Canvas mutations append node IDs to a bounded `DirtyLog` rather than rebuilding the full vertex/instance buffer every frame. Frame preparation updates only the `k` dirty node slots (`O(k)` vs `O(N)`).
3. **Lazy Per-Page Component Instance Expansion:**
   - Master component changes invalidate dependent instances lazily on the active page first; background pages defer instance tree reconciliation until activated.
4. **Off-Main-Thread Image Decode & Texture Upload:**
   - Raster fills decode asynchronously on background worker threads and stream into the GPU texture atlas without stalling the egui/wgpu UI thread.

## 2. Orca Ghostty Terminal Reverse-Engineering Lessons (`P7`)
By analyzing Orca's native Ghostty terminal integration, we upgraded `crates/designos-app/src/terminal.rs`:
- **Per-File Session Isolation:** Each document tab owns its own `TerminalSession` map keyed by document ID so multi-agent workflows never cross-contaminate state or working directories.
- **Procedural Box-Drawing (`U+2500..U+257F`):** Font glyphs for TUI borders often leave subpixel seams at fractional DPI; rendering box-drawing ranges procedurally as crisp pixel-snapped quads eliminates gaps in Claude Code and `design-os` CLI tables.
- **Canvas Shortcut Suppression:** When `Panel/AgentTerminal` holds keyboard focus, single-key tool shortcuts (`V`, `F`, `R`, `T`, `Backspace`, `Space`) are routed exclusively to the PTY input stream.
