# Session Handoff — DESIGN:OS Studio (`2026-10-14`)

## 1. Executive Summary (Current State as of 2026-10-14)

- **Product Identity & Standalone Ownership:**
  - Established **DESIGN:OS Studio** as a standalone repository (`https://github.com/jangtrinh/design-os-studio`, `isFork: false`, binaries `designos-studio` / `designos` and `designos-mcp` / `designos-mcp`).
  - Separated upstream tracking into a dedicated mirror (`https://github.com/jangtrinh/designos-upstream` / `/Users/jangtrinh/Products/designos-upstream`) managed via `./scripts/upstream-radar.sh` (`plans/upstream-radar/latest.md`).
  - Applied the **EaseUI BrainWave 2.0** dark/light theme (`crates/designos-app/src/theme.rs`: `#121215` canvas, `#18181B` panels, `#27272A` surface/borders, `#E36323` BrainWave orange brand accent, `#3582FF` canvas blue selection, `#8755E9` component violet).
  - Embedded the **spherical orange DESIGN:OS Studio logo** across `assets/icon/png/`, `assets/icon/designos.icns`, `assets/icon/designos-studio.icns`, window/Dock icon (`crates/designos-app/src/main.rs`), and top-left toolbar painter (`crates/designos-app/src/icons.rs`).
- **Enterprise `.fig` Storage Compaction (`VSF - PCP.fig`, 723,136 nodes):**
  - Compact single-line JSON serialization (`skip_serializing_if` on `Node`, `FrameProps`, `TextProps`, `ComponentLink`) + `"hex:..."` image serialization (`serde_images` in `crates/designos-doc/src/lib.rs`) reduced the on-disk library file `/Users/jangtrinh/Library/Application Support/designos/library/files/18de678a186512c80000.designos` from **`6.8 GB` -> `242 MB` (`-96.5%`, `28.3x` smaller)** with 100% backward compatibility.
- **Fixed Frame-1 `parking_lot::RwLock` Self-Deadlock (`crates/designos-app/src/icons.rs`):**
  - Root cause: `icons::paint_logo` called `ctx.load_texture(...)` inside `ctx.data_mut(|d| ...)`, attempting to acquire a second write lock on `egui`'s non-reentrant `parking_lot::RwLock<ContextImpl>` on Frame 1 and hanging the process immediately on launch.
  - Fix: Split `paint_logo` so `ctx.data_mut` only reads the cached `Option<TextureHandle>`, drops the lock before decoding the PNG and calling `ctx.load_texture("designos-logo", ...)`, and then stores the handle in a separate `ctx.data_mut` call.
- **Fixed `egui-wgpu` Metal Staging Buffer Crash & 100% CPU Loop (`crates/designos-gpu` & `crates/designos-app`):**
  - Implemented `FrameKey` reuse in `crates/designos-app/src/canvas/gpu.rs` (`Cb { frame: None }` when `last_key == Some(key) && stale_hits == 0`) so static viewports reuse the GPU `Output` texture without recompiling.
  - Added pre-resolve frustum & `< 0.75px` sub-pixel culling in `crates/designos-gpu/src/compile.rs` before `board::get` and `resolve_node`.
  - Added `< 3.5px` interactive text greeking (`1.25px..3.5px` translucent bar, `< 1.25px` culled), hardware scissor clip fallback (`max_r_px <= 2.5` or `progs.len() >= 32`), and a `< 64` offscreen GPU `Program` cap gated behind `self.interactive()`.
  - Upgraded `GeomCache` (`crates/designos-gpu/src/geom.rs`) with cumulative per-frame `frame_build_nanos` budgeting (8ms budget) and progressive multi-frame refinement via `stale_hits`.
  - Verified live with `VSF - PCP` (`--open-id 18de678a186512c80000`, 723,136 nodes): **`0.0%` idle CPU**, ~`606 MB` RSS, zero crashes, and instant pan/zoom/hover.

---

## 2. Key Modified & Owned Subsystems

| Area | Files | Summary of Changes |
|---|---|---|
| **Compact Serialization** | `crates/designos-doc/src/lib.rs`, `crates/designos-doc/src/node.rs` | `serde_images` `"hex:..."` encoder + dual hex/array decoder; `skip_serializing_if` on `Node`, `ComponentLink`, `TextProps`, `FrameProps`; single-line `serde_json::to_string`. |
| **Fast Library Probe & Import** | `crates/designos-library/src/store.rs`, `crates/designos-app/src/home/session.rs` | 8 KB header probe in `Library::file_kind`; single-pass `Library::import_file_with_doc`; zero-allocation `HashSerializer` in `doc_hash`. |
| **GPU Compiler & Tessellation Budget** | `crates/designos-gpu/src/compile.rs`, `crates/designos-gpu/src/geom.rs`, `crates/designos-app/src/canvas/gpu.rs` | `FrameKey` frame reuse; `interactive()` guard; pre-resolve frustum/sub-pixel culling; `< 3.5px` text greeking; hardware scissor fallback; `< 64` offscreen `Program` cap; `frame_build_nanos` cumulative 8ms tessellation budget. |
| **Hit-Testing, Layers & Fonts** | `crates/designos-app/src/canvas/hit.rs`, `crates/designos-app/src/canvas/interact.rs`, `crates/designos-app/src/panels/layers.rs`, `crates/designos-fonts/src/ensure.rs` | `SceneIndex`-culled `hit_test_with_index` / `deep_hit_test_with_index`; cached `LayersState::rows_cache` with borrowed `&Node` (`RowInfo`); zero-clone `ensure_font_requests_with`; removed `app.changed()` from view-only pan/zoom/fit. |
| **Brand Icon & Theme** | `crates/designos-app/src/icons.rs`, `crates/designos-app/src/theme.rs`, `assets/icon/**` | Spherical orange DESIGN:OS Studio icon; fixed Frame-1 `ctx.data_mut` + `ctx.load_texture` `RwLock` self-deadlock in `paint_logo`; EaseUI BrainWave 2.0 dark/light theme. |

| **Phase 2 Engine Architecture** | `crates/designos-doc/src/lib.rs`, `crates/designos-render/src/index.rs`, `crates/designos-gpu/src/geom.rs`, `crates/designos-app/src/canvas/hit.rs` | `Document::prune_unreachable` & `page_node_count`; `SceneIndex` uniform spatial grid (`GRID_CELL = 2048.0`, `query_top_level`) & incremental `update_subtree`; `GeomCache` `IdHasher`, `canonical_kind`, and `shared` canonical geometry deduplication pool (`shared_hits`). |
| **Figma Feature Parity Roadmap** | `plans/specs/figma-feature-parity/roadmap.md`, `plans/specs/figma-engine-phase2/plan.md` | Full gap analysis comparing Figma Desktop vs DESIGN:OS Studio across 7 product pillars + prioritized Phase 3–6 implementation waves. |

---

## 3. Verification & Live Runtime Metrics

- **Unit & Integration Tests:**
  - `cargo test --workspace --lib` -> **PASS** (all workspace library tests green, including Phase 2 `prune_unreachable`, `page_node_count`, `query_top_level`, `canonical_kind_deduplicates_frames_components_and_instances`, and `shared_pool_deduplicates_repeated_text_and_vectors`)
  - `cargo test -p designos-gpu` -> **PASS** (adapter + bit-exact CPU-vs-GPU fidelity tests green)
  - `cargo test -p designos-mcp` -> **PASS** (MCP server session & stdio tests green)
- **Live Benchmark (`VSF - PCP`, `--open-id 18de678a186512c80000`, 723,136 Kiwi records / 112,474 nodes):**
  - On-disk library size: `6.8 GB` -> **`242 MB`** (`-96.5%`, `28.3x` smaller)
  - Cold-start + load + GPU 1440×900 screenshot (`IAM · Audit log` at 60% zoom): **`1.54s`**
  - Idle CPU usage with 112k-node canvas open: **`0.0%–0.3%` CPU**
  - Packaged macOS app bundle: `target/release/bundle/DESIGN-OS Studio.app`

---

## 4. Next Steps (Phase 3 Roadmap — See `plans/specs/figma-feature-parity/roadmap.md`)

1. **Auto-Layout 5.0 Completeness (`designos-layout` + `designos-doc`):** Add `Wrap` (`flex-wrap`) with `counter_axis_spacing`, `min_width` / `max_width` / `min_height` / `max_height` clamping, and `FirstBaseline` alignment.
2. **Scoped Variable Binding UI & Token Studio (`designos-app` + `designos-doc`):** Add 1-click Variable Binding Picker popover on Spacing, Padding, Corner Radius, Width/Height, and Typography fields + W3C Design Tokens JSON import/export.
3. **Interactive Prototype Player (`designos-app` + `designos-doc`):** Add `interactions: Vec<Interaction>` (On Click / Hover / Navigate To / Open Overlay / Smart Animate) and an in-app Prototype Present modal (`Cmd+Enter`).
4. **Instanced Analytic SDF Shader & GPU Glyph Atlas (`designos-gpu`):** Render standard rounded rects/shadows via analytic SDF shader and rasterize text glyphs above `3.5px` into a shelf-packed GPU texture atlas.
