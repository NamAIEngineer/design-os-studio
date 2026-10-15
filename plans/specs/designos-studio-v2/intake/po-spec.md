---
id: po-spec
type: intake
title: "PO Intake — design:os studio: 99% 1:1 Figma Parity, Multi-Tab Projects, REA, Auto-Update & Agentic Product OS"
status: Approved
updated: 2026-10-15
owner_role: Product Owner
---

## 1. Bối cảnh và Bài toán (Problem Statement)

Các nhóm thiết kế và phát triển sản phẩm AI-native đang bị phân mảnh giữa 3 bề mặt công cụ tách rời:
1. **Figma Desktop (`com.figma.Desktop` UI3)** để thiết kế giao diện vector trên nhiều tab dự án mở song song, quản lý Component/Variable và lưu trữ định dạng nhị phân `.fig` (Kiwi container v106).
2. **Spec Driven Reader (`product-os-spec-driven`)** để quản lý gói đặc tả Format v2 (`Intake`, `Clarify`, `Contract`, `PRD`, `US`, `Wireframe`, `Prototype`, `Design`, `TDD`, `UIUX`, `Handoff`), ma trận `ui-contract.md` 12 cột và các Gate kiểm duyệt `G1..G5`.
3. **CLI Agent Terminal (`claude` / `codex` / `grok` / `$SHELL`)** để thực thi tác tử lập trình AI, sinh giao diện `.spec.json` và chạy kiểm thử tự động.

**`design`:os studio** (luôn viết thường toàn bộ, in đậm chữ **`design`**, viết thường `:os studio`) hợp nhất cả 3 bề mặt trong một ứng dụng native Rust/egui/wgpu tốc độ 120fps duy nhất, đạt **99% 1:1 parity** với Figma Desktop UI3 về định dạng nhị phân `.fig` v106, thanh **Multiple Tab Project Strip (`36px`)**, giao diện chrome, menu/popover, tương tác trực tiếp trên từng phần tử Canvas, lẫn **Cơ chế Tự động Cập nhật Kép (Dual-Layer Auto-Update Architecture)**.

---

## 2. 13 Chỉ thị gốc của Product Owner (Canonical User Directives `DIR-01..DIR-13`)

| Mã | Chỉ thị của Product Owner (User Directive) | Mục tiêu định lượng | Ánh xạ Epic / US |
|---|---|---|---|
| **DIR-01** | **Reverse Engineering Figma (`morluto/rea`)**: Giải mã `/Applications/Figma.app/Contents/Resources/app.asar` và `/Users/jangtrinh/Downloads/VSF - PCP.fig` (`fig-kiwi` v106, 653 Kiwi definitions, 155,297 `NodeChange` records) để trích xuất schema nhị phân, bảng ánh xạ biến và hệ token UI3. | 100% trường `NodeChange` & `symbolOverrides` được lập tài liệu | `EP-01` → `US-07`, `US-01` |
| **DIR-02** | **99% 1:1 `.fig` Binary Import & Lossless Roundtrip**: Đọc/ghi trực tiếp `.fig` v106 bảo toàn 100% `raw_fields`, `parameterConsumptionMap` (id=445), `variableConsumptionMap`, `symbolOverrides`, `overriddenSymbolID`, `fontVariations`, `toggledOnOTFeatures` và `handoffStatusMap`. | 0 byte mất mát trên các trường mở rộng khi round-trip | `EP-02` → `US-01` |
| **DIR-03** | **Auto-Layout 2D CSS Grid & `.spec.json` Bridge**: Hỗ trợ bố cục lưới 2 chiều (`LayoutDirection::Grid`, `stackMode: "GRID"`) với track sizing `fr` / `px` / `auto`, `col_span` / `row_span` và đồng bộ hai chiều với `.spec.json` (`NodeSpec`). | Giải chuẩn 100% track `fr`/`px`/`auto` & span | `EP-02`, `EP-03` → `US-01`, `US-06` |
| **DIR-04** | **1:1 Per-Element Canvas Interactions (17 Node Types)**: Tương tác trực tiếp trên toàn bộ 17 loại node (`Frame`, `Section`, `Group`, `Component`, `ComponentSet`, `Instance`, `Rectangle`, `Ellipse`, `Polygon`, `Star`, `Line`, `Vector`, `BooleanOp`, `Text`, `Slice`, `Sticky`, `Connector`) với đường viền hover `#0D99FF` / `#9747FF`, 8 chốt kéo, chốt bo góc, chốt padding/gap màu hồng, khoan sâu double-click và lưới ô chấm 2D Grid. | Đủ 17 loại node & toàn bộ chốt điều khiển trực quan | `EP-03` → `US-06` |
| **DIR-05** | **1:1 Figma UI3 Dark Chrome & Inspector**: Áp dụng đúng bảng màu Figma UI3 (`#1E1E1E` canvas, `#2C2C2C` panel, `#383838` input pill, `#444444` border, `#0D99FF` accent blue, `#9747FF` component purple, `#24CB71` Dev Mode green), thanh rail trái `48px` chỉ có icon, thanh bên trái `240px` (`VSF - PCP` + `Team project`, `Pages`, `Layers`), thanh công cụ nổi dưới đáy `528×48px` và Inspector phải `240px` có nút `Share` xanh. | 99% độ khớp thị giác so với ảnh chụp Figma Desktop | `EP-04` → `US-05` |
| **DIR-06** | **1:1 Figma UI3 Settings, Menus & Popovers**: Hoàn thiện hộp thoại `Type settings` 3 tab (`Basic \| Details \| Variable`), thanh `Actions (Cmd+K)` neo dưới đáy (`All \| AI \| Actions \| Libraries`), menu `Zoom/View` có ô nhập `%`, hộp chọn màu `Color Picker` (`HEX/RGB/CSS/HSL/HSB`), `Scoped Variables Picker` và `Dev Mode (</>)` Box Model + sinh mã 5 ngôn ngữ. | 100% menu/popover hoạt động thật trên dữ liệu tài liệu | `EP-04` → `US-05` |
| **DIR-07** | **1:1 Figma UI3 Home & File Manager**: Màn hình quản lý file (`Drafts`, `All folders`, `Resources / Templates`, `Starred`, `Trash`), tạo file mới (`Design`, `FigJam`, `Slides`, `Import .fig`), tìm kiếm, lọc theo loại file, sắp xếp, chuyển đổi Grid/List, huy hiệu loại file và menu ngữ cảnh chuột phải. | Đầy đủ 5 mục điều hướng Home & thao tác CRUD file | `EP-05` → `US-04` |
| **DIR-08** | **Native Embedded PTY Agent Terminal**: Tích hợp terminal đa tab thực thi trực tiếp `claude`, `codex`, `grok`, `$SHELL` qua `portable-pty` + `vt100` TrueColor, tự động tiêm biến môi trường `DESIGNOS_*`, nhận trích dẫn `Copy cho AI` và stream kết quả `NodeSpec` lên Canvas ở 120fps. | Khởi chạy PTY không chặn luồng UI 120fps | `EP-06` → `US-03` |
| **DIR-09** | **Native `product-os-spec-driven` Format v2 Reader**: Đọc trực tiếp gói Spec Format v2 đủ 11 stage theo 5 Role Views (`PO View`, `Dev View`, `QA & Coverage`, `Wireframes & UI Contract`, `Gates & Canvas`), thanh 4-Metric KPI Strip đồng nhất, Attention Line và nút `Copy cho AI` theo chuẩn `es:designer` Operate-mode (`#16171B` / `#1F2024` / `#26272C`, không emoji chrome, không lồng card trong card). | 11 stage, 5 Role Views, 0 lệch số KPI | `EP-07` → `US-02` |
| **DIR-10** | **100% Background Headless Verification (`with_active(false)`)**: Mọi lệnh chụp màn hình kiểm thử tự động (`--screenshot`, `--panel`) phải chạy hoàn toàn ở chế độ nền (`with_visible(false).with_active(false)` hoặc bộ dựng phần mềm offscreen), tuyệt đối không chiếm quyền điều khiển cửa sổ hay làm gián đoạn màn hình của người dùng. | 0 lần cướp focus cửa sổ macOS khi chạy kiểm thử | `EP-08` → `US-08` |
| **DIR-11** | **Dual-Layer Auto-Update Architecture (`updater.rs` + Figma Schema Watchdog)**: Thiết kế cơ chế tự động cập nhật 2 tầng: (1) **Tầng 1 — Native App & CLI Atomic Self-Updater** hỗ trợ 3 kênh (`Stable`, `Beta`, `Nightly`), kiểm tra/tải ngầm, xác thực băm `SHA-256`, hoán đổi nguyên tử (`.staged` → `.bak` → rename) kèm tự động rollback và giao diện UI3 Update Pill/Modal (`S09`); (2) **Tầng 2 — Upstream Figma Desktop (`app.asar` & `.fig` Kiwi Schema) Watchdog** theo dõi thụ động `/Applications/Figma.app/Contents/Info.plist` và phiên bản schema Kiwi (`v106`) để đồng bộ tức thì khi Figma cập nhật. | 100% cập nhật nguyên tử có kiểm tra SHA-256 & rollback an toàn | `EP-09` → `US-09` |
| **DIR-12** | **Quy chuẩn Thương hiệu `design`:os studio (`BR-14` / `D10`)**: Tên thương hiệu sản phẩm luôn luôn viết thường toàn bộ là **`design`:os studio** (hoặc **`design`:os**), trong đó chữ **`design`** in đậm (`Bold 700` / `egui::TextFormat` bold span) và phần `:os studio` viết thường nét thường (`Regular 400`). Nghiêm cấm viết hoa (`DESIGN:OS Studio`, `Design OS Studio`, `DESIGN:OS`) trên toàn bộ UI chrome và tiêu đề hiển thị. | 100% nhãn thương hiệu hiển thị đúng **`design`:os studio** | `EP-04`, `EP-10` → `US-05`, `US-10` |
| **DIR-13** | **Multiple Tab Project Workspace (`EP-10` / `REQ-12` / `US-10` / `S10` / `BR-15`)**: Hỗ trợ mở và làm việc song song trên nhiều tab dự án cùng lúc qua thanh **Figma UI3 Desktop Tab Strip (`36px`)** ở đỉnh cửa sổ (`Home` icon tab + danh sách tab dự án đang mở kèm huy hiệu loại file `Design`/`FigJam`/`Slides`/`Specs`, chấm báo chưa lưu `•`, trạng thái ghim `Pin`, nút đóng `×`, nút `+` tạo tab mới), cô lập hoàn toàn trạng thái từng tab (`Document`, `History` undo/redo, `Page`, `Selection`, `View` zoom/pan, `Dev Mode`, `file_path`), hỗ trợ phím tắt (`Cmd+T`, `Cmd+W`, `Ctrl+Tab` / `Cmd+Shift+]`, `Cmd+1..9`) và menu chuột phải trên tab (`Duplicate tab`, `Pin/Unpin tab`, `Close other tabs`). | Cô lập 100% state giữa các tab & chuyển tab tức thì `< 1ms` | `EP-10` → `US-10` |

---

## 3. Phân loại Dự án và Ràng buộc Kiến trúc (Classification & Constraints)

- **Phân loại quy trình (`pipeline.yaml`)**: Dự án phức hợp (`Complex`), kích hoạt toàn bộ **11 stage** của `product-os-spec-driven` Format v2 (`intake` → `clarify` → `contract` → `prd` → `us` → `wireframe` → `prototype` → `design` → `tdd` → `uiux` → `handoff`) và **5 Gate kiểm duyệt (`G1..G5`)**.
- **Ràng buộc kỹ thuật bắt buộc**:
  1. Không phụ thuộc Electron hay trình duyệt web cho nhân dựng hình; toàn bộ Canvas, UI3 Chrome, Multi-Tab Project Workspace, Spec Reader v2, PTY Terminal và Auto-Updater chạy thuần Rust (`designos-doc`, `designos-fig`, `designos-render`, `designos-gpu`, `designos-export`, `designos-app`, `designos-mcp`).
  2. Mọi lệnh chụp ảnh xác minh giao diện tự động và tác vụ kiểm tra cập nhật/watchdog phải chạy ngầm (`with_active(false)`), không hiển thị cửa sổ đè lên màn hình làm việc của người dùng.
