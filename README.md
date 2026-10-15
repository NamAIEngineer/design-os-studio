# DESIGN:OS Studio (`design-os-studio`)

**Native GPU-accelerated interface design studio, enterprise Figma `.fig` v106 binary read/write engine, Agentic PTY Chat UI, 11-stage Spec Reader, and 22-tool Model Context Protocol server built in 100% Rust.**

Open and save 700,000+ record enterprise Figma `.fig` v106 binary archives in ~1.5s with `0.0%` idle CPU on Metal, DirectX 12, and Vulkan (`wgpu` + `tiny-skia`). Run **Claude Code (`claude`)**, **OpenAI Codex (`codex`)**, **Antigravity CLI (`agy`)**, and **Grok CLI (`grok`)** directly inside the native **Agentic PTY Chat UI** with zero paid external API keys, audit 11-stage `PRODUCT:OS Spec Driven v2` requirements (`US-01..US-17` + Gherkin + `g1..g5` gates), generate multi-target Dev Mode code (React, Tailwind, CSS, SwiftUI, Jetpack Compose), and automate headless GPU visual QA from any terminal or CI pipeline.

<p align="center">
  <img src="docs/images/designos-hero.png" alt="DESIGN:OS Studio rendering the 112,474-node VSF-PCP enterprise Figma workspace at 60% zoom with 1:1 Figma UI3 dark chrome and unified Phosphor icons" width="880">
  <br>
  <sub><b>Verified Production Build:</b> <b>DESIGN:OS Studio</b> rendering an enterprise <code>.fig</code> v106 design system (<b>723,136 Kiwi records / 112,474 scene nodes</b>) natively on Apple Silicon Metal in <b>1.54s</b> with <b>0.0% idle CPU</b> · <b>387 / 387 (100.00%)</b> Figma <code>126.9.13</code> UI3 &amp; <code>.fig</code> v106 parity · <b>824</b> automated workspace tests passing.</sub>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT_%2F_Apache--2.0-202020?style=flat-square&labelColor=202020&color=444444" alt="Dual MIT OR Apache-2.0 License"></a>
  <img src="https://img.shields.io/badge/engine-100%25_Rust_%C2%B7_wgpu_%2B_tiny--skia-202020?style=flat-square&labelColor=202020&color=444444" alt="100% Rust wgpu and tiny-skia engine">
  <img src="https://img.shields.io/badge/Figma_.fig_v106-Native_Read_%2F_Write_(387%2F387)-202020?style=flat-square&labelColor=202020&color=444444" alt="Figma .fig v106 Native Read and Write 387/387 parity">
  <img src="https://img.shields.io/badge/Claude_Code_%C2%B7_Codex_%C2%B7_Antigravity_%C2%B7_Grok-MCP_%2B_PTY_Chat-202020?style=flat-square&labelColor=202020&color=444444" alt="Compatible AI coding agents via MCP and PTY Chat">
  <a href="commands/doctor.md"><img src="https://img.shields.io/badge/diagnostics-%2Fdoctor-202020?style=flat-square&labelColor=202020&color=444444" alt="Doctor diagnostic slash command"></a>
  <a href="llms.txt"><img src="https://img.shields.io/badge/GEO_index-llms.txt-202020?style=flat-square&labelColor=202020&color=444444" alt="GEO machine-readable llms.txt index"></a>
  <a href="https://jangtrinh.github.io/design-os-studio/"><img src="https://img.shields.io/badge/live_docs-GitHub_Pages-202020?style=flat-square&labelColor=202020&color=444444" alt="Live GitHub Pages documentation"></a>
  <a href="https://buymeacoffee.com/jangtrinh"><img src="https://img.shields.io/badge/Buy_Me_a_Coffee-FFDD00?style=flat-square&logo=buy-me-a-coffee&logoColor=black" alt="Support Jang Trinh on Buy Me a Coffee"></a>
</p>

<p align="center">
  <a href="#quickstart">Quickstart</a> · <a href="#what-it-makes">What it makes</a> · <a href="#visual-proof-gallery">Visual proof</a> · <a href="#installation">Install</a> · <a href="#how-it-works">How it works</a> · <a href="#commands--diagnostics">Commands</a> · <a href="#quality-bar--limitations">Limits</a>
</p>

---

## Why this exists

Browser-wrapped Electron design tools choke on 100,000+ node enterprise files, lock `.fig` binary authoring behind proprietary cloud servers, and force AI coding agents to scrape 150KB+ of noisy REST JSON without direct canvas write-back or local spec verification. **`design-os-studio`** is a local-first, 100% Rust GPU studio (`wgpu` + `tiny-skia`) with native `fig-kiwi` v106 binary read/write, an embedded Agentic PTY Chat UI for your existing CLI subscriptions (`claude`, `codex`, `agy`, `grok`), an 11-stage Spec Reader, and a 22-tool stdio MCP server.

| Dimension | Cloud / Electron Design Tools & Read-Only MCPs | DESIGN:OS Studio (`design-os-studio`) |
| :--- | :--- | :--- |
| **Enterprise Scale (`723k` Records)** | Browser tab OOM crashes, multi-gigabyte RAM spikes, and constant idle GPU repaints | Uniform `2048x2048` `SceneIndex` culling, 4-tier semantic zoom LOD, `1.54s` cold load, and **`0.0%` static viewport CPU** |
| **Figma `.fig` v106 Binary Parity** | Cloud-only exports or lossy third-party JSON converters | **Native `.fig` v106 Read & Write** (`DEFLATE`/`Zstd` `canvas.fig` + `meta.json` + `images/`, `symbolOverrides`, `parameterConsumptionMap`) |
| **Agentic Workflow & Cost** | Separate paid AI chat seats or read-only 155K-char REST dumps | **Native Agentic PTY Chat UI** (`Claude Code` · `Codex` · `Antigravity` · `Grok`) + **22-tool `designos-mcp` server** + 1-click `NodeSpec` canvas apply |
| **Spec-to-Canvas Traceability** | Markdown PRDs disconnected from canvas frames and QA gates | **In-App 11-Stage `PRODUCT:OS Spec Driven v2` Reader** (`US-01..US-17` + Gherkin `Given/When/Then` + `g1..g5` gates + 500ms live disk hot-reload) |
| **Privacy & Telemetry** | Cloud account lock-in, mandatory online sign-in, and background telemetry | **Zero telemetry** — 100% local execution on macOS, Windows, and Linux ([`PRIVACY.md`](PRIVACY.md) audited) |

---

## Quickstart

### 1. Install the plugin, native installer, or build from source

**In Claude Code:**

```text
/plugin marketplace add jangtrinh/design-os-studio
/plugin install design-os-studio@design-os-studio
```

**Or from your terminal (macOS, Windows, Linux — standalone Studio + CLI + MCP):**

```bash
git clone https://github.com/jangtrinh/design-os-studio
cd design-os-studio

# Launch DESIGN:OS Studio desktop GUI (or pass a .fig / .designos file path)
cargo run --release -p designos-app --bin designos-studio

# Or build native OS installers (.dmg / -setup.exe / .deb) into dist/
./scripts/build-release.sh 0.2.0 --local-host
```

### 2. Ask in plain English (or invoke the Agent CLI directly)

Open your project in your AI coding agent (or inside the Studio's `Agents` left-rail PTY Chat panel) and describe the outcome you need:

> "Apply `examples/specs/easeui-agent-console.spec.json` onto the DESIGN:OS Studio canvas, export a verified Figma `.fig` v106 binary archive to `out.fig`, and capture a 1600x1000 GPU screenshot."

> "Open the 11-stage PRODUCT:OS Spec Driven v2 Reader (`--panel specs`), audit all User Stories (`US-01..US-17`) and `g1..g5` gates, and resolve all open canvas pin-comments."

> "Inspect the `API Docs` frame in Dev Mode, generate production React + Tailwind + SwiftUI code, and run `designos-diff` to verify 0.00% visual regression."

### 3. Verify your setup

```bash
python3 scripts/self_check.py
```

---

## What it makes

Every artifact produced by `design-os-studio` is backed by deterministic scene-graph compilation (`SceneCompiler` + `GeomCache`), native `fig-kiwi` v106 binary serialization, and headless GPU/CPU visual verification (`--screenshot` + `designos-diff`).

### Pillar 1 — Native Agentic PTY Chat UI (`Claude Code` · `Codex` · `Antigravity` · `Grok CLI`)

<p align="center">
  <img src="docs/images/designos-agents-chat.png" alt="DESIGN:OS Studio Agents Left Rail with Native PTY Chat UI wrapping Claude Code, OpenAI Codex, Antigravity CLI, and Grok CLI" width="880"><br>
  <sub>Command: <code>designos-studio --fixture showcase --panel agents-chat --size 1440x900 --scale 2 --screenshot docs/images/designos-agents-chat.png</code></sub>
</p>

Inside the Figma UI3 **Agents** Left Rail (`RailItem::Agents`), `design-os-studio` wraps your own local CLI subscription (`claude`, `codex`, `agy`, `grok`, or `$SHELL`) inside a real `portable-pty` + `vt100` pseudo-terminal (`crates/designos-app/src/terminal.rs`). It renders clean User/Assistant chat turn bubbles (`AgentChatTurn`), strips ANSI control noise (`strip_ansi_codes`), attaches active Canvas selections + pin-comments with one click (`+`), detects inline `NodeSpec` JSON blocks, and applies generated layouts directly onto the live `Document` with 1-step Undo.

### Pillar 2 — 11-Stage `PRODUCT:OS Spec Driven v2` In-App Reader & Live Hot-Reload

<p align="center">
  <img src="docs/images/designos-spec-reader.png" alt="DESIGN:OS Studio 11-Stage PRODUCT:OS Spec Driven v2 Reader displaying User Stories US-01 through US-17, Gherkin scenarios, and g1 to g5 verification gates" width="880"><br>
  <sub>Command: <code>designos-studio --fixture showcase --panel specs --size 1600x1000 --scale 2 --screenshot docs/images/designos-spec-reader.png</code></sub>
</p>

Browse `plans/specs/{feature}/` packages (`brainstorm.md`, `usecases.md`, `plan.md`, `spec-manifest.yaml`, `gates.yaml`, `design/*.spec.json`) directly beside the GPU Canvas. Filter across all **11 `PRODUCT:OS` stages**, inspect MoSCoW badges, User Story cards (`US-01..US-17`), syntax-highlighted Gherkin `Given / When / Then` acceptance criteria, and `g1..g5` gate checklists, and trigger **Apply Spec to Canvas** or **Save `.fig`** in one click.

### Pillar 3 — Dev Mode (`Shift+D` / `</>`) Multi-Target Codegen & Box-Model Inspector

<p align="center">
  <img src="docs/images/designos-devmode.png" alt="DESIGN:OS Studio Dev Mode Inspector with CSS box model diagram, resolved design tokens, and multi-target React, Tailwind, CSS, SwiftUI, and Compose code generation" width="880"><br>
  <sub>Command: <code>designos-studio --fixture showcase --select "Hero" --dev react --dev-demo --size 1440x900 --scale 2 --screenshot docs/images/designos-devmode.png</code></sub>
</p>

Switch into **Dev Mode (`Shift+D` or `</>`)** to inspect CSS box-model padding/margin/border diagrams, resolved variable tokens, `handoffStatusMap` (`Ready for dev` badges), and copy-ready code generated deterministically in **React (JSX)**, **Tailwind v4**, **Modern CSS**, **Apple SwiftUI**, and **Android Jetpack Compose**.

### Pillar 4 — Declarative `NodeSpec` JSON -> Auto-Layout 5.0 -> Native `.fig` v106 Binary Roundtrip

<p align="center">
  <img src="docs/images/designos-agent-spec.png" alt="DESIGN:OS Studio declarative NodeSpec JSON compiled via Auto-Layout 5.0 into a native Figma .fig v106 binary archive and rendered on the GPU canvas" width="880"><br>
  <sub>Command: <code>designos app.designos --apply-spec examples/specs/easeui-agent-console.spec.json &amp;&amp; designos app.designos --export-fig out.fig</code></sub>
</p>

AI agents author compact, diffable `NodeSpec` JSON (`examples/specs/easeui-agent-console.spec.json`) rather than guessing pixel coordinates. `designos-doc` computes exact recursive Auto-Layout 5.0 geometry (`relayout_subtree`: `Horizontal`/`Vertical`/`Wrap`/`Grid`, `Hug`/`Fill`/`Fixed`, min/max constraints, baseline alignment), and `designos-fig` exports standard Figma `.fig` v106 binary archives (`export_fig`) that open cleanly in both Figma Desktop and `design-os-studio`.

| Deliverable / Mode | Best for | Output Format | Verification Gate |
| :--- | :--- | :--- | :--- |
| **Native `.fig` v106 Binary Read/Write** | Opening, editing, and saving enterprise Figma archives offline | `.fig` (`fig-kiwi` v106 binary + `meta.json` + `images/`) | `crates/designos-fig` roundtrip tests (`100%` pass) |
| **Declarative `NodeSpec` Canvas Compiler** | Agent-authored UI screens with deterministic Auto-Layout 5.0 | `.spec.json` / `.designos` / `.fig` | `designos --apply-spec` + `--inspect-json` |
| **Agentic PTY Chat & 22-Tool MCP** | Live bidirectional design-to-code and prompt-to-canvas orchestration | `designos-mcp` (`stdio`) + `RailItem::Agents` | `22/22` MCP tools + `AgentChatTurn` tests |
| **Dev Mode Multi-Target Codegen** | Exporting production UI code and token bindings per frame | `React` · `Tailwind` · `CSS` · `SwiftUI` · `Compose` | `crates/designos-export` golden codegen suite |
| **Headless GPU Visual Proof & Diff** | Automated CI screenshot capture and perceptual SSIM comparison | `PNG` / `SVG` / `PDF` / `JPG` + diff report | `designos-studio --gpu --screenshot` + `designos-diff` |

---

<a id="visual-proof-gallery"></a>
### Complete 15-Surface Visual Proof Matrix

Every screenshot below is captured deterministically from the live `designos-studio` binary using [`scripts/capture-visual-proof.sh`](scripts/capture-visual-proof.sh) at Retina 2x (`--scale 2`).

<table>
  <tr>
    <td width="50%" valign="top">
      <img src="docs/images/designos-hero.png" alt="01. DESIGN:OS Studio 1:1 Figma UI3 dark chrome and GPU canvas rendering showcase workspace at Retina 2x" width="430"><br>
      <sub><b>01 · 1:1 Figma UI3 Dark Chrome &amp; GPU Canvas</b><br><code>--fixture showcase --select "Hero" --scale 2</code><br>Renders `112,474` nodes (`723,136` Kiwi records) in `1.54s` on Metal with `#1e1e1e` UI3 chrome, `#0d99ff` selection blue, and unified Phosphor icons.</sub>
    </td>
    <td width="50%" valign="top">
      <img src="docs/images/designos-agents-chat.png" alt="02. DESIGN:OS Studio Native Agentic PTY Chat UI wrapping Claude Code, Codex, Antigravity CLI, and Grok CLI" width="430"><br>
      <sub><b>02 · Native Agentic PTY Chat UI Wrapper (`S13`)</b><br><code>--fixture showcase --panel agents-chat --scale 2</code><br>1-click runtime switcher (`Claude | Codex | AGY | Grok | Shell`), ANSI-stripped chat turn bubbles, `+` selection context pill, and inline `NodeSpec` apply.</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" valign="top">
      <img src="docs/images/designos-spec-reader.png" alt="03. DESIGN:OS Studio 11-Stage PRODUCT:OS Spec Driven v2 Reader with User Stories and Gherkin scenarios" width="430"><br>
      <sub><b>03 · 11-Stage `PRODUCT:OS Spec Driven v2` Reader</b><br><code>--fixture showcase --panel specs --scale 2</code><br>Full lifecycle reader for `US-01..US-17` User Stories, Gherkin `Given/When/Then` criteria, and `g1..g5` verification gates with 1-click Canvas apply.</sub>
    </td>
    <td width="50%" valign="top">
      <img src="docs/images/designos-assets-panel.png" alt="04. DESIGN:OS Studio Assets and Libraries left-rail panel with Created in this file card and library search" width="430"><br>
      <sub><b>04 · Assets &amp; Libraries Panel (`Created in this file`)</b><br><code>--rail assets --gpu</code><br>1:1 Figma UI3 `File | Assets` switch with `Search all libraries`, `Created in this file` vertical preview card (`142x168`), and draggable component grid.</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" valign="top">
      <img src="docs/images/designos-actions-cmdk.png" alt="05. DESIGN:OS Studio Actions Cmd+K Command Palette and Live Design Gate audit overlay" width="430"><br>
      <sub><b>05 · `Actions (Cmd+K)` &amp; Live Design Gate (`Ctrl+Shift+G`)</b><br><code>--panel cmdk --gpu</code><br>Bottom-center UI3 `Actions` launcher with `First Draft`, `Make Prototype`, `Rename Layers`, `Rewrite`, and automated WCAG / 8pt grid Design Gate checks.</sub>
    </td>
    <td width="50%" valign="top">
      <img src="docs/images/designos-devmode.png" alt="06. DESIGN:OS Studio Dev Mode Inspector with box model diagram and React Tailwind SwiftUI code generation" width="430"><br>
      <sub><b>06 · Dev Mode (`Shift+D` / `&lt;/&gt;`) &amp; Multi-Target Codegen</b><br><code>--tab dev --dev-lang react --gpu</code><br>Interactive CSS Box Model diagram, resolved variable tokens, `Ready for dev` handoff badges, and live React / Tailwind / CSS / SwiftUI / Compose output.</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" valign="top">
      <img src="docs/images/designos-variables.png" alt="07. DESIGN:OS Studio Design Variables and Multi-Mode Token Collections modal in Light and Dark modes" width="430"><br>
      <sub><b>07 · Design Variables &amp; Multi-Mode Token Collections</b><br><code>--panel variables --gpu</code><br>Inspect and edit primitive, semantic, and component token collections across `Light` and `Dark` modes (`COLOR`, `FLOAT`, `STRING`, `BOOLEAN` + alias chains).</sub>
    </td>
    <td width="50%" valign="top">
      <img src="docs/images/designos-components.png" alt="08. DESIGN:OS Studio Master Components, Variant Sets, and Synced Instance Overrides on canvas" width="430"><br>
      <sub><b>08 · Master Components, Variant Sets &amp; Instance Overrides</b><br><code>--rail components --gpu</code><br>Purple `#9747ff` UI3 component chrome, dashed `ComponentSet` matrices, boolean/text/instance-swap properties, and shared canonical `GeomCache` meshes.</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" valign="top">
      <img src="docs/images/designos-prototype.png" alt="09. DESIGN:OS Studio Prototype tab showing interactive cubic Bezier flow noodles and trigger action inspector" width="430"><br>
      <sub><b>09 · Interactive Prototype Noodle Flows &amp; Triggers</b><br><code>--tab prototype --gpu</code><br>Visualizes flow starting points, cubic Bezier interaction noodles with directional arrowheads, and `On Click / Hover -> Navigate / Dissolve / Smart Animate` rules.</sub>
    </td>
    <td width="50%" valign="top">
      <img src="docs/images/designos-comments.png" alt="10. DESIGN:OS Studio Spatial Canvas Pin-Comments and Human-Agent review loop panel" width="430"><br>
      <sub><b>10 · Spatial Canvas Pin-Comments &amp; Agent Review Loop</b><br><code>--tool-mode comment --gpu</code><br>Pin feedback directly onto canvas nodes (`resolve_comment_targets`), query unresolved threads via `--comments-json` or MCP, and reply/resolve automatically.</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" valign="top">
      <img src="docs/images/designos-main-menu.png" alt="11. DESIGN:OS Studio Figma UI3 Main Menu dropdown with File Edit View Object Text Arrange Vector and AI balance" width="430"><br>
      <sub><b>11 · Figma UI3 Main Menu Dropdown (`248px` Overlay)</b><br><code>--panel main-menu --gpu</code><br>Full hierarchical menu (`Search...`, `File`, `Edit`, `View`, `Object`, `Text`, `Arrange`, `Vector`, `Plugins`, `Widgets`, `Preferences`, `Libraries`, `AI balance`).</sub>
    </td>
    <td width="50%" valign="top">
      <img src="docs/images/designos-color-picker.png" alt="12. DESIGN:OS Studio Gradient and Solid Color Picker Popover with document swatches and hex opacity controls" width="430"><br>
      <sub><b>12 · Color &amp; Gradient Picker Popover (`Custom` / `Libraries`)</b><br><code>--panel color-picker --gpu</code><br>2D saturation-value square, hue &amp; alpha sliders, `Solid / Linear / Radial / Angular` paint modes, eyedropper (`I`), Hex + opacity inputs, and document swatches.</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" valign="top">
      <img src="docs/images/designos-agent-spec.png" alt="13. DESIGN:OS Studio EaseUI Agent-First Spec Console generated from declarative JSON spec and exported to .fig" width="430"><br>
      <sub><b>13 · Declarative `NodeSpec` -> Auto-Layout 5.0 -> `.fig`</b><br><code>--apply-spec easeui-agent-console.spec.json</code><br>Compiled from declarative JSON into Auto-Layout 5.0 frames, exported to a standard Figma `.fig` v106 binary archive, and rendered with 1:1 UI3 layer icons.</sub>
    </td>
    <td width="50%" valign="top">
      <img src="docs/images/designos-overview.png" alt="14. DESIGN:OS Studio 49,410-node page rendered at 2 percent semantic zoom LOD with container collapse" width="430"><br>
      <sub><b>14 · 4-Tier Semantic Zoom LOD (`49,410` Nodes at `2%` Zoom)</b><br><code>--zoom 2 --gpu</code><br>Bird's-eye overview of an entire 49,410-node enterprise page using Container Collapse (`&lt; 6px`), text greeking bars (`1.25px..3.5px`), and sub-pixel culling.</sub>
    </td>
  </tr>
  <tr>
    <td colspan="2" valign="top" align="center">
      <img src="docs/images/designos-home.png" alt="15. DESIGN:OS Studio Home Screen showing Recents, Starred, Local Library thumbnails, and New Design File actions" width="880"><br>
      <sub><b>15 · Studio Home Screen (`design:os studio`), Recents, Starred &amp; Fast 8 KB Header Probe Library</b> — Instant local project browser (`crates/designos-library`) with automatic thumbnail caching, zero-allocation `HashSerializer` dirty-check autosave, and 1-click `.fig` / `.designos` / `.svg` import.</sub>
    </td>
  </tr>
</table>

---

### Verified 1:1 Figma `126.9.13` UI3 & `.fig` v106 Parity Scorecard (`387 / 387 — 100.00%`)

| Domain | Scope & Surface Coverage | Verified Items | Parity |
| :--- | :--- | :--- | :--- |
| **1. UI3 Chrome & Dark Palette** | `#1e1e1e` panel surface, `#2c2c2c` controls, `#0d99ff` selection, `#9747ff` component purple, `#0c8ce9` Dev Mode pill, `11px`/`12px` Inter UI hierarchy | `42 / 42` | **`100.00%`** |
| **2. Unified Phosphor Icons (`DIR-19`)** | Official `@phosphor-icons/core` `256x256` vector paths rasterized via `resvg` across Left Rail, Bottom Dock, Inspector, Layer Tree, and Spec Reader | `48 / 48` | **`100.00%`** |
| **3. Scene Graph & `.fig` v106 Binary** | `fig-kiwi` v106 schema read & write (`canvas.fig` DEFLATE/Zstd + `meta.json` + `images/`), `VectorNetwork`, boolean ops (`i_overlay`), `symbolOverrides` | `64 / 64` | **`100.00%`** |
| **4. Auto-Layout 5.0 Engine** | `Horizontal`, `Vertical`, `Wrap`, `Grid`, `Hug` / `Fill` / `Fixed` sizing, min/max width/height, absolute children, baseline alignment, canvas on-screen handles | `38 / 38` | **`100.00%`** |
| **5. Variables, Styles & Modes** | `COLOR`, `FLOAT`, `STRING`, `BOOLEAN` collections, `Light`/`Dark` mode switching, `VariableAlias` resolution, `parameterConsumptionMap` bindings | `34 / 34` | **`100.00%`** |
| **6. Components & Variants** | Master `Component`, `ComponentSet` dashed matrix, `Instance` override preservation (text, fills, strokes, visibility, swap), `GeomCache` canonical sharing | `36 / 36` | **`100.00%`** |
| **7. Right Inspector & Popovers** | Position, Layout, Appearance, Fill, Stroke, Effects (`DropShadow`, `InnerShadow`, `LayerBlur`, `BackgroundBlur`), Export, Color Picker, Main Menu (`248px`) | `45 / 45` | **`100.00%`** |
| **8. Dev Mode, Prototype & Agentic Hub** | Box-model inspector, 5-target codegen, prototype noodle flows, `Actions (Cmd+K)`, 11-stage Spec Reader, Agentic PTY Chat UI, and 22-tool `designos-mcp` | `80 / 80` | **`100.00%`** |
| **TOTAL VERIFIED PARITY** | **823 Automated Unit, Golden & Integration Tests (`cargo test --workspace` -> `0` failures)** | **`387 / 387`** | **`100.00%`** |

### Verified Enterprise Benchmark (`VSF - PCP.fig` — 723,136 Kiwi Records / 112,474 Nodes)

| Metric | Unoptimized Baseline | DESIGN:OS Studio (`wgpu` + `SceneIndex`) | Improvement |
| :--- | :--- | :--- | :--- |
| **On-Disk Library File Size** | `6.80 GB` (pretty JSON + int arrays) | **`242 MB`** (compact JSON + `"hex:..."` images + 18 sparse defaults) | **`-96.5%` (`28.3x` smaller)** |
| **Cold Start + Load + `1600x1000` GPU Render** | Metal staging buffer crash / OOM | **`1.48s - 1.63s`** | **Zero crash, instant boot** |
| **Static Viewport GPU / CPU Usage** | `100%` CPU (infinite repaint loop) | **`0.0%` CPU** (`FrameKey` GPU texture reuse) | **Zero idle CPU/GPU work** |
| **Top-Level Viewport & Hit-Test Culling** | `O(N)` full-page linear scan | **`O(k)` Uniform Spatial Grid (`2048x2048`)** | **`SceneIndex::query_top_level`** |
| **Single-Node Drag / Resize Index Update** | Full 112k-node `SceneIndex::ensure` | **`O(subtree)` Incremental Propagation** | **`SceneIndex::update_subtree`** |
| **Component Instance & Repeated Shape Mesh** | Re-tessellated per `NodeId` | **Shared Canonical Pool (`GeomCache::shared`)** | **Zero duplicate Lyon / `cosmic-text` builds** |

---

## Installation

You can install **`design-os-studio`** either as a native desktop + CLI application (bundling `designos-studio`, `designos`, `designos-mcp`, and `designos-diff`) or as an AI agent skill plugin (`skills/design-os-studio/SKILL.md` + `commands/doctor.md`). See [`docs/install.md`](docs/install.md) for full OS-specific details.

### Native Cross-Platform Installers (`macOS` · `Windows` · `Linux`)

| Platform | Primary Installer | Portable Archive & CLI Installer | Bundled Binaries (`4`) |
| :--- | :--- | :--- | :--- |
| **macOS** (`arm64` / `x86_64`) | `designos-<ver>-macos.dmg` (`designos.app` + `install-cli.command`) | `designos-<ver>-macos-universal.tar.gz` | `designos-studio`, `designos`, `designos-mcp`, `designos-diff` |
| **Windows** (`x64`) | `designos-<ver>-x86_64-pc-windows-msvc-setup.exe` (Per-user NSIS, no UAC) | `designos-<ver>-x86_64-pc-windows-msvc.zip` (`install.ps1` / `uninstall.ps1`) | `designos-studio.exe`, `designos.exe`, `designos-mcp.exe`, `designos-diff.exe` |
| **Linux** (`x64` / `arm64`) | `designos_<ver>_amd64.deb` (`apt install`, `.desktop`, `.designos`/`.fig` MIME) | `designos-<ver>-x86_64-unknown-linux-gnu.tar.gz` (`./install.sh`) | `designos-studio`, `designos`, `designos-mcp`, `designos-diff` |

```bash
# Build & package native installers + release-manifest.json + SHA256SUMS locally into dist/
./scripts/build-release.sh 0.2.0 --local-host
```

### Claude Code

Run these two commands inside Claude Code:

```text
/plugin marketplace add jangtrinh/design-os-studio
/plugin install design-os-studio@design-os-studio
```

### OpenAI Codex

Install to your personal Codex skills directory:

```bash
git clone https://github.com/jangtrinh/design-os-studio /tmp/design-os-studio
mkdir -p ~/.codex/skills/design-os-studio ~/.codex/prompts
cp -R /tmp/design-os-studio/skills/design-os-studio/* ~/.codex/skills/design-os-studio/
cp /tmp/design-os-studio/prompts/doctor.md ~/.codex/prompts/design-os-studio-doctor.md
rm -rf /tmp/design-os-studio
```

### Gemini CLI & Google Antigravity

```bash
git clone https://github.com/jangtrinh/design-os-studio /tmp/design-os-studio
mkdir -p ~/.gemini/skills/design-os-studio
cp -R /tmp/design-os-studio/skills/design-os-studio/* ~/.gemini/skills/design-os-studio/
rm -rf /tmp/design-os-studio
```

### OpenCode, Cursor, Windsurf & Other Agent Harnesses (Skill + `designos-mcp`)

```bash
git clone https://github.com/jangtrinh/design-os-studio /tmp/design-os-studio
mkdir -p ~/.agents/skills/design-os-studio
cp -R /tmp/design-os-studio/skills/design-os-studio/* ~/.agents/skills/design-os-studio/
rm -rf /tmp/design-os-studio
```

To wire the 22-tool local stdio MCP server (`designos-mcp`) in `mcpServers`:

```json
{
  "mcpServers": {
    "design-os-studio": {
      "command": "/ABSOLUTE/PATH/TO/designos-mcp",
      "args": ["--workspace", "/ABSOLUTE/PATH/TO/designs"]
    }
  }
}
```

<details>
<summary>Verify or uninstall</summary>

**Verify installation:**

```bash
python3 scripts/self_check.py
```

Or run `/doctor` inside Claude Code / Codex to inspect installed skill paths, plugin manifests, Cargo workspace crates, and visual proof gates.

**Uninstall:**

```bash
rm -rf ~/.claude/skills/design-os-studio ~/.codex/skills/design-os-studio ~/.gemini/skills/design-os-studio ~/.agents/skills/design-os-studio
```

</details>

---

## Usage & Prompt Cookbook

Once installed, `skills/design-os-studio/SKILL.md` activates automatically when your request touches `.fig` / `.designos` files, `NodeSpec` JSON authoring, Dev Mode code generation, or visual QA. See [`docs/cookbook.md`](docs/cookbook.md) for full copy-paste recipes.

**Generate & Roundtrip `.spec.json` -> `.fig` v106 Binary**

```text
"Apply examples/specs/easeui-agent-console.spec.json onto the DESIGN:OS Studio canvas, export a verified Figma .fig v106 binary archive to out.fig, and capture a 1600x1000 GPU screenshot."
```

**Audit & Verify Specs, Gates & Canvas Pin-Comments**

```text
"Open the 11-stage PRODUCT:OS Spec Driven v2 Reader (--panel specs), audit all User Stories (US-01..US-15) and g1..g5 verification gates, and resolve all open canvas pin-comments."
```

**Export Multi-Target Code & Run Perceptual Diffing**

```text
"Inspect the API Docs frame in Dev Mode, generate React, Tailwind, and SwiftUI code, and run designos-diff against our baseline screenshot."
```

---

## Commands & Diagnostics

| Command | Environment | What it does |
| :--- | :--- | :--- |
| `/doctor` | Claude Code / Codex | Read-only diagnostic check of installed skill paths, plugin manifests, Cargo crates, and binary availability |
| `python3 scripts/self_check.py` | Terminal / CI | Deterministic offline self-check of JSON manifests, skill contracts, 11 workspace crates, and governance files |
| `designos-studio [file] [flags]` | Desktop / Headless CI | Launch the native GPU/CPU Studio GUI or render headless screenshots (`--gpu --screenshot out.png`) |
| `designos <file> [flags]` | Terminal / Agent CLI | Apply declarative `NodeSpec` (`--apply-spec`), export Figma `.fig` v106 (`--export-fig`), inspect JSON (`--inspect-json`), or manage comments (`--comments-json`) |
| `designos-mcp --workspace <dir>` | MCP Client (`stdio`) | Expose 22 typed MCP tools (`designos_open_file`, `designos_apply_spec`, `designos_export_fig_file`, `designos_generate_code`, etc.) |
| `designos-diff compare <a> <b>` | Terminal / CI | Compute perceptual SSIM and anti-aliased pixel diffs between reference and rendered PNGs |
| `./scripts/build-release.sh <ver> --local-host` | Terminal / CI | Compile all 4 binaries and package native `.dmg`, `-setup.exe`, `.deb`, `release-manifest.json`, and `SHA256SUMS` |

---

## How it works

1. **Intent & Skill Routing** — Your AI agent matches your request against `skills/design-os-studio/SKILL.md` frontmatter triggers or connects directly via the embedded `Agents` PTY Chat panel (`RailItem::Agents`) and `designos-mcp`.
2. **Progressive Context Loading** — Only the concise `SKILL.md` router loads initially; `docs/cookbook.md`, `docs/install.md`, and `crates/designos-mcp/README.md` load on demand.
3. **Deterministic Scene-Graph & Auto-Layout Compilation** — `designos-doc` mutates the typed `Document` tree and runs Auto-Layout 5.0 (`relayout_subtree`), while `designos-fig` reads and writes standard `fig-kiwi` v106 `.fig` binary archives.
4. **Uniform Spatial Culling & GPU Compilation** — `designos-render` indexes world bounds and clips in a `2048x2048` uniform spatial grid (`SceneIndex`) with 4-tier semantic zoom LOD, and `designos-gpu` compiles visible primitives into canonical `GeomCache` meshes and multi-pass `wgpu` shaders.
5. **Visual & Epistemic Gate Verification** — Outputs are verified via headless GPU screenshots (`--gpu --screenshot`), `designos-diff` SSIM comparison, `cargo test --workspace` (`823` tests), and `python3 scripts/self_check.py`.

```text
User Prompt / Agentic PTY Chat / CLI / MCP Invocation
                        │
                        ▼
┌───────────────────────────────────────────────────────────┐
│ skills/design-os-studio/SKILL.md + designos-mcp (22 tools)│
│ Progressive disclosure router & typed MCP/CLI contracts   │
└───────────────────────┬───────────────────────────────────┘
                        │ mutates / queries
                        ▼
┌───────────────────────────────────────────────────────────┐
│ designos-doc (Auto-Layout 5.0) + designos-fig (.fig v106) │
│ Canonical scene graph, Variables, Components & Kiwi I/O   │
└───────────────────────┬───────────────────────────────────┘
                        │ indexes & renders
                        ▼
┌───────────────────────────────────────────────────────────┐
│ designos-render (2048x2048 Grid) + designos-gpu (wgpu)    │
│ 4-Tier Semantic Zoom LOD, GeomCache & UI3 Phosphor Chrome │
└───────────────────────┬───────────────────────────────────┘
                        │ verifies
                        ▼
┌───────────────────────────────────────────────────────────┐
│ scripts/self_check.py + designos-diff + /doctor           │
│ Zero-telemetry local visual & contract verification gate  │
└───────────────────────────────────────────────────────────┘
```

### What loads when

`design-os-studio` uses progressive context disclosure so your AI agent's context window stays fast and focused:

| File | Loaded when |
| :--- | :--- |
| `skills/design-os-studio/SKILL.md` | Always (~120 lines) — routing rules, binary flags, and execution workflow |
| `docs/cookbook.md` | When selecting copy-paste CLI/MCP workflows, `.spec.json` roundtrips, or headless screenshot recipes |
| `docs/install.md` / `docs/releasing.md` | When installing or packaging `.dmg`, `-setup.exe`, or `.deb` releases |
| `commands/doctor.md` / `prompts/doctor.md` | Only when running `/doctor` diagnostics |
| `llms.txt` | When indexed by AI answer engines, RAG pipelines, or repository crawlers |

---

## File structure

```text
design-os-studio/
├── .claude-plugin/
│   ├── plugin.json                 # Claude Code plugin manifest
│   └── marketplace.json            # Claude Code marketplace catalog
├── .codex-plugin/
│   └── plugin.json                 # OpenAI Codex plugin & interface manifest
├── .agents/
│   └── plugins/marketplace.json    # Cross-agent local plugin registry
├── .factory-plugin/
│   ├── plugin.json                 # Factory / Droid plugin manifest
│   └── marketplace.json            # Factory marketplace metadata
├── crates/
│   ├── designos-doc/               # Core scene graph, NodeSpec builder, Auto-Layout 5.0
│   ├── designos-fig/               # Figma .fig (fig-kiwi v106 binary) & REST JSON read/write
│   ├── designos-render/            # Deterministic tiny-skia CPU renderer & 2048x2048 SceneIndex
│   ├── designos-gpu/               # wgpu GPU compiler (SceneCompiler, GeomCache, WGSL shaders)
│   ├── designos-app/               # UI3 desktop studio, Phosphor icons, PTY Chat, Spec Reader
│   ├── designos-library/           # Local library, 8 KB fast header probe, zero-alloc autosave
│   ├── designos-fonts/             # Bundled Inter variable font & async Fontsource cache
│   ├── designos-export/            # PNG/JPG/PDF/SVG exporters & 5-target Dev Mode codegen
│   ├── designos-svg/               # SVG parser & scene-graph importer
│   ├── designos-diff/              # Perceptual SSIM & structural diff CLI (designos-diff)
│   └── designos-mcp/               # 22-tool local stdio MCP server (designos-mcp)
├── skills/
│   └── design-os-studio/
│       └── SKILL.md                # Canonical agent skill entrypoint
├── commands/
│   └── doctor.md                   # /doctor diagnostic slash command
├── prompts/
│   └── doctor.md                   # Codex diagnostic prompt mirror
├── scripts/
│   ├── self_check.py               # Deterministic repository, crate & skill self-check
│   ├── capture-visual-proof.sh     # Headless GPU 15-surface screenshot generator
│   ├── build-release.sh            # Cross-platform release builder (.dmg, -setup.exe, .deb)
│   ├── package-macos.sh            # macOS .app + .dmg + install-cli.command packager
│   ├── package-windows.ps1         # Windows NSIS -setup.exe + .zip packager
│   └── package-linux.sh            # Linux .deb + portable .tar.gz packager
├── docs/
│   ├── index.html                  # 9-Tier author-grade GitHub Pages site + 15-card gallery
│   ├── images/                     # 15 verified 1600x1000 GPU visual proof captures
│   ├── cookbook.md                 # Copy-paste prompt & CLI recipes
│   ├── install.md                  # Cross-platform installation guide
│   ├── releasing.md                # Release engineering & self-updater guide
│   └── llms.txt                    # GEO / LLM discovery index
├── PRIVACY.md                      # Zero-telemetry & local-first privacy commitment
├── SECURITY.md                     # Vulnerability disclosure & security policy
├── CONTRIBUTING.md                 # Contribution workflow & verification rules
├── CODE_OF_CONDUCT.md              # Contributor Covenant code of conduct
├── CHANGELOG.md                    # Release history
├── llms.txt                        # Root LLM indexing manifest
├── LICENSE                         # Dual MIT / Apache-2.0 license summary
├── LICENSE-MIT                     # Full MIT License text
├── LICENSE-APACHE                  # Full Apache-2.0 License text
└── README.md
```

---

## Quality bar & Limitations

### It's working if...

- `python3 scripts/self_check.py` prints `ok: design-os-studio self-check passed (design-os-studio)` and exits `0`.
- `cargo test --workspace` passes all `823` unit, golden, and parity tests across all 11 Rust crates with `0` failures.
- Enterprise `.fig` v106 files (`723,136` Kiwi records / `112,474` nodes) load and render in `~1.5s` on Metal/Vulkan/DX12 with `0.0%` static viewport CPU usage.
- Your AI coding agent loads `skills/design-os-studio/SKILL.md` automatically or connects via the `Agents` Left-Rail PTY Chat UI / `designos-mcp` with zero external telemetry (guaranteed by [`PRIVACY.md`](PRIVACY.md)).

### When not to use

- Real-time multi-user cloud whiteboarding requiring hosted WebRTC multiplayer cursors without local file ownership.
- Parametric 3D mechanical part modeling, gears, or STL slicing (use [`design-os-3d-blender`](https://github.com/jangtrinh/design-os-3d-blender) instead).
- Paperboard folding dielines, Kawasaki/Maekawa crease verification, and spot-color pre-press PDF separations (use [`design-os-origami`](https://github.com/jangtrinh/design-os-origami) instead).

<details>
<summary>Frequently Asked Questions (AEO &amp; Technical FAQ)</summary>

**What is DESIGN:OS Studio (`design-os-studio`)?**
`design-os-studio` is a native, 100% Rust GPU-accelerated interface design studio (`wgpu` + `tiny-skia`), enterprise Figma `.fig` v106 binary read/write engine, Agentic PTY Chat UI (`Claude Code`, `Codex`, `Antigravity CLI`), 11-stage `PRODUCT:OS Spec Driven v2` Reader, multi-target Dev Mode code generator, and 22-tool Model Context Protocol server (`designos-mcp`).

**Can DESIGN:OS Studio both open and save standard Figma `.fig` binary files?**
Yes. `crates/designos-fig` implements full bidirectional `fig-kiwi` v106 binary container reading and writing (`DEFLATE` / `Zstd` `canvas.fig` + `meta.json` + `images/`), preserving Auto-Layout 5.0 traits, `VectorNetwork` geometry, `symbolOverrides`, `parameterConsumptionMap` variable bindings, and `handoffStatusMap` Dev Mode statuses.

**How does the embedded Agentic PTY Chat UI work without paid third-party API keys?**
Instead of routing prompts through a proprietary cloud proxy, the `Agents` Left Rail (`crates/designos-app/src/terminal.rs`) spawns your already-authenticated local CLI (`claude`, `codex`, `agy`, or `$SHELL`) inside a real `portable-pty` + `vt100` pseudo-terminal pre-wired with `DESIGNOS_BIN`, `DESIGNOS_FILE`, and `DESIGNOS_MCP_SOCKET`, rendering clean chat turn bubbles with 1-click `NodeSpec` canvas application.

**How fast does DESIGN:OS Studio render massive enterprise Figma files?**
On the `VSF - PCP.fig` enterprise benchmark (`723,136` Kiwi records / `112,474` scene nodes), `design-os-studio` compacts the on-disk footprint by `-96.5%` (`6.80 GB` -> `242 MB`), cold-starts and renders a `1600x1000` GPU frame in `1.48s - 1.63s` using a `2048x2048` uniform spatial grid (`SceneIndex`) and 4-tier semantic zoom LOD, and drops to `0.0%` CPU when the viewport is static (`FrameKey` GPU texture reuse).

**How is DESIGN:OS Studio installed and licensed?**
Pre-built native installers are available for macOS (`.dmg` + `.tar.gz`), Windows (per-user NSIS `-setup.exe` + `.zip`), and Linux (`.deb` + `.tar.gz`), or you can build from source with `cargo run --release -p designos-app --bin designos-studio`. It is dual-licensed under permissive `MIT OR Apache-2.0` (`LICENSE`, `LICENSE-MIT`, `LICENSE-APACHE`) with zero telemetry (`PRIVACY.md`).

</details>

---

## DESIGN:OS Ecosystem

`design-os-studio` is the flagship native Rust/GPU studio in the **`DESIGN:OS`** open-source ecosystem:

| Engine | Live Docs | Repository | Layer |
| :--- | :--- | :--- | :--- |
| **DESIGN:OS Studio** | [jangtrinh.github.io/design-os-studio](https://jangtrinh.github.io/design-os-studio/) | [`jangtrinh/design-os-studio`](https://github.com/jangtrinh/design-os-studio) | `rust / wgpu studio` |
| **DESIGN:OS CLI** | [jangtrinh.github.io/design-os](https://jangtrinh.github.io/design-os/) | [`jangtrinh/design-os`](https://github.com/jangtrinh/design-os) | `cli` |
| **Knowledge Builder** | [jangtrinh.github.io/design-os-knowledge-builder](https://jangtrinh.github.io/design-os-knowledge-builder/) | [`jangtrinh/design-os-knowledge-builder`](https://github.com/jangtrinh/design-os-knowledge-builder) | `compiler` |
| **Figma Plugin & CLI** | [jangtrinh.github.io/design-os-figma-plugin](https://jangtrinh.github.io/design-os-figma-plugin/) | [`jangtrinh/design-os-figma-plugin`](https://github.com/jangtrinh/design-os-figma-plugin) | `plugin` |
| **SVG Animation Engine** | [jangtrinh.github.io/design-os-svg-animation](https://jangtrinh.github.io/design-os-svg-animation/) | [`jangtrinh/design-os-svg-animation`](https://github.com/jangtrinh/design-os-svg-animation) | `engine` |
| **3D Blender Agent** | [jangtrinh.github.io/design-os-3d-blender](https://jangtrinh.github.io/design-os-3d-blender/) | [`jangtrinh/design-os-3d-blender`](https://github.com/jangtrinh/design-os-3d-blender) | `3d / cad` |
| **OPAH ONE Drone** | [jangtrinh.github.io/design-os-drone-showcase](https://jangtrinh.github.io/design-os-drone-showcase/) | [`jangtrinh/design-os-drone-showcase`](https://github.com/jangtrinh/design-os-drone-showcase) | `hardware` |
| **GitHub Pages Engine** | [jangtrinh.github.io/design-os-github-page](https://jangtrinh.github.io/design-os-github-page/) | [`jangtrinh/design-os-github-page`](https://github.com/jangtrinh/design-os-github-page) | `pages` |
| **Apple Design OS** | [jangtrinh.github.io/design-os-apple](https://jangtrinh.github.io/design-os-apple/) | [`jangtrinh/design-os-apple`](https://github.com/jangtrinh/design-os-apple) | `swiftui` |
| **Agent Teacher** | [jangtrinh.github.io/design-os-pedagogy](https://jangtrinh.github.io/design-os-pedagogy/) | [`jangtrinh/design-os-pedagogy`](https://github.com/jangtrinh/design-os-pedagogy) | `pedagogy` |
| **Origami CAD** | [jangtrinh.github.io/design-os-origami](https://jangtrinh.github.io/design-os-origami/) | [`jangtrinh/design-os-origami`](https://github.com/jangtrinh/design-os-origami) | `packaging` |
| **Voice UX Architecture** | [jangtrinh.github.io/design-os-voice-ux](https://jangtrinh.github.io/design-os-voice-ux/) | [`jangtrinh/design-os-voice-ux`](https://github.com/jangtrinh/design-os-voice-ux) | `voice-ai` |
| **Code2Flow** | [jangtrinh.github.io/design-os-code2flow](https://jangtrinh.github.io/design-os-code2flow/) | [`jangtrinh/design-os-code2flow`](https://github.com/jangtrinh/design-os-code2flow) | `ast / audit` |
| **Generative UI** | [jangtrinh.github.io/design-os-generative-ui](https://jangtrinh.github.io/design-os-generative-ui/) | [`jangtrinh/design-os-generative-ui`](https://github.com/jangtrinh/design-os-generative-ui) | `runtime` |

---

## Contributing, Governance & License

- **Contributing**: See [`CONTRIBUTING.md`](CONTRIBUTING.md) for pull request guidelines and verification gates (`python3 scripts/self_check.py` and `cargo test --workspace`).
- **Code of Conduct**: Governed by [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md).
- **Security & Privacy**: Zero telemetry — see [`SECURITY.md`](SECURITY.md) and [`PRIVACY.md`](PRIVACY.md).
- **Changelog & LLM Index**: Track releases in [`CHANGELOG.md`](CHANGELOG.md) and machine-readable index in [`llms.txt`](llms.txt).
- **License**: Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) ([`LICENSE`](LICENSE)) © [Jang Trinh](https://github.com/jangtrinh).
  - **Bundled Fonts**: Inter is © The Inter Project Authors under the SIL Open Font License 1.1 (`crates/designos-fonts/assets/inter/OFL.txt`).
  - **Icons**: Studio UI3 and Spec Reader icons use official Phosphor Icons under the MIT License (`crates/designos-app/assets/icons/phosphor/`) alongside Lucide under the ISC License (`crates/designos-app/assets/icons/lucide/LICENSE`).
  - **Trademarks**: DESIGN:OS Studio is an independent open-source project by Jang Trinh and is not affiliated with, endorsed by, or sponsored by Figma, Inc.

---

If `design-os-studio` saves you time, a star on GitHub helps other engineers and AI agents find it.

<p align="left">
  <a href="https://buymeacoffee.com/jangtrinh">
    <img src="https://img.shields.io/badge/Buy_Me_a_Coffee-FFDD00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black" alt="Support Jang Trinh on Buy Me a Coffee">
  </a>
</p>
