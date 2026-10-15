---
id: design-tokens
type: design
title: "Stage 8 Design System & Token Mapping — Figma UI3, product:os Operate & EaseUI"
status: Verified
updated: 2026-10-15
owner_role: Developer
design_system: "figma-ui3-dark + pcp-dark-operate-v2"
trace:
  decisions: [D2, D6, D9]
  us: [US-02, US-04, US-05, US-06, US-08, US-09]
---

Tài liệu này quy định bảng ánh xạ Design Tokens chính thức giữa 3 hệ thống bề mặt trong **DESIGN:OS Studio**:
1. **Figma UI3 Dark Mode (`com.figma.Desktop`)**: Áp dụng cho toàn bộ trình soạn thảo Canvas (`S01`), Home File Manager (`S04`), Type Settings & Zoom Menu (`S05`), Bottom Actions `Cmd+K` (`S06`), Dev Mode (`S07`), Color/Variables Picker (`S08`) và Auto-Update & Figma Schema Watchdog (`S09`).
2. **`product-os-spec-driven` Operate Dark (`pcp-dark-operate-v2`)**: Áp dụng riêng cho bề mặt **Spec Driven v2 Reader (`S02`)** theo đúng quy tắc tách theme bộ đọc của `SPEC.md §10` và quyết định `D2`.
3. **EaseUI BrainWave 2.0 Tokens**: Áp dụng cho các mẫu giao diện `.spec.json` (`NodeSpec`) được dựng trên Canvas.

---

## 1. Bảng Token Màu sắc Chuẩn (Canonical Color Token Ladder)

| Token Name | Figma UI3 Canonical (sRGB) | Figma UI3 Display-P3 Sampled | `product:os` Reader (`S02`) | EaseUI `.spec.json` Blueprint | Vai trò hiển thị (Semantic Usage) |
|---|---|---|---|---|---|
| `bg-canvas` | **`#1E1E1E`** (`rgb(30,30,30)`) | `#1C1C1C` | **`#16171B`** | `#09090B` / `#121215` | Nền trang Canvas mặc định (`S01`) và nền chính của Spec Reader (`S02`). |
| `bg-panel` | **`#2C2C2C`** (`rgb(44,44,44)`) | `#272727` | **`#1F2024`** | `#18181B` | Nền Left Rail (`48px`), Left Sidebar (`240px`), Right Inspector (`240px`), Bottom Toolbar (`528×48px`), Auto-Update Modal (`S09`) và Card cấp 1 trong Reader (`S02`). |
| `bg-input` / `bg-elevated` | **`#383838`** (`rgb(56,56,56)`) | `#313131` / `#3C3C3C` | **`#26272C`** | `#27272A` | Nền ô nhập (`X`, `Y`, `W`, `H`, `100%`), thanh chọn phân đoạn (`Flow`, `Channel`), hàng trang đang chọn và Header bảng trong Reader (`S02`). |
| `border-default` | **`#444444`** / **`#3C3C3C`** | `#3C3C3C` | **`#2E2F36`** | `#27272A` | Đường kẻ phân cách `1px` giữa các bảng, viền popover và đường chia bảng dữ liệu. |
| `text-primary` | **`#FFFFFF`** | `#FFFFFF` | **`#F3F4F6`** | `#FAFAFA` | Chữ tiêu đề, giá trị ô nhập, tên trang và nội dung chính. |
| `text-secondary` | **`#B3B3B3`** / **`#999999`** | `#999999` | **`#9CA3AF`** | `#A1A1AA` | Nhãn thuộc tính (`Position`, `Layout`), phụ đề `Team project` và breadcrumb. |
| `accent-primary` | **`#0D99FF`** | `#0F81E6` / `#71BCF7` | **`#3B82F6`** | **`#FF6A3D`** (BrainWave Orange) | Nút công cụ đang chọn (`Move`), nút `Share`, thanh tiến trình tải cập nhật (`S09`), viền chọn node trên Canvas và tab đang mở. |
| `accent-selection-row` | **`#323B55`** (`#0D99FF` @ 22%) | `#2D3340` | **`#1E293B`** | `#27272A` | Nền hàng Layer đang chọn và nút icon đang kích hoạt trên thanh Left Rail `48px`. |
| `accent-component` | **`#9747FF`** / **`#C58FFF`** | `#9C80CB` / `#352B47` | **`#8B5CF6`** | `#8B5CF6` | Màu biểu tượng và chữ của `Component (❖)`, `ComponentSet` và `Instance (◇)` trong cây Layers và trên Canvas. |
| `accent-dev-mode` | **`#24CB71`** | `#5FA773` | **`#10B981`** | `#10B981` | Huy hiệu `</>` Ready-for-Dev, huy hiệu `SHA-256 Verified ✓` / `Update Ready` (`S09`) và trạng thái Gate `APPROVED`. |
| `accent-warning` | **`#F24822`** / **`#FFCD29`** | `#E5484D` / `#F5A623` | **`#F59E0B`** / **`#EF4444`** | `#EF4444` | Đường gióng khoảng cách `Alt`, chốt hồng `padding`/`gap` (`#FF4081`), cảnh báo lệch schema / `SHA-256` (`S09`) và Gate `PENDING` / `MISSING`. |

---

## 2. Thông số Hình học & Tỷ lệ Khung Chuẩn Figma UI3 (Macro Layout Geometry)

| Vùng giao diện (Surface Region) | Kích thước chuẩn (1x Logical Pixels) | Bo góc (`Radius`) | Khoảng cách / Viền (`Border & Padding`) | Nguồn kiểm chứng |
|---|---|---|---|---|
| **Far-Left Navigation Rail** | Rộng **`48px`** (`x = 0..48px`), cao toàn cửa sổ | Nút icon `28×28px`, bo góc **`6px`** | Chỉ hiển thị icon vector (không có nhãn chữ dưới icon); vạch chia ngang `24×1px` `#3C3C3C` | `figma_live_reference.png`, `US-05` |
| **Left Sidebar (`File` / `Pages` / `Layers`)** | Rộng tối thiểu **`240px`** (`x = 49..289px`) | Hàng trang `28px` cao, bo góc **`6px`** | Header 2 dòng cao `64px` (`VSF - PCP ⌄` + `Team project`); cây Layers có đường dẫn thụt lề `16px/cấp` | `figma_live_reference.png`, `US-05` |
| **Canvas Rulers (Top & Left)** | Dày **`20px`** (`#2C2C2C`) | `0px` | Tô dải xanh `#1E3A5F` kèm chữ `#0D99FF` chiếu theo khung bao của phần tử đang chọn | `figma_live_editor.png`, `US-06` |
| **Right Design / Dev Inspector** | Rộng **`240px`** (`268px` khi cuộn/mở rộng) | Ô nhập `#383838` cao **`28px`**, bo góc **`6px`** | Dòng 1 (`38px`): Avatar + `▶ ⌄` + nút `Share` (`48×28px` `#0D99FF`); Dòng 2 (`36px`): `Design \| Prototype` + `Zoom % ⌄` | `figma_live_editor.png`, `US-05` |
| **Bottom Floating Toolbar** | **`528px × 48px`** (cách đáy `12px`) | Bo góc ngoài **`12px`**, nút trong **`6px`** | 7 nút công cụ tách (`32×32px` + `16×32px` chevron `⌄`) + vạch dọc `1×24px` + khay 4 nút `Toolbelt Mode` (`122×32px`) | `figma_live_editor.png`, `US-05` |
| **Bottom-Docked `Actions (Cmd+K)`** | **`480px × 320px`** (neo cách đáy `68px`) | Bo góc **`12px`** | Nền `#2C2C2C`, viền `1px` `#444444`, 4 tab lọc `All \| AI \| Actions \| Libraries` | `US-05`, `S06` |
| **Auto-Update & Schema Watchdog (`S09`)** | **`440px × 340px`** modal + **`208×28px`** Home pill | Bo góc modal **`12px`**, pill **`6px`** | Nền `#2C2C2C`, viền `1px` `#444444`, thanh chọn kênh 3 phân đoạn `Stable \| Beta \| Nightly` (`#383838`) | `US-09`, `S09` |
| **Desktop Multi-Tab Project Strip (`S10`)** | Cao **`36px`** toàn chiều ngang đỉnh cửa sổ | Bo góc tab **`6px`** (active `#2C2C2C`, inactive `#1E1E1E`) | Nút `Home` (`36×28px`) + cụm thương hiệu **`design`:os studio** + dải tab dự án (`140..220×28px`, icon loại file, chấm `•` dirty, ghim `Pin`, nút `×`) + nút `+` (`28×28px`) | `US-10`, `S10` |

---

## 3. Luật Cấm & Quy chuẩn Thương hiệu trong Thiết kế (`es:designer` Craft Floor & Brand Rules)

1. **Zero Emoji Icons in UI Chrome**: Cấm dùng ký tự emoji Unicode làm biểu tượng trên thanh điều hướng, tiêu đề bảng hay nút công cụ; bắt buộc dùng hệ thống glyph vector `crate::icons::Icon` (Lucide/Figma UI3 SVGs).
2. **No Cards-in-Cards**: Trong Spec Reader (`S02`), Inspector (`S05..S08`) và Auto-Update Modal (`S09`), cấm lồng một thẻ có viền/nền (`#1F2024` / `#2C2C2C`) bên trong một thẻ khác; phân tách nhóm con bằng đường kẻ mảnh `1px` (`#2E2F36` / `#3C3C3C`) hoặc dòng tiêu đề bảng `#26272C`.
3. **Tabular Numerals & Safe Truncation**: Mọi ô số liệu (`X`, `Y`, `W`, `H`, `%`, `SHA-256` prefix, KPI Strip, bảng `ui-contract.md`) phải căn thẳng hàng số và cắt gọn chữ an toàn (`elide`) không tràn khỏi khung.
4. **All-Lowercase Bold-Prefix Brand Wordmark (`BR-14` / `D10`)**: Tên thương hiệu sản phẩm luôn luôn viết thường toàn bộ là **`design`:os studio** (hoặc **`design`:os**), trong đó chữ **`design`** bắt buộc dùng font **Bold (`700`)** và `:os studio` dùng font **Regular (`400`)**. Cấm tuyệt đối mọi dạng viết hoa (`DESIGN:OS Studio`, `Design OS Studio`, `DESIGN:OS`) trên thanh tiêu đề, Home sidebar, tab bar và hộp thoại.
