---
id: design-tokens
type: design
title: "Stage 8 Design System & Token Mapping — Figma UI3, product:os Operate, EaseUI & Unified Phosphor Icons"
status: Verified
updated: 2026-10-15
owner_role: Developer
design_system: "figma-ui3-dark + pcp-dark-operate-v2 + phosphor-icons-core"
trace:
  decisions: [D2, D6, D9, D10, D14, D15, D17]
  us: [US-02, US-04, US-05, US-06, US-08, US-09, US-10, US-11, US-12, US-13, US-14]
---

Tài liệu này quy định bảng ánh xạ Design Tokens và Hệ thống Biểu tượng Phosphor Icons chính thức giữa 3 hệ thống bề mặt trong **`design`:os studio**:
1. **Figma UI3 Dark Mode (`com.figma.Desktop` `126.9.13`)**: Áp dụng cho toàn bộ trình soạn thảo Canvas (`S01`), Home File Manager (`S04`), Type Settings & Zoom Menu (`S05`), Bottom Actions `Cmd+K` (`S06`), Dev Mode (`S07`), Color/Variables Picker (`S08`), Auto-Update & Figma Schema Watchdog (`S09`), Multi-Tab Project Strip (`S10`), Figma AI Agent Hub (`S11`) và `Agents` Left Side Panel (`S13`).
2. **`product-os-spec-driven` Operate Dark (`pcp-dark-operate-v2`)**: Áp dụng riêng cho bề mặt **Spec Driven v2 Reader (`S02`)** và **5-Pillar Figma Parity Scorecard (`S12`)** theo đúng quy tắc tách theme bộ đọc của `SPEC.md §10` và quyết định `D2`, `D14`.
3. **Unified Phosphor Icons System (`https://phosphoricons.com/` · `@phosphor-icons/core` · `DIR-19` · `D17`)**: Áp dụng thống nhất trên toàn bộ 13 màn hình `S01..S13` thông qua `crates/designos-app/src/icons.rs` và `crates/designos-app/assets/icons/phosphor/*.svg`.

---

## 1. Bảng Token Màu sắc Chuẩn (Canonical Color Token Ladder)

| Token Name | Figma UI3 Canonical (sRGB) | Figma UI3 Display-P3 Sampled | `product:os` Reader (`S02`/`S12`) | EaseUI `.spec.json` Blueprint | Vai trò hiển thị (Semantic Usage) |
|---|---|---|---|---|---|
| `bg-canvas` | **`#1E1E1E`** (`rgb(30,30,30)`) | `#1C1C1C` | **`#16171B`** | `#09090B` / `#121215` | Nền trang Canvas mặc định (`S01`) và nền chính của Spec Reader (`S02`/`S12`). |
| `bg-panel` | **`#2C2C2C`** (`rgb(44,44,44)`) | `#272727` | **`#1F2024`** | `#18181B` | Nền Left Rail (`48px`), Left Sidebar (`240px`), `Agents` Left Side Panel (`S13`), Right Inspector (`240px`), Bottom Toolbar (`528×48px`), Auto-Update Modal (`S09`) và Card cấp 1 trong Reader (`S02`/`S12`). |
| `bg-input` / `bg-elevated` | **`#383838`** (`rgb(56,56,56)`) | `#313131` / `#3C3C3C` | **`#26272C`** | `#27272A` | Nền ô nhập (`X`, `Y`, `W`, `H`, `100%`), thanh chọn phân đoạn (`Flow`, `Channel`), hàng trang đang chọn và Header bảng trong Reader (`S02`/`S12`). |
| `border-default` | **`#444444`** / **`#3C3C3C`** | `#3C3C3C` | **`#2E2F36`** | `#27272A` | Đường kẻ phân cách `1px` giữa các bảng, viền popover và đường chia bảng dữ liệu. |
| `text-primary` | **`#FFFFFF`** | `#FFFFFF` | **`#F3F4F6`** | `#FAFAFA` | Chữ tiêu đề, giá trị ô nhập, tên trang và nội dung chính. |
| `text-secondary` | **`#B3B3B3`** / **`#999999`** | `#999999` | **`#9CA3AF`** | `#A1A1AA` | Nhãn thuộc tính (`Position`, `Layout`), phụ đề `Team project` và breadcrumb. |
| `accent-primary` | **`#0D99FF`** | `#0F81E6` / `#71BCF7` | **`#3B82F6`** | **`#FF6A3D`** (BrainWave Orange) | Nút công cụ đang chọn trên Bottom Toolbar (`Move`, `Frame`...), viền hộp nhập `"Ask for changes"` trong `Agents` (`S13`), nút `Share`, thanh tiến trình tải cập nhật (`S09`), viền chọn node trên Canvas. |
| `accent-selection-row` | **`#323B55`** (`#0D99FF` @ 22%) | `#2D3340` | **`#1E293B`** | `#27272A` | Nền hàng Layer đang chọn và nút icon đang kích hoạt trên thanh Left Rail `48px`. |
| `accent-component` | **`#9747FF`** / **`#C58FFF`** | `#9C80CB` / `#352B47` | **`#8B5CF6`** | `#8B5CF6` | Màu biểu tượng và chữ của `Component (❖)`, `ComponentSet` và `Instance (◇)` trong cây Layers và trên Canvas. |
| `accent-dev-mode` | **`#24CB71`** | `#5FA773` | **`#10B981`** | `#10B981` | Huy hiệu `</>` Ready-for-Dev, huy hiệu `SHA-256 Verified ✓` / `Update Ready` (`S09`), thanh Parity `>=99%` (`S12`) và trạng thái Gate `APPROVED`. |
| `accent-warning` | **`#F24822`** / **`#FFCD29`** | `#E5484D` / `#F5A623` | **`#F59E0B`** / **`#EF4444`** | `#EF4444` | Đường gióng khoảng cách `Alt`, chốt hồng `padding`/`gap` (`#FF4081`), cảnh báo lệch schema / `SHA-256` (`S09`) và Gate `PENDING` / `MISSING`. |

---

## 2. Thông số Hình học & Tỷ lệ Khung Chuẩn Figma UI3 (Macro Layout Geometry)

| Vùng giao diện (Surface Region) | Kích thước chuẩn (1x Logical Pixels) | Bo góc (`Radius`) | Khoảng cách / Viền (`Border & Padding`) | Nguồn kiểm chứng |
|---|---|---|---|---|
| **Far-Left Navigation Rail** | Rộng **`48px`** (`x = 0..48px`), cao toàn cửa sổ | Nút icon `28×28px`, bo góc **`6px`** | Thứ tự chuẩn: `File`, `Agents` (`✦`), `Assets`, `Variables`, `Tools`, `Specs` + nút Sync chấm `#0D99FF` ở đáy (`DIR-17`, `DIR-18`) | `figma_live_reference.png`, `US-13` |
| **Left Side Panel (`File` / `Agents` / `Assets`)** | Rộng **`240px–320px`** (`x = 49..289px`) | Hàng trang `28px` cao, bo góc **`6px`** | `File` hiện `<Project> / <File>` + `Pages` + `Layers`; `Agents` (`S13`) và `Assets` chiếm trọn chiều cao từ đỉnh panel (`UF-02`) | `US-05`, `US-13` |
| **Canvas Rulers (Top & Left)** | Dày **`20px`** (`#2C2C2C`) | `0px` | Tô dải xanh `#1E3A5F` kèm chữ `#0D99FF` chiếu theo khung bao của phần tử đang chọn | `figma_live_editor.png`, `US-06`, `US-11` |
| **Right Design / Prototype / Dev Inspector** | Rộng **`240px`** (`268px` khi cuộn/mở rộng) | Ô nhập `#383838` cao **`28px`**, bo góc **`6px`** | Dòng 1 (`38px`): Avatar + `Present ▷ v` + nút `Share` (`#0D99FF`); góc dưới phải có nút trợ giúp nổi `?` (`DIR-18`) | `US-05`, `US-13` |
| **Bottom Floating Toolbar & Sub-Toolbar** | **`528px × 48px`** (cách đáy `12px`) | Bo góc ngoài **`12px`**, nút trong **`6px`** | Công cụ đang chọn tô xanh `#0D99FF` + cụm `Design \| Prototype \| </>` + thanh phụ ngữ cảnh phía trên (`Crop \| Select area \| Remove bg \| Edit with prompt \| More v`) | `US-05`, `US-13` |
| **Bottom-Docked `Actions (Cmd+K)`** | **`480px × 320px`** (neo cách đáy `68px`) | Bo góc **`12px`** | Nền `#2C2C2C`, viền `1px` `#444444`, 4 tab chữ gạch chân `All \| Assets \| Plugins & widgets \| Design Gate` + nhãn `[AI]` / `[Development]` (`DIR-18`) | `US-05`, `US-11`, `US-13` |
| **Auto-Update & Schema Watchdog (`S09`)** | **`440px × 340px`** modal + **`208×28px`** Home pill | Bo góc modal **`12px`**, pill **`6px`** | Nền `#2C2C2C`, viền `1px` `#444444`, thanh chọn kênh 3 phân đoạn `Stable \| Beta \| Nightly` (`#383838`) | `US-09`, `S09` |
| **Desktop Multi-Tab Project Strip (`S10`)** | Cao **`36px`** toàn chiều ngang đỉnh cửa sổ | Bo góc tab **`6px`** (active `#2C2C2C`, inactive `#1E1E1E`) | Nút `Home` (`36×28px`) + cụm thương hiệu **`design`:os studio** + dải tab dự án + nút `+` (`28×28px`) | `US-10`, `S10` |

---

## 3. Hệ thống Biểu tượng Phosphor Icons Đồng nhất (`https://phosphoricons.com/` · `DIR-19` · `BR-22` · `D17`)

| Thuộc tính | Quy chuẩn kỹ thuật trong **`design`:os studio** | Nguồn kiểm chứng |
|---|---|---|
| **Nguồn tài nguyên chuẩn** | Bộ biểu tượng chính thức **`@phosphor-icons/core` (`https://phosphoricons.com/`)** với `viewBox="0 0 256 256"` và `fill="currentColor"`. | `DIR-19`, `US-14` |
| **Đường dẫn đóng gói** | `crates/designos-app/assets/icons/phosphor/*.svg` được nhúng tĩnh vào binary qua `include_str!` trong `crates/designos-app/src/icons.rs`. | `crates/designos-app/src/icons.rs` |
| **Động cơ Raster & Cache** | Phân tích bằng `resvg::usvg::Tree::from_str`, rasterize theo kích thước điểm ảnh vật lý `px = (size * pixels_per_point).round()` vào `SVG_CACHE: Mutex<HashMap<(Icon, u32), TextureHandle>>` với `TextureOptions::LINEAR`. | `SCN-14-01`, `SCN-14-02` |
| **Căn lưới điểm ảnh (`DPR Snap`)** | Mọi hình chữ nhật vẽ icon đều đi qua `round_to_pixel(pos, pixels_per_point)` (`step = 1.0 / ppp`) để không bị nhòe nửa điểm ảnh trên màn hình Retina/HiDPI. | `SCN-14-02` |
| **Ngoại lệ thương hiệu duy nhất** | Chỉ duy nhất `Icon::Logo` giữ nguyên hình khối thương hiệu **`design`:os studio**; toàn bộ các biến thể `Icon` còn lại trên `S01..S13` bắt buộc dùng Phosphor Icons. | `BR-22`, `SCN-14-03` |
| **Pure PTY Chat UI Wrapper (`S13` · `DIR-21`)** | Pill chọn CLI (`Claude Code \| Codex \| Antigravity`): active `#0D99FF`28 viền `#0D99FF`, inactive `#383838`; bong bóng `User` (`AgentChatRole::User`) nền `#26354A` bo góc `8px`; bong bóng `Assistant` (`AgentChatRole::Assistant`) nền `#232323` viền `#383838` bo góc `8px`; huy hiệu `NodeSpec` nội tuyến nền `#1E3A2F` chữ `#24CB71`. | `BR-23`, `SCN-15-01..03` |

---

## 4. Luật Cấm & Quy chuẩn Thương hiệu trong Thiết kế (`es:designer` Craft Floor & Brand Rules)

1. **Zero Emoji Icons & Zero Mixed Icon Families (`BR-02`, `BR-22`, `DIR-19`)**: Cấm tuyệt đối dùng ký tự emoji Unicode, cấm pha trộn nhiều họ icon khác nhau (Lucide/Feather/custom paths) và cấm vẽ biểu tượng UI chrome thủ công rời rạc bên ngoài `crate::icons::Icon` (Phosphor Icons `@phosphor-icons/core`).
2. **No Cards-in-Cards**: Trong Spec Reader (`S02`/`S12`), Inspector (`S05..S08`) và Auto-Update Modal (`S09`), cấm lồng một thẻ có viền/nền bên trong một thẻ khác; phân tách nhóm con bằng đường kẻ mảnh `1px` (`#2E2F36` / `#3C3C3C`) hoặc dòng tiêu đề bảng `#26272C`.
3. **Tabular Numerals & Safe Truncation**: Mọi ô số liệu (`X`, `Y`, `W`, `H`, `%`, `SHA-256` prefix, KPI Strip, Parity Scorecard `S12`, bảng `ui-contract.md`) phải căn thẳng hàng số và cắt gọn chữ an toàn (`elide`) không tràn khỏi khung.
4. **All-Lowercase Bold-Prefix Brand Wordmark (`BR-14` / `D10`)**: Tên thương hiệu sản phẩm luôn luôn viết thường toàn bộ là **`design`:os studio** (hoặc **`design`:os**), trong đó chữ **`design`** bắt buộc dùng font **Bold (`700`)** và `:os studio` dùng font **Regular (`400`)**.
5. **Zero Raw ANSI Escape Noise in Agent Chat Bubbles (`BR-23` / `D18` / `DIR-21`)**: Luồng stdout từ PTY `claude`, `codex`, `agy` khi đồng bộ sang bong bóng `AgentChatTurn` phải luôn đi qua `strip_ansi_codes` để loại bỏ hoàn toàn các chuỗi điều khiển CSI/OSC trước khi hiển thị cho người dùng.

