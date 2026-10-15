# Stage 11: Developer Handoff & Readiness Packet (`READY.md`)

> **Feature:** `designos-studio-v2` — **`design`:os studio** Native Rust GPU Design Studio, Multi-Tab Projects, 11-Stage Spec-Driven OS & Dual-Layer Auto-Update  
> **Stage:** `11 / handoff`  
> **Gate:** `g5-handoff` (`GATE: APPROVED`)  
> **Updated:** `2026-10-15T09:05:00Z`

---

## 1. End-to-End Traceability Matrix (`REQ` -> `US` -> `Screen` -> `Decision` -> `BR` -> `SCN` -> `Code / Test`)

| REQ ID | User Story | Screen(s) | Decision | Business Rule | Test Scenarios | Primary Rust Module & Test Target |
|---|---|---|---|---|---|---|
| `REQ-01` | `US-01`, `US-06` | `S01` | `D1` | `BR-01` | `SCN-01-01..02` | `crates/designos-fig/` (`.fig` Kiwi v106 lossless R/W) + `crates/designos-app/src/layout.rs` (2D CSS Grid) |
| `REQ-02` | `US-01` | `S01`, `S02`, `S03` | `D1`, `D5` | `BR-01`, `BR-05` | `SCN-01-03` | `crates/designos-app/src/specs.rs` (`apply_node_spec`, `.spec.json` <-> Canvas bridge) |
| `REQ-03` | `US-02` | `S02` | `D2`, `D3` | `BR-02`, `BR-03` | `SCN-02-01` | `crates/designos-app/src/specs.rs` (11-stage Spec Format v2 loader & KPI Strip) |
| `REQ-04` | `US-02` | `S02` | `D2`, `D4` | `BR-02`, `BR-04` | `SCN-02-02..03` | `crates/designos-app/src/specs.rs` (5 Role Views, `format_copy_for_ai`, `approve_gate`) |
| `REQ-05` | `US-03` | `S03` | `D4`, `D5` | `BR-04`, `BR-05` | `SCN-03-01..03` | `crates/designos-app/src/terminal.rs` (`portable-pty` + `vt100` multi-tab agent terminal) |
| `REQ-06` | `US-04` | `S04` | `D6` | `BR-06` | `SCN-04-01..03` | `crates/designos-app/src/home/mod.rs` (`Drafts`, `All folders`, `Resources`, `Starred`, `Trash`) |
| `REQ-07` | `US-05` | `S05`, `S06`, `S07`, `S08` | `D6`, `D10` | `BR-07`, `BR-08`, `BR-14` | `SCN-05-01..03` | `crates/designos-app/src/panels/menus.rs`, `cmdk.rs`, `dev/mod.rs`, `variables/mod.rs` + **`design`:os studio** wordmark |
| `REQ-08` | `US-06` | `S01` | `D1`, `D6` | `BR-09` | `SCN-06-01..03` | `crates/designos-app/src/canvas/` (`interact.rs`, `handles.rs`, `grids.rs`, `snap.rs` across 17 `NodeKind`s) |
| `REQ-09` | `US-07` | `S01`, `S07` | `D7` | `BR-10` | `SCN-07-01..03` | `plans/specs/designos-studio-v2/research/` + `crates/designos-fig/tests/fig_roundtrip.rs` |
| `REQ-10` | `US-08` | `S01..S10` | `D8` | `BR-11`, `BR-12` | `SCN-08-01..03` | `crates/designos-app/src/cli.rs` & `main.rs` (`with_visible(false).with_active(false)` background capture) |
| `REQ-11` | `US-09` | `S09`, `S04`, `S01` | `D9` | `BR-13` | `SCN-09-01..03` | `crates/designos-app/src/updater.rs` (`Dual-Layer Auto-Update` + `FigmaSchemaWatchdog`) |
| `REQ-12` | `US-10` | `S10`, `S04`, `S01` | `D10`, `D11` | `BR-14`, `BR-15` | `SCN-10-01..03` | `crates/designos-app/src/tabs.rs` / `app.rs` (`36px` Desktop Tab Strip, per-tab state isolation, **`design`:os studio** wordmark) |

---

## 2. Pre-Release & Verification Commands

```bash
# 1. Verify full workspace compilation and unit/integration tests
PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace

# 2. Verify 60 UI contract rows (S01..S10 x 6 states)
grep -E '^\| `?S(0[1-9]|10)`? \|' plans/specs/designos-studio-v2/ux/ui-contract.md | wc -l
```

---

## 3. Handoff Sign-Off Checklist
- [x] All 13 directives (`DIR-01..DIR-13`) traced to `EP-01..EP-10`, `REQ-01..REQ-12`, and `US-01..US-10`.
- [x] Brand typography rule (`DIR-12` / `BR-14` / `D10`): always all-lowercase **`design`:os studio** with bold **`design`** (`700`) and regular `:os studio` (`400`).
- [x] Multiple Tab Project Workspace (`DIR-13` / `EP-10` / `REQ-12` / `US-10` / `S10` / `BR-15` / `D11`): `36px` Desktop Tab Strip with 100% per-tab state isolation.
- [x] Contract `FC-001.md` Locked (`BR-01..BR-15`, Amendments `A-01..A-05`).
- [x] Decision Log `D1..D11` recorded with zero blocking `OPEN` items.
- [x] Screen Inventory (`S01..S10`) and UI Contract (`60` rows = `10 screens × 6 states`) complete.
- [x] Test Plan (`30` scenarios `SCN-01-01..SCN-10-03`) mapped 1:1 to acceptance criteria.
- [x] All 5 Quality Gates (`g1-intake` .. `g5-handoff`) audited with SHA-256 artifact digests.
