# DESIGN:OS Studio — Architecture & Roadmap Overview (`PLAN.md`)

**DESIGN:OS Studio** (`https://github.com/jangtrinh/design-os-studio`) is an independently owned, native Rust + `wgpu` interface design tool and Figma `.fig` engine integrated with the **DESIGN:OS / EaseUI** ecosystem.

## Active Specifications & Roadmaps

- **Phase 1 — Enterprise `.fig` Engine & EaseUI Theme (Completed):** [`plans/specs/figma-engine-opt/plan.md`](plans/specs/figma-engine-opt/plan.md)
  - `-96.5%` compact `.designos`/`.designos` storage (`6.8 GB` -> `242 MB`), `FrameKey` GPU reuse (`0.0%` idle CPU), 4-tier semantic zoom LOD, `< 3.5px` text greeking, and hardware scissor fallback.
- **Phase 2 — Spatial Grid, Incremental Index & Canonical Mesh Pool (Completed):** [`plans/specs/figma-engine-phase2/plan.md`](plans/specs/figma-engine-phase2/plan.md)
  - `2048×2048` uniform spatial grid (`SceneIndex::query_top_level`), $O(\text{subtree})$ incremental index updates (`SceneIndex::update_subtree`), `GeomCache` canonical shape deduplication (`shared_hits`), and `Document::prune_unreachable`.
- **Phase 3–6 — Figma Feature Parity Roadmap (Active):** [`plans/specs/figma-feature-parity/roadmap.md`](plans/specs/figma-feature-parity/roadmap.md)
  - Wave 1 (Phase 3): Auto-Layout 5.0 (`Wrap`, `Min/Max` constraints, `FirstBaseline`) + Scoped Variable Binding UI on Inspector + W3C Design Tokens JSON.
  - Wave 2 (Phase 4): Interactive Prototype Player (`Cmd+Enter`, Smart Animate, Overlays) + Instanced Analytic SDF Shader & GPU Glyph Atlas.
  - Wave 3 (Phase 5): True Vector Network Graph Editor + Rich Text Lists & OpenType features.
  - Wave 4 (Phase 6): Live Desktop-Connected MCP Bridge (`designos-mcp` <-> running `designos-studio` window).
