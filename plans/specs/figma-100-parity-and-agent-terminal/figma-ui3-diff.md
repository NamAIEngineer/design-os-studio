# Figma UI3 Live Desktop (`com.figma.Desktop`) vs `design-os-studio` — 1:1 Pixel-by-Pixel Diff Report

**Captured Reference Artifacts:**
- **Unselected Page State (`VSF - PCP`):** [`figma_live_reference.png`](file:///Users/jangtrinh/.gemini/antigravity/brain/cdc69942-7241-4d03-83d5-38096c9fb771/figma_live_reference.png) (`3840×2100` @2x Retina = `1920×1050` logical 1x)
- **Selected Frame + Auto-Layout + Effects + Image Toolbar State (`VSF - PCP`):** [`figma_live_editor.png`](file:///Users/jangtrinh/.gemini/antigravity/brain/cdc69942-7241-4d03-83d5-38096c9fb771/figma_live_editor.png) (`3840×2100` @2x Retina = `1920×1050` logical 1x)
- **Additional Inspector States (`/tmp`):** `/tmp/figma_live_instance.png` (Component Instance `❖/◇` + Variant Properties), `/tmp/figma_live_section.png` (Section), `/tmp/figma_live_text.png` (Text + Typography section)
- **Current `design-os-studio` Render:** [`designos-agent-spec.png`](file:///Users/jangtrinh/Products/design-os-studio/docs/images/designos-agent-spec.png) (`3200×2000` @2x = `1600×1000` logical 1x)

---

## 1. Macro Chrome Geometry & Color Tokens (Exact Pixel Audit)

### 1.1 Horizontal & Vertical Split Geometry (1x Logical Pixels)

| Region | Figma UI3 Live (`1920×1050` 1x) | `design-os-studio` (`1600×1000` 1x) | Exact 1:1 Parity Fix Required |
| :--- | :--- | :--- | :--- |
| **Top Desktop Tab Bar** | `y = 0..38px` (`38px` tall; Home icon + File tabs `VSF - PCP ×` + `+` button). Excluded in web/canvas-only mode, or `38px` native title/tab bar. | None (`y = 0` starts directly at panels). | Optional desktop tab bar (`38px` tall) or keep `0px` in single-document studio window. |
| **Far-Left Navigation Rail** | `x = 0..48px` (**`48px` wide**), `1px` border at `x = 48..49px` (`#3C3C3C` / sRGB `#444444`). **Icon-only** (`28×28px` hit targets, no text labels below icons!). | `x = 0..56px` (**`56px` wide**), `1px` border at `x = 56..57px`. Shows **text labels** (`File`, `Assets`, `Specs`, `Tools`, `Variables`, `Plugins`) under icons. | Narrow rail from `56px` → **`48px`** (`-8px`). Remove text labels under rail icons; use `28×28px` icon buttons (`6px` radius) with active blue-tinted pill `#323B55` (or `#0D99FF26`). |
| **Left Sidebar (`File` / `Pages` / `Layers`)** | `x = 49..289px` (**`240px` min width**, resizable via `slider Resize handle, Value: 240 pixels (min)`), `1px` right border at `x = 289..290px`. | `x = 57..297px` (**`240px` wide**), `1px` right border at `x = 297..298px`. | Width `240px` matches (`x = 49..289px` once rail is `48px`). |
| **Canvas Rulers (Top & Left)** | `20px` thick (`x = 290..310px` vertical ruler, `y = 38..58px` horizontal ruler) with `#2C2C2C` bg, `1px` border `#3C3C3C`, and blue selection span highlight (`#1E3A5F` fill with `#0D99FF` coordinate labels `0 .. 1347`). | Missing canvas rulers. | Add optional `20px` top & left canvas rulers with blue selection projection band. |
| **Right Design Inspector** | `x = 1680..1920px` (**`240px` min width** + `1px` left border at `x = 1679..1680px`; expands to `268px` when hovered/scrolled). | `x = 1360..1600px` (**`240px` wide** + `1px` left border at `x = 1359..1360px`). | Width `240px` matches (`240px` default, `268px` expanded). |
| **Bottom Floating Toolbar** | Centered in canvas viewport (`x = 696..1224px`, **`528px` wide**, **`48px` outer / `44px` inner height**, `y = 990..1038px`, **`12px` bottom margin**, **`12px` corner radius**). | `x = 544..1056px` (`512px` wide, `48px` outer height, `y = 940..988px`, `12px` bottom margin). | Match Figma UI3 tool order, active tool blue `#0D99FF` (instead of EaseUI orange `#E36323`), and 4-icon `Toolbelt Mode` segmented well (`Draw`, `Design`, `Motion`, `Dev Mode </>`). |

---

### 1.2 Exact Hex Color Palette: Figma UI3 Dark Mode vs `design-os-studio`

> **Color-Space Note:** Live macOS screenshots captured via `orca computer` go through Display P3 window backing (`#2C2C2C` sRGB renders as `#272727` in raw screenshot bytes; `#1E1E1E` sRGB renders as `#1C1C1C`, confirmed by the live Figma `Page` background hex field explicitly reading `1E1E1E` in `figma_live_reference.png`). Both canonical sRGB tokens and Display-P3 sampled values are listed below.

| Semantic Token | Figma UI3 Canonical (sRGB) | Figma UI3 Display-P3 Sampled | Current `design-os-studio` (`designos-agent-spec.png`) | Visual / Token Mismatch |
| :--- | :--- | :--- | :--- | :--- |
| **Panel / Chrome Background (`bg-panel`)** | **`#2C2C2C`** | `#272727` | `#121215` (Left Rail, Left Sidebar, Right Inspector) | **MAJOR:** Our panels are near-black (`#121215`), whereas Figma UI3 uses neutral warm charcoal **`#2C2C2C`** (`rgb(44,44,44)`). |
| **Default Dark Canvas Background (`bg-canvas`)** | **`#1E1E1E`** | `#1C1C1C` | `#F5F5F5` (Light page bg in `designos-agent-spec.png`) | Default dark-mode page background in Figma UI3 is **`#1E1E1E`**. |
| **Input / Control Well / Hover Pill (`bg-secondary` / `bg-input`)** | **`#383838`** | `#313131` / `#3C3C3C` | `#18181B` / `#27272A` | Figma UI3 inputs (`X`, `Y`, `W`, `H`, `Opacity`, `Corner radius`, `Color` well) and selected page row use **`#383838`** fill (`6px` radius) over `#2C2C2C` panel bg (lighter than panel, not darker!). |
| **Panel & Section Hairline Border (`border-default`)** | **`#444444`** (or `#383838` internal) | `#3C3C3C` / `#3D3D3D` | `#27272A` | Increase border luminance to **`#3C3C3C`–`#444444`** so `1px` panel and section dividers read crisply against `#2C2C2C`. |
| **Primary Selection Accent (`accent-primary`)** | **`#0D99FF`** | `#0F81E6` (button) / `#71BCF7` (icon tint) / `#323B55` (row/rail tint) | `#E36323` (EaseUI orange on active Move tool & agent pill) | **MAJOR:** Figma UI3 uses **`#0D99FF`** for active tool button (`Move`), `Share` button, canvas selection box, and `#323B55` (`#0D99FF` @ 22% alpha) for selected layer rows & active left-rail icon. |
| **Component / Instance Purple (`accent-component`)** | **`#9747FF`** (icon/border) / **`#C58FFF`** (layer text) | `#9C80CB` (row icon & label) / `#382D4C` (selected component row bg) | `#A855F7` (canvas text only; missing in Layers tree) | In Figma UI3 Layers tree, Components (`❖`) and Instances (`◇`) tint **both the icon and the layer label text** light purple (`#B794F4` / `#9C80CB`), with a purple-tinted selection row `#352B47`. |
| **Dev Mode / Ready-for-Dev Green (`accent-dev`)** | **`#24CB71`** | `#5FA773` | `#22C55E` | Used on `</>` Ready-for-Dev badges in the Pages list (`CC`, `CC · 30/10`, `IDP`, `Design system`) and the Dev Mode toggle. |

---

## 2. Component-by-Component 1:1 Diff

### 2.1 Far-Left Navigation Rail (`48px` wide) & Left Sidebar (`240px` wide)

1. **Far-Left Navigation Rail (`x = 0..48px`):**
   - **Figma UI3 Live (`23 toolbar Navigation Bar`):**
     - Width is **`48px`** (`x = 0..48px`), full window height, `#2C2C2C` background, `1px` right border `#3C3C3C`.
     - Top to bottom (`28×28px` icon buttons centered at `x = 24px`, `12px` vertical gap, **NO text labels**):
       1. `Main menu` (Figma logo + small down chevron, `y = 58px`)
       2. Horizontal `1px` splitter (`width = 24px`, `x = 12..36px`, `y = 85px`, `#3C3C3C`)
       3. `File` (`button (expanded) File`, page/document icon inside `28×28px` rounded-`6px` pill with `#323B55` active blue tint, `y = 108px`)
       4. `Agents` / Sparkle icon (`pop up button Agents`, 4-point star with plus, `y = 144px`)
       5. `Assets` (component library book/grid icon, `y = 180px`)
       6. `Tools` / Community globe/puzzle icon (`y = 216px`)
       7. Horizontal `1px` splitter (`width = 24px`, `x = 12..36px`, `y = 248px`, `#3C3C3C`)
       8. `Variables` (sliders/tokens icon, `y = 274px`)
       9. Bottom-pinned icon: `Review unpublished changes` / Library updates (`y = 1020px`).
   - **Current `design-os-studio` Diff:**
     - Rail is `56px` wide (`+8px` too wide) and renders `9px` text labels (`File`, `Assets`, `Specs`, `Tools`, `Variables`, `Plugins`) underneath each icon.
     - Active `File` icon has an orange/brown fill instead of Figma's `#323B55` blue tint.

2. **Left Sidebar Header (`x = 49..289px`, `y = 38..102px`, `64px` total height):**
   - **Figma UI3 Live:**
     - Row 1 (`y = 48..72px`): File title `VSF - PCP` (`13px` SemiBold `#FFFFFF`) + small chevron `⌄` (`Edit file menu`) on the left; `Minimize UI` sidebar-toggle icon (`16×16px` split-rect icon) at `x = 266px`.
     - Row 2 (`y = 72..94px`): Subtitle link `Team project` (`11px` Regular `#999999` / `#B3B3B3`).
     - Bottom border: `1px` solid `#3C3C3C` at `y = 102px`.
   - **Current `design-os-studio` Diff:**
     - Only renders a single `40px` header row (`easeui-agent-console ⌄` + collapse icon) and omits the `Team project` / workspace subtitle line (`24px`).

3. **Pages Collapsible List (`y = 103..799px`, resizable via horizontal splitter):**
   - **Figma UI3 Live:**
     - Section header (`36px` tall, `y = 103..139px`): `Pages` (`11px` SemiBold `#FFFFFF` at `x = 65px`) + `Find` magnifier icon (`x = 246px`) + `Add new page` `+` icon (`x = 268px`).
     - Page rows (`28px` height each, `6px` border-radius inset `x = 57..281px` = `224px` pill width):
       - Selected page (`↳ Integration Management`): `#383838` (sampled `#313131`) pill background, `#FFFFFF` `11px` text.
       - Inactive pages: `#B3B3B3` `11px` text, transparent background.
       - Page dividers (`58 row Page divider`): rendered as `1px` horizontal line `#3C3C3C` (`12px` vertical slot).
       - Section header pages with Dev Mode status (`CC`, `CC · 30/10`, `IDP`, `Design system`): right-aligned green `</>` badge (`#24CB71` / `#5FA773`).

4. **Layers Tree (`y = 800..1050px`):**
   - **Figma UI3 Live:**
     - Top splitter (`1px` `#3C3C3C` at `y = 799px`) + `Layers` header (`32px` tall, `y = 800..832px`): `Layers` (`11px` SemiBold `#FFFFFF`) on left, `Collapse layers` icon (`14×14px` stacked-chevron-up icon, **NOT** a search magnifier!) at `x = 268px`.
     - Virtualized tree rows (`28px`–`32px` row height; `32px` stride when spacious/comfortable mode is active, `28px` compact):
       - **Selection states:**
         - Ancestor of selected node (`IM · Archive · replaced by code shots`): subtle blue-gray tint `#2D3340` with right-aligned `🔒` (`Toggle layer locking`) and `👁` (`Toggle layer visibility`) icons visible on hover/selection.
         - Directly selected Frame/Text row (`screen:edit-service`): solid muted selection blue `#323B55` (sRGB `#1E3A5F` / `#0D99FF` @ 25%) across full width (`x = 53..285px`, `4px` radius).
         - Directly selected Component Instance row (`[ARCHIVED] shot/4/...`): purple-tinted selection row `#352B47` with purple text `#B794F4` / `#9C80CB`.
       - **Layer Type Icons (`12×12px` glyph at `8px` left of label):**
         - `Section`: rounded-corner tag/folder-frame icon (`#B3B3B3` or `#FFFFFF` when selected).
         - `Frame`: `#` hash grid icon (`#B3B3B3` default, `#FFFFFF` when selected).
         - `Auto-Layout`: `||` (horizontal bars) or `=` (vertical bars) (`#B3B3B3`).
         - `Component`: 4-diamond `❖` (`#9747FF`).
         - `Instance`: hollow diamond `◇` (`#9747FF` / `#9C80CB`, label text ALSO colored `#9C80CB`!).
         - `Text`: `T` serif/sans glyph (`#B3B3B3`).
       - **Indent Guide Lines:** `1px` vertical lines (`#3C3C3C`) under expanded parent chevrons (`16px` indent per nesting depth).
   - **Current `design-os-studio` Diff:**
     - `Layers` header right icon is a Search magnifier instead of Figma UI3's `Collapse layers` icon (search is already in the `Pages` header).
     - Layer icons only show generic grey boxes (`=`) without distinct `#` Frame, `❖` Component purple, `◇` Instance purple (with purple text), `T` Text, or right-aligned `🔒` / `👁` action icons on selected/hovered rows.

---

### 2.2 Bottom Floating Toolbar (`528px × 48px` outer / `44px` inner, `12px` radius)

| Attribute | Figma UI3 Live (`324 toolbar Editor`) | Current `design-os-studio` | Required 1:1 Parity Change |
| :--- | :--- | :--- | :--- |
| **Outer Dimensions & Placement** | `528px × 48px` (`x = 696..1224px`, `y = 990..1038px` in `1920×1050` window), `12px` above bottom edge, `12px` border radius, `#2C2C2C` fill, `1px` `#3E3E3E` top border, drop shadow `0 4px 16px rgba(0,0,0,0.45)`. | `512px × 48px` (`x = 544..1056px`, `y = 940..988px` in `1600×1000` window), `#18181B` fill, `1px` `#2C2C2F` border. | Change background from `#18181B` → **`#2C2C2C`** and border to **`#3E3E3E`**. |
| **Active Tool Highlight** | `32×32px` rounded-`6px` button at `x = 703.5..735.5px`, `y = 998..1030px` (`8px` inset from top/bottom), filled with **Figma Selection Blue `#0D99FF`** (`#0F81E6` P3) and white cursor arrow icon. | `32×32px` button filled with **EaseUI Orange `#E36323`**. | Replace `#E36323` orange with **`#0D99FF`** Figma selection blue. |
| **Left Tool Group (7 items, `57px` stride for split buttons)** | Each split tool has a `32×32px` main button + `16×32px` `⌄` flyout chevron button:<br>1. **Move** (`V`) + `⌄` (`Move tools`: Move, Hand, Scale)<br>2. **Frame / Section** (`F`/`S`) + `⌄` (`Region tools`: Frame, Section, Slice)<br>3. **Rectangle** (`R`) + `⌄` (`Shape tools`: Rectangle, Line, Arrow, Ellipse, Polygon, Star, Image)<br>4. **Pen** (`P`) + `⌄` (`Creation tools`: Pen, Pencil)<br>5. **Text** (`T`) + `⌄` (`Type tools`: Text, Table)<br>6. **Comment** (`C`, with red/orange unread dot badge if unread) + `⌄`<br>7. **Actions** (`⌘K` — 4-square / AI sparkle icon, `32×32px`, **no chevron**). | Has `Move ⌄`, `Frame (#) ⌄`, `Rectangle ⌄`, `Pen ⌄`, `Text ⌄`, `Comment` (no chevron), `Sparkle` (wrong icon for Actions). | Add `⌄` flyout chevron to `Comment`, and use Figma UI3's **Actions** icon (3 squares + 4-point sparkle in top-right corner) for button #7. |
| **Vertical Divider** | `1px × 24px` line at `x = 1086px` (`#3C3C3C` / `#444444`), `8px` horizontal margin on each side. | `1px` line at `x = 917px` (`#27272A`). | Match `#444444` color. |
| **Right Mode Switcher (`332 radio group Toolbelt Mode`)** | A `122px × 32px` recessed track (`x = 1094.5..1216.5px`, `#383838` / `#3C3C3C` fill, `6px` radius) containing **4 radio mode buttons** (`28×28px` each):<br>1. **Draw** (squiggle pen icon)<br>2. **Design** (active: `#2C2C2C` inset pill with **`#71BCF7` light-blue** ruler/layout icon)<br>3. **Motion / Variables** (diamond icon)<br>4. **Dev Mode (`</>`)** toggle button (turns green `#24CB71` when active). | Has 4 icons (`Pencil`, `Cursor` in orange/brown tint, `Component ❖`, `Dev Mode </>`). | Fix the 4 `Toolbelt Mode` icons to match Figma UI3: (1) **Draw** squiggle pen, (2) **Design** layout/panel icon (highlighted with `#2C2C2C` pill + `#71BCF7` blue icon when active), (3) **Motion/Prototype** diamond, (4) **Dev Mode** `</>`. |
| **Contextual Secondary Toolbar (Above Main Pill)** | When an image-filled Frame is selected (`figma_live_editor.png`), Figma renders `338 toolbar Image` (`Crop`, `Select area`, `Remove background`, `Edit with prompt`, `More ⌄`) `8px` above the main toolbar (`y = 934..982px`). | Renders a static toast pill (`● Opened easeui-agent-console`) above the toolbar. | Auto-dismiss file-open toast after `2s` so it does not permanently block the canvas or collide with contextual secondary toolbars. |

---

### 2.3 Right Design Inspector (`240px` min width, `x = 1680..1920px`)

| Section (Top → Bottom) | Figma UI3 Live (`com.figma.Desktop`) | Current `design-os-studio` (`designos-agent-spec.png`) | Actionable 1:1 Parity Diff |
| :--- | :--- | :--- | :--- |
| **1. Top Multiplayer & Share Header (`y = 38..76px`, `38px` tall)** | Left: Multiplayer avatar circles (`24×24px`, `O`, `H`, `O` + `⌄` dropdown).<br>Right: `Present ▶` outline icon + `⌄` (`x = 1838..1868px`) + **`Share` primary blue button** (`48×28px`, `6px` radius, **`#0D99FF`** fill, `#FFFFFF` `11px` SemiBold text at `x = 1874..1912px`). | Missing Multiplayer avatars and missing **`Share` (`#0D99FF`) button** — only shows `▶ ⌄` on the right side of the first row! | Add top bar row (`38px` tall) with avatar pill on the left and `▶ ⌄` + blue **`Share`** (`#0D99FF`, `48×28px`, `6px` radius) button on the right. |
| **2. Mode Tabs & Zoom Row (`y = 76..112px`, `36px` tall)** | Left: `Design` (active: `#FFFFFF` `11px` SemiBold) and `Prototype` (inactive: `#999999` `11px` Medium) text tabs (`no dark pill box` in UI3 when unselected, or subtle `#383838` hover pill).<br>Right: Zoom dropdown `7% ⌄` (`11px` `#FFFFFF`) at `x = 1880..1912px`. Bottom `1px` border `#3C3C3C`. | Combines `Design | Prototype`, `65% ⌄`, and `▶ ⌄` into two cramped rows without `Share`. | Move `▶ ⌄` + `Share` to Row 1 (`y = 38..76px`), keeping Row 2 (`y = 76..112px`) strictly for `Design` / `Prototype` tabs on the left and `Zoom % ⌄` on the right. |
| **3. Node Type & Component Header (`y = 113..188px`)** | Displays selected node type dropdown (`Frame ⌄`, `Section ⌄`, `Browser ⌄`, or `Text`) in `12px` SemiBold `#FFFFFF` on left + quick action icons on right (`</>` Ready for dev, `❖` Create component, `⤢` More `⋯`).<br>• When **Frame** selected: full-width outline button `Copy to Figma Slides` (`28px` tall, `6px` radius, `1px` `#444444` border).<br>• When **Instance** selected: `From this file ◇` subtitle + variant property rows (`Property 1: [Chrome ⌄]`, `Property 2: [Dark ⌄]`). | Not rendered in unselected state (`designos-agent-spec.png` has no selected node); verify selected node header includes node-type dropdown + right action icons (`</>`, `❖`, `⋯`). | Ensure selected-node header renders `NodeType ⌄` on left and `</>`, `❖`, `⋯` icon buttons on right. |
| **4. Position Section (`y = 189..324px`)** | Header: `Position` (`11px` SemiBold `#FFFFFF`).<br>• **Row 1 (Alignment):** Two `3-button` segmented bars (`100px × 28px` each, `#383838` fill, `6px` radius): Horizontal (`Align left`, `Align horizontal centers`, `Align right`) + Vertical (`Align top`, `Align vertical centers`, `Align bottom`) + right `More actions` (`||=` distribute icon).<br>• **Row 2 (`X` / `Y`):** Two `100px × 28px` `#383838` input pills: `X  7152` and `Y  8278` (`#999999` prefix label, `#FFFFFF` value).<br>• **Row 3 (`Rotation` & Transforms):** Left `100px × 28px` pill `∟ 0°`; Right three `28×28px` icon buttons (`Rotate 90° right`, `Flip horizontal`, `Flip vertical`). | Ensure all 3 rows (`Alignment 3+3+1`, `X / Y`, `Rotation 0° + Rotate/FlipH/FlipV`) use `#383838` (`6px` radius, `28px` height) pills. | Match exact 3-row `Position` layout (`Alignment` segmented 3+3 bars, `X`/`Y` pills, `∟ 0°` + 3 transform icon buttons). |
| **5. Layout / Auto-Layout Section (`y = 325..468px`)** | Header: `Layout` (`11px` SemiBold `#FFFFFF`) + right icons: `Resize to fit` (`⤡`) and `Toggle auto layout` (`⊞`).<br>• **Row 1 (`Flow` — 4-segment radio bar):** Full-width `208px × 28px` `#383838` segmented control with 4 buttons: **`Freeform`** (dashed circle/square), **`Vertical`** (`↓` / `=`), **`Horizontal`** (`→` / `||`), **`Grid`** (`2×2` grid).<br>• **Row 2 (`Dimensions`):** `W  1920` (`92px × 28px` pill, with `Fixed/Hug/Fill` dropdown when Auto-Layout is active) + `H  1347` (`92px × 28px` pill) + right `Lock aspect ratio` (`28×28px` proportions icon).<br>• **Row 3 (Auto-Layout Alignment & Gap/Padding / Clip):** When Auto-Layout is active, shows `3×3` alignment matrix box + `Gap` (`||` / `=`) and `Padding` (`⊞` horizontal/vertical) `#383838` pills; plus `[✓] Clip content` checkbox (`#0D99FF` checked box). | Ensure `Flow` 4-mode segmented control (`Freeform`, `Vertical`, `Horizontal`, `Grid`) is present above `W` / `H` dimensions. | Add the 4-segment `Flow` radio group (`Freeform`, `Vertical`, `Horizontal`, `Grid`) and `Resize to fit` + `Toggle auto layout` header icons. |
| **6. Appearance Section (`y = 469..548px`)** | Header: `Appearance` (`11px` SemiBold `#FFFFFF`) + right icons: `👁` (`Hide`) and `💧` (`Apply blend mode` teardrop).<br>• **Row 1:** Left `92px × 28px` pill `[▫ 100%]` (`Opacity`); Middle `92px × 28px` pill `[⌜ 12]` (`Corner radius`); Right `28×28px` icon button `⛶` (`Individual corners`). | Ensure `Appearance` is its own dedicated section (not merged into `Position` or `Fill`) with `👁` and `💧` header icons and `Opacity %` + `Corner radius` + `Individual corners` row. | Match exact `Appearance` header (`👁`, `💧`) and 3-column control row (`Opacity 100%`, `Corner radius`, `Individual corners`). |
| **7. Typography Section (When Text Selected, `/tmp/figma_live_text.png`)** | Header: `Typography` + `4-dot style` icon.<br>• Row 1: Full-width `Inter ⌄` font family combo box (`28px` tall, `#383838`).<br>• Row 2: `Medium ⌄` weight (`116px`) + `72 ⌄` font size (`84px`).<br>• Row 3: `Line height: Auto` (`98px`) + `Letter spacing: 0%` (`98px`).<br>• Row 4: Horizontal align (`3-segment`: Left/Center/Right) + Vertical align (`3-segment`: Top/Middle/Bottom) + `Type settings` (`⋯`). | Verify text inspector layout matches 4-row UI3 Typography layout. | Match 4-row UI3 `Typography` block when `Text` node is selected. |
| **8. Fill Section (`y = 549..628px`)** | Header: `Fill` + right `4-dot styles` (`∷`) + `+` (`Add fill`).<br>• Fill row (`28px` tall): `[swatch 16×16px] FFFFFF | 100 %` inside a `164px × 28px` `#383838` combined pill + right `👁` (`Toggle visibility`) + `−` (`Remove`). | In Unselected Page mode (`designos-agent-spec.png`), `Page` background row shows `[#F5F5F5] [100 %] 👁`. | Keep combined `[swatch + hex | opacity %]` `#383838` pill (`6px` radius) + `👁` + `−` icons. |
| **9. Stroke Section (`y = 629..744px`)** | Header: `Stroke` + `∷` + `+`.<br>• Row 1: `[swatch] FFFFFF | 100 %` pill + `👁` + `−`.<br>• Row 2: `Outside ⌄` (`Stroke align`, `92px × 28px` pill) + `[≡ 24]` (`Stroke weight`, `64px × 28px` pill) + `Advanced stroke settings` (sliders icon) + `Individual strokes` (`⛶` icon). | Ensure Row 2 of `Stroke` includes `Inside/Center/Outside ⌄` dropdown + `Weight` pill + `Advanced` & `Individual strokes` icons. | Match 2-row `Stroke` layout when a node has an active stroke. |
| **10. Effects Section (`y = 745..888px`)** | Header: `Effects` + `∷` + `+`.<br>• Each effect row (`28px` tall): `[▫ effect settings icon + Drop shadow ⌄]` inside a `164px × 28px` `#383838` dropdown pill + right `👁` + `−`. | Ensure `Effects` rows render the left effect-settings sun/shadow icon inside the `Drop shadow ⌄` `#383838` pill. | Match `[icon + Drop shadow ⌄]` pill + `👁` + `−` row layout. |
| **11. Selection Colors / Layout Guide / Export (`y = 889..980px`)** | • `Selection colors` (when multi-color container selected): horizontal swatch strip `[⬜][⬛][⬛] +11`.<br>• `Layout guide` (`+` collapsed row).<br>• `Export` (`+` collapsed row).<br>• Bottom-right floating `?` Help button (`32×32px` circle at `x = 1894px, y = 1020px`). | Unselected Page view in `designos-agent-spec.png` shows `Page`, `Styles`, `Export` (missing `Text styles / Color styles / Effect styles` list when styles exist, and missing bottom-right `?` button). | Add bottom-right `?` (`32×32px` dark circle `#2C2C2C` with `1px` `#444444` border) help button in Right Inspector footer area. |

---

## 3. Top Priority Action Checklist for 100% Figma UI3 Visual Parity

1. **Theme Surface & Input Tokens (`#2C2C2C` / `#383838` / `#1E1E1E` / `#0D99FF`):**
   - Switch panel chrome (`Left Rail`, `Left Sidebar`, `Right Inspector`, `Bottom Floating Toolbar`) from `#121215`/`#18181B` → **`#2C2C2C`**.
   - Switch input wells, segmented button tracks, and selected page pills from darker `#18181B` → **lighter `#383838`** (`6px` corner radius, `28px` height).
   - Switch active tool (`Move`), `Share` button, and selection accent from EaseUI orange `#E36323` → **Figma UI3 Blue `#0D99FF`** (with `#323B55` for selected layer rows and active left-rail button).
2. **Far-Left Navigation Rail (`48px` icon-only):**
   - Shrink rail width from `56px` → **`48px`** and remove text captions below rail icons (`28×28px` icon buttons with horizontal `24px` splitters).
3. **Left Sidebar Header, Pages & Layers Tree (`240px` wide):**
   - Add the `Team project` subtitle line below the file name in the top-left header (`64px` total header height).
   - Replace the duplicate search icon in the `Layers` section header with Figma UI3's `Collapse layers` icon.
   - Color Component (`❖`) and Instance (`◇`) icons **and layer label text** in Figma component purple (`#9747FF` / `#B794F4`), and show `🔒` + `👁` action icons on selected/hovered layer rows.
4. **Bottom Floating Toolbar (`528px × 48px`, `12px` radius):**
   - Use `#0D99FF` for the active `Move` button, add the `⌄` flyout chevron to `Comment`, use the 3-squares+sparkle `Actions` icon, and render the 4-icon `Toolbelt Mode` well (`Draw`, `Design` with `#2C2C2C` active pill + `#71BCF7` blue icon, `Motion`, `Dev Mode </>`).
5. **Right Inspector (`240px` wide):**
   - Add the top Multiplayer + blue **`Share`** (`#0D99FF`) header row above `Design | Prototype` + `Zoom % ⌄`.
   - Ensure selected Frame inspector renders: `NodeType ⌄` header, `Position` (`Alignment 3+3+1`, `X`/`Y`, `Rotation 0° + 3 flip/rotate buttons`), `Layout` (`Flow` 4-mode radio group `Freeform | Vertical | Horizontal | Grid`, `W`/`H` pills, `Clip content`), `Appearance` (`Opacity %`, `Corner radius`, `Individual corners`), `Fill`, `Stroke`, `Effects`, `Layout guide`, and `Export`.
