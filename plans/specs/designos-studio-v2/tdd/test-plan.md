---
id: test-plan
type: tdd
title: "Stage 9 TDD & Automated Scenario Verification Plan (SCN-01-01..SCN-09-03)"
status: Passed
updated: 2026-10-15
owner_role: QC
trace:
  us: [US-01, US-02, US-03, US-04, US-05, US-06, US-07, US-08, US-09]
---

Tài liệu này ánh xạ toàn bộ **27 kịch bản nghiệm thu (`SCN-01-01` .. `SCN-09-03`)** của 9 User Stories (`US-01..US-09`) sang các bài kiểm thử tự động (`cargo test`) và các lệnh chụp ảnh xác minh ngầm (`--screenshot` với `with_active(false)`).

---

## 1. Ma trận Kịch bản Kiểm thử Tự động (`SCN-01-01` .. `SCN-09-03`)

| Scenario ID | User Story & AC | Screen(s) | Method | Status | Automated Test Target / CLI Verification Command |
|---|---|---|---|---|---|
| **SCN-01-01** | `US-01` · `AC-01` | `S01` | `auto` | `passed` | `cargo test -p designos-fig --test fig_roundtrip` (`raw_fields` & Kiwi v106 lossless R/W) |
| **SCN-01-02** | `US-01` · `AC-02` | `S01` | `auto` | `passed` | `cargo test -p designos-app layout::tests` (`LayoutDirection::Grid`, `fr`/`px`/`auto`, `col_span`/`row_span`) |
| **SCN-01-03** | `US-01` · `AC-03` | `S01`, `S02` | `auto` | `passed` | `cargo test -p designos-app specs::tests::summarizes_nodespec_json_metrics` |
| **SCN-02-01** | `US-02` · `AC-01` | `S02` | `auto` | `passed` | `cargo test -p designos-app specs::tests::loads_workspace_spec_packages_and_json_blueprints` |
| **SCN-02-02** | `US-02` · `AC-02` | `S02`, `S03` | `auto` | `passed` | `cargo test -p designos-app specs::tests::formats_copy_cho_ai_citation_markdown` |
| **SCN-02-03** | `US-02` · `AC-03` | `S02` | `auto` | `passed` | `cargo test -p designos-app specs::tests::parses_us_01_frontmatter_and_ac_table_and_ui_contract_and_gates` |
| **SCN-03-01** | `US-03` · `AC-01` | `S03` | `auto` | `passed` | `cargo test -p designos-app terminal::tests` (PTY spawn, ANSI VT100 screen buffer, 120fps non-blocking) |
| **SCN-03-02** | `US-03` · `AC-02` | `S02`, `S03` | `auto` | `passed` | `cargo test -p designos-app terminal::tests` (`last_ai_copy` citation injection into PTY stdin) |
| **SCN-03-03** | `US-03` · `AC-03` | `S01`, `S03` | `auto` | `passed` | `cargo test -p designos-app terminal::tests` (live stdout `NodeSpec` extraction & canvas sync) |
| **SCN-04-01** | `US-04` · `AC-01` | `S04` | `auto` | `passed` | `cargo test -p designos-app home::` (`Drafts`, `All folders`, `Resources`, `Starred`, `Trash` navigation & badges) |
| **SCN-04-02** | `US-04` · `AC-02` | `S04` | `auto` | `passed` | `cargo test -p designos-app home::` (search, type filter, sort order, Grid/List toggle) |
| **SCN-04-03** | `US-04` · `AC-03` | `S04` | `auto` | `passed` | `cargo test -p designos-app shell::tests` (file card context menu: Star, Duplicate, Rename, Trash/Restore) |
| **SCN-05-01** | `US-05` · `AC-01` | `S05` | `auto` | `passed` | `cargo test -p designos-app panels::properties::tests` (`Type settings` 3-tab `Basic \| Details \| Variable` & OpenType) |
| **SCN-05-02** | `US-05` · `AC-02` | `S05`, `S06` | `auto` | `passed` | `cargo test -p designos-app cmdk::tests` & `panels::menus::tests` (Bottom `Cmd+K` & `Zoom/View` `%` input) |
| **SCN-05-03** | `US-05` · `AC-03` | `S07`, `S08` | `auto` | `passed` | `cargo test -p designos-app variables::` & `dev::tests` (`matches_scope` picker & Dev Mode Box Model + 5-lang codegen) |
| **SCN-06-01** | `US-06` · `AC-01` | `S01` | `auto` | `passed` | `cargo test -p designos-app canvas::hit::tests` & `canvas::interact::tests` (17 `NodeKind` hover/select & double-click drill-down) |
| **SCN-06-02** | `US-06` · `AC-02` | `S01` | `auto` | `passed` | `cargo test -p designos-app canvas::handles::tests` & `canvas::snap::tests` (resize, corner radius, pink padding/gap handles, `Alt` redlines) |
| **SCN-06-03** | `US-06` · `AC-03` | `S01` | `auto` | `passed` | `cargo test -p designos-app canvas::grids::tests` & `layout::tests` (dotted 2D CSS Grid cell overlay & track header pills) |
| **SCN-07-01** | `US-07` · `AC-01` | `S01`, `S07` | `auto` | `passed` | `cargo test -p designos-fig` (`fig-kiwi` v106 schema, 653 definitions, `parameterConsumptionMap` id=445) |
| **SCN-07-02** | `US-07` · `AC-02` | `S01` | `auto` | `passed` | `cargo test -p designos-fig --test fig_roundtrip` (`symbolOverrides`, stale binding clear, `overriddenSymbolID` swap) |
| **SCN-07-03** | `US-07` · `AC-03` | `S01`, `S07` | `auto` | `passed` | `cargo test -p designos-app icons::tests` & UI3 token verification in `plans/specs/designos-studio-v2/research/` |
| **SCN-08-01** | `US-08` · `AC-01` | `S01..S09` | `auto` | `passed` | `cargo test -p designos-app cli::tests` (`with_visible(false).with_active(false)` background screenshot capture) |
| **SCN-08-02** | `US-08` · `AC-02` | `S01..S09` | `auto` | `passed` | `cargo test -p designos-app specs::tests::renders_specs_reader_and_left_panel_without_lint_issues` |
| **SCN-08-03** | `US-08` · `AC-03` | `S01..S09` | `auto` | `passed` | `cargo test --workspace` (full workspace regression + 99% Figma UI3 visual parity verification) |
| **SCN-09-01** | `US-09` · `AC-01` | `S09`, `S04` | `auto` | `passed` | `cargo test -p designos-app updater::tests` (`Stable \| Beta \| Nightly` channel manifest check & UI3 Update Pill) |
| **SCN-09-02** | `US-09` · `AC-02` | `S09` | `auto` | `passed` | `cargo test -p designos-app updater::tests` (SHA-256 verification, `.staged` → `.bak` atomic swap & automatic rollback) |
| **SCN-09-03** | `US-09` · `AC-03` | `S09`, `S01` | `auto` | `passed` | `cargo test -p designos-app updater::tests` (passive read-only `/Applications/Figma.app` `Info.plist` & Kiwi `v106` watchdog) |
| **SCN-10-01** | `US-10` · `AC-01` | `S10`, `S04` | `auto` | `passed` | `cargo test -p designos-app tabs::tests` (`36px` Desktop Tab Strip, **`design`:os studio** lowercase bold-prefix wordmark, `Home` tab + project tabs) |
| **SCN-10-02** | `US-10` · `AC-02` | `S10`, `S01` | `auto` | `passed` | `cargo test -p designos-app tabs::tests` (100% per-tab isolation of `Document`, `History` undo/redo, `Page`, `Selection`, `View` zoom/pan & `Dev Mode`) |
| **SCN-10-03** | `US-10` · `AC-03` | `S10` | `auto` | `passed` | `cargo test -p designos-app tabs::tests` (`Cmd+T`, `Cmd+W`, `Ctrl+Tab`, `Cmd+1..9` & context menu `Duplicate tab`, `Pin/Unpin tab`, `Close other tabs`) |

---

## 2. Quy trình TDD & Kiểm chứng Nền Không Xâm lấn (`with_active(false)`)

1. **Red → Green → Refactor**: Mọi hành vi mới trong `ui-contract.md` phải được khai báo `SCN-xx-yy` trước khi viết mã giao diện và kiểm chứng bằng `cargo test`.
2. **Chụp ảnh bằng chứng ngầm & Kiểm tra Cập nhật (Zero Focus Stealing)**:
   ```bash
   cargo run --release -p designos-app -- "/Users/jangtrinh/Downloads/VSF - PCP.fig" --screenshot /tmp/designos_s01_canvas.png
   cargo run --release -p designos-app -- --panel specs --screenshot /tmp/designos_s02_specs.png
   cargo run --release -p designos-app -- --panel terminal --screenshot /tmp/designos_s03_terminal.png
   cargo run --release -p designos-app -- update --check
   ```
   Tất cả lệnh trên thực thi với `ViewportBuilder::default().with_visible(false).with_active(false)` để không bao giờ hiện cửa sổ đè lên màn hình người dùng.
