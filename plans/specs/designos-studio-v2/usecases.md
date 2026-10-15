---
id: usecases-figma-ai-terminal-core
type: usecases
title: "es:brainstorm Phase 2 — Use Cases, Gherkin ACs & UX Law Matrix (US-11: Figma AI Agent + PTY Terminal Core)"
status: Approved
updated: 2026-10-15
owner_role: Product Owner
trace:
  req: [REQ-13, REQ-05, REQ-07, REQ-12]
  us: [US-11, US-03, US-05, US-10]
  decisions: [D10, D11, D12, D13]
---

# Phase 2: Use Cases & Interaction Specifications — 1:1 Figma AI Agent Suite & Native PTY Terminal Core

> **Product:** **`design`:os studio** (always all-lowercase, bold **`design`**, regular `:os studio`)  
> **Primary Story:** [`US-11`](us/US-11.md) · **Screens:** `S03` (PTY Terminal), `S06` (`Actions ⌘K`), `S11` (AI Agent Hub & 5-Runtime Launcher)

---

## 1. Canonical Use Cases (`UC-01` .. `UC-07`)

### `UC-01`: Trigger "Make Design (First Draft)" from `Actions (⌘K)` or `Agents` Rail (`S06` / `S11`)
- **Actor**: Product Designer / Developer
- **Preconditions**: An active project tab is open in **`design`:os studio** (`S01`/`S10`).
- **Main Success Scenario**:
  1. User presses `Cmd+K` (opening bottom-docked `Actions` bar `S06` on the `AI` tab) or clicks the `Agents` icon on the `48px` Left Rail (`S11`).
  2. User selects **Make Design (First Draft)** and enters a prompt (e.g., *"SaaS Billing Dashboard with 2D CSS Grid KPI cards and dark UI3 tokens"*).
  3. **`design`:os studio** immediately pushes an Undo snapshot, synthesizes a multi-frame UI3/2D-Grid `NodeSpec` onto the canvas, AND pipes the structured prompt + `$DESIGNOS_LIVE_SPEC` path into the active PTY session (`claude` / `codex` / `agy` / `grok`).
  4. A bottom-right diff toast confirms: `First Draft applied (+18 nodes) · Piped to active PTY · Undo ⌘Z`.

### `UC-02`: Launch & Switch Between 5 First-Class CLI Runtimes (`claude`, `codex`, `agy`, `grok`, `$SHELL`) (`S03` / `S11`)
- **Actor**: AI Engineer / Fullstack Developer
- **Preconditions**: User opens the `Agents` Left Rail (`S11`) or bottom PTY Terminal dock (`S03`).
- **Main Success Scenario**:
  1. System probes `$PATH` once on startup/refresh and displays the 5 CLI Runtime Preset cards (`Claude Code` `claude`, `Codex` `codex`, `Antigravity` `agy`, `Grok` `grok`, `$SHELL` `zsh`/`bash`) with live `INSTALLED` or `FALLBACK SHELL` badges.
  2. User clicks `+ Claude Code` (or `Codex` / `Antigravity` / `Grok`).
  3. System spawns a native `portable-pty` session pre-configured with `$DESIGNOS_BIN`, `$DESIGNOS_FILE`, `$DESIGNOS_MCP_BIN`, `$DESIGNOS_LIVE_SPEC`, and `$DESIGNOS_SELECTION_JSON`, marking the preset badge `ACTIVE PTY`.
  4. User authenticates or interacts directly in the 120fps `vt100` terminal without leaving **`design`:os studio**.

### `UC-03`: Live Hot-Apply (`<50ms`) with 1-Step Atomic Undo (`Cmd+Z`) & Diff Toast (`S01` / `S03` / `S11`)
- **Actor**: CLI Coding Agent (`claude`, `codex`, `agy`, `grok`) + Designer
- **Preconditions**: A PTY session is active and `$DESIGNOS_LIVE_SPEC` file watcher is armed.
- **Main Success Scenario**:
  1. The CLI agent writes an updated `.spec.json` (`NodeSpec`) to `$DESIGNOS_LIVE_SPEC` or emits a `DESIGNOS_SPEC_JSON:` payload block on stdout.
  2. Within `<50ms`, **`design`:os studio** validates the JSON, records a single coalesced `History` undo entry on the active `ProjectTab`, applies the new/updated frames to `Document`, and triggers a canvas repaint.
  3. System displays a diff toast (`Hot-applied from PTY (<50ms) · Press ⌘Z to undo`).
  4. Pressing `Cmd+Z` reverts the entire hot-applied batch in 1 step.

### `UC-04`: Semantic "Rename Layers" & "Rewrite / Translate / Shorten Text" (`S06` / `S11`)
- **Actor**: Product Designer
- **Preconditions**: User selects a Frame or subtree containing generic layer names (`Frame 42`, `Rectangle 19`) or draft copy.
- **Main Success Scenario**:
  1. User invokes **Rename Layers** or **Rewrite / Translate / Shorten Text** (e.g., `Translate -> Vietnamese / English`, `Shorten microcopy`) from `Actions (⌘K)` or right-click Context Menu AI.
  2. System records a single `History` snapshot, renames generic layers to semantic role-based names (`Header Bar`, `Primary CTA Button`, `KPI Card Grid`) or transforms text content in place, and logs the context payload to the active PTY session.

### `UC-05`: "Make Prototype (Auto-Flow Wiring)" & "Populate Realistic Data" (`S06` / `S11`)
- **Actor**: Product Designer / Prototyper
- **Preconditions**: User selects 2 or more screens/frames or a repeatable card/table grid on the canvas.
- **Main Success Scenario**:
  1. User triggers **Make Prototype (Auto-Flow Wiring)** or **Populate Realistic Data**.
  2. For **Make Prototype**, system wires `On Click -> Navigate To` prototype interactions (`flow_starting_point` + connector arrows) between CTA buttons and target frames.
  3. For **Populate Realistic Data**, system fills repeated text cells with realistic domain records (names, prices, timestamps, statuses) in a single undoable step.

### `UC-06`: "Design Gate Audit & Auto-Fix" (`es:designer` + WCAG AA Contrast Guard) (`S06` / `S11`)
- **Actor**: QA / Design Systems Lead
- **Preconditions**: User opens `Actions (⌘K)` -> `Design Gate Audit & Auto-Fix`.
- **Main Success Scenario**:
  1. System scans the active page/selection for `<44px` touch targets, hardcoded off-token hex colors, `<4.5:1` WCAG AA text contrast, emoji chrome violations, and nested cards-in-cards.
  2. System auto-fixes token/contrast/sizing issues in a single undoable step and pipes a structured Markdown audit report into the active PTY terminal.

### `UC-07`: Unified Phosphor Icon System & Visual Consistency Across All Studio Surfaces (`S01..S13` · `DIR-19` / `US-14`)
- **Actor**: Product Designer / Developer / QA Lead
- **Preconditions**: User navigates across `Home` (`S04`), `Multi-Tab Bar` (`S10`), `Left Rail` & `Side Panels` (`Layers`, `Agents` `S13`, `Assets`), `Bottom Floating Toolbar` & `Sub-Toolbar` (`S01`), `Right Inspector` (`Design`, `Prototype`, `Dev Mode` `S05..S08`), `Actions (⌘K)` (`S06`), and `PRODUCT:OS Spec Driven v2` Reader (`S02`/`S12`).
- **Main Success Scenario**:
  1. System loads all non-logo `crate::icons::Icon` variants (`Icon::ALL`) from official **Phosphor Icons** (`https://phosphoricons.com/` · `@phosphor-icons/core` `viewBox="0 0 256 256"`, `fill="currentColor"`) vendored in `crates/designos-app/assets/icons/phosphor/*.svg`.
  2. `resvg` rasterizes each Phosphor SVG with DPR-aware pixel snapping (`(size * dpr).round()`) and tinting, ensuring crisp 1:1 visual weight and optical balance across 1x/2x Retina displays while preserving the `Icon::Logo` brand mark.
  3. Zero legacy Lucide SVGs, zero ad-hoc procedural icon strokes in Left Rail / panels, and zero emoji chrome icons appear anywhere in **`design`:os studio**.

### `UC-08`: Pure Agentic Chat UI Wrapping Live PTY Shell (`claude` / `codex` / `agy` · `S13` · `DIR-21` / `US-15`)
- **Actor**: Product Designer / AI Engineer
- **Preconditions**: User opens the `Agents` Left Side Panel (`RailItem::Agents`, `S13`) on an open project tab.
- **Main Success Scenario**:
  1. User selects a CLI runtime in the 1-click pill bar (`Claude Code | Codex | Antigravity`), which activates or spawns the corresponding `portable-pty` + `vt100` shell (`claude`, `codex`, or `agy`) with `$DESIGNOS_*` env vars and zero external paid API keys required.
  2. User optionally clicks `+` in the `"Ask for changes"` composer to attach current Canvas Selection + Pin-Comment context, types a prompt, and submits (`Enter` or Up-Arrow button).
  3. System appends a `User` chat bubble (`AgentChatTurn` with `AgentChatRole::User`), writes the prompt line to the active PTY master, and streams ANSI-stripped stdout (`sync_chat_from_pty` + `strip_ansi_codes`) into the corresponding `Assistant` chat bubble (`AgentChatRole::Assistant`).
  4. When the CLI stdout or `$DESIGNOS_LIVE_SPEC` emits a valid `NodeSpec` JSON block, the `Assistant` chat bubble displays an inline `NodeSpec` badge (`Apply to Canvas` / `Applied`) that applies the design onto the Canvas in 1 undoable `History` step (`Cmd+Z`), and the user can toggle at any time between the clean Chat thread view and the underlying raw VT100 terminal screen (`>_ Terminal`).

---

## 2. Gherkin Acceptance Criteria (`SCN-11-01` .. `SCN-11-04`, `SCN-14-01` .. `SCN-14-03`, `SCN-15-01` .. `SCN-15-03`)

```gherkin
Feature: 1:1 Figma AI Agent Suite, Native PTY Terminal Core, Unified Phosphor Icons & Pure PTY Chat Wrapper (US-11, US-14, US-15)

  Scenario: SCN-11-01 — 5 First-Class CLI Runtime Presets & $DESIGNOS_* Environment Injection
    Given the user opens the Agents Left Rail (S11) or PTY Terminal dock (S03) in "design:os studio"
    When the user inspects the runtime launcher or spawns a preset ("claude", "codex", "agy", "grok", "$SHELL")
    Then all 5 runtime presets are listed with live INSTALLED / ACTIVE PTY badges
    And the spawned PTY environment contains $DESIGNOS_BIN, $DESIGNOS_FILE, $DESIGNOS_MCP_BIN, $DESIGNOS_LIVE_SPEC, and $DESIGNOS_SELECTION_JSON

  Scenario: SCN-11-02 — 6 Figma UI3 AI Actions Execution & Hybrid PTY Prompt Piping
    Given a Frame subtree is selected on the Canvas (S01)
    When the user triggers any of the 6 Figma AI Actions ("Make Design", "Make Prototype", "Rename Layers", "Rewrite/Translate/Shorten", "Populate Content", "Design Gate Audit & Fix") from Actions Cmd+K (S06) or Agents Rail (S11)
    Then the canvas transformation executes immediately as a single undoable History step
    And the structured prompt plus selection context is piped to the active PTY agent session

  Scenario: SCN-11-03 — Live Hot-Apply (<50ms) Watcher & 1-Step Atomic Undo (Cmd+Z)
    Given an active PTY agent session is watching $DESIGNOS_LIVE_SPEC
    When the CLI agent writes a valid NodeSpec JSON patch to $DESIGNOS_LIVE_SPEC (or emits DESIGNOS_SPEC_JSON on stdout)
    Then the active ProjectTab hot-applies the spec onto the Canvas within 50ms and shows a Diff Toast
    And pressing Cmd+Z restores the exact pre-apply Document node tree in 1 step

  Scenario: SCN-11-04 — Full Figma 126.9.13 Gap Closure (Gaps A-F) & Zero-Clone Multi-Tab Sync
    Given multiple project tabs are open on the 36px Desktop Tab Strip (S10) with Dev Mode active on one tab
    When the user inspects Type Settings (S05), Color Picker contrast bar (S08), Dev Mode 5-tool dock (S07), Variables "/" groups (S08), and Canvas Rulers (S01)
    Then each tab isolates its Dev Mode green "</>" badge and syncs without per-frame Document cloning (es:rust Law 10)
    And all Figma 126.9.13 UI3 details (WCAG AA/AAA contrast bar, Re-center Shift+2, pluginData/hyperlink/symbolDescription/proportionsConstrained) pass verification

  Scenario: SCN-14-01 — 100% Phosphor Icons Asset Integrity (viewBox="0 0 256 256") & resvg Rasterization
    Given all variants in crate::icons::Icon::ALL (except Icon::Logo)
    When the icon table is loaded from crates/designos-app/assets/icons/phosphor/*.svg and rasterized via resvg
    Then every SVG has official Phosphor viewBox="0 0 256 256" and produces non-empty alpha pixels without fallback errors

  Scenario: SCN-14-02 — Unified Phosphor Icons Across Left Rail, Side Panels, Toolbars, Inspector & Spec Reader
    Given the user inspects Left Rail, Layers, Agents (S13), Assets, Bottom Floating Toolbar, Sub-Toolbar, Top Multi-Tab Bar, Right Inspector, Cmd+K, Home, and Spec Reader (S02)
    When any icon button, badge, or layer row is rendered
    Then all icons render through the unified Phosphor Icon pipeline with DPR-snapped crispness and zero ad-hoc procedural icon drawing

  Scenario: SCN-14-03 — Brand Mark Preservation (Icon::Logo) & Zero Emoji / Mixed-Family Compliance
    Given the top-left Studio brand mark and all UI3/Operate chrome surfaces
    When the application renders Icon::Logo alongside Phosphor UI icons
    Then Icon::Logo preserves the custom design:os studio mark while 100% of other chrome icons use Phosphor Icons with zero emoji or legacy Lucide remnants

  Scenario: SCN-15-01 — 1-Click CLI Runtime Selector Pill Bar (Claude Code | Codex | Antigravity) Over Live PTY Shell
    Given the user opens the Agents Left Side Panel (RailItem::Agents, S13)
    When the user clicks "Claude Code", "Codex", or "Antigravity" in the CLI runtime selector pill bar
    Then the active PTY session switches to or spawns the selected CLI binary ("claude", "codex", or "agy") with $DESIGNOS_* environment variables and zero external API keys required

  Scenario: SCN-15-02 — Structured User/Assistant Chat Turn Bubbles Streaming ANSI-Stripped PTY Stdout
    Given an active PTY shell tab in the Agents Left Side Panel (S13)
    When the user submits a prompt (with optional "+" Selection + Pin-Comment context attachment) and the PTY shell emits ANSI-colored stdout
    Then a User turn bubble (AgentChatRole::User) and streaming Assistant turn bubble (AgentChatRole::Assistant) are recorded via sync_chat_from_pty
    And strip_ansi_codes removes raw CSI/OSC ANSI escape sequences so the chat bubble displays clean human-readable text

  Scenario: SCN-15-03 — Inline NodeSpec Detection & 1-Click/Auto Canvas Apply Badge + Instant Chat/VT100 Terminal Toggle
    Given an Assistant chat turn bubble receives PTY stdout containing a valid NodeSpec JSON block
    When the chat turn is synchronized from the PTY log
    Then the Assistant bubble surfaces an inline NodeSpec badge that applies the design onto the Canvas as a single undoable History step
    And toggling between the Chat thread view and raw VT100 terminal screen preserves full PTY session state and chat history
```

---

## 3. Edge Cases & Error States

| State | Trigger Condition | System Behavior & Recovery |
|---|---|---|
| **CLI Binary Not Installed (`empty` / fallback)** | User clicks `Antigravity (agy)` or `Grok (grok)` on a machine where the binary is not on `$PATH`. | Display `FALLBACK -> $SHELL` badge, launch an interactive `$SHELL` PTY tab with a 1-line install/login hint (`# agy not found in PATH; opened interactive shell with $DESIGNOS_* env ready`), and never crash or block UI. |
| **Malformed `$DESIGNOS_LIVE_SPEC` JSON (`error`)** | A CLI agent writes truncated or invalid JSON to `$DESIGNOS_LIVE_SPEC`. | Reject the partial write, keep the current `Document` and `History` untouched, and surface an non-blocking red `#F24822` toast (`Hot-apply skipped: invalid JSON at line:col`) while waiting for the next valid write. |
| **Empty Canvas Selection (`disabled`)** | User triggers selection-scoped AI actions (`Rename Layers`, `Rewrite Text`) with 0 selected nodes. | Automatically scope to the active Page's top-level frames or show a clear hint (`Select a frame or text layer, or run Make Design (First Draft)`). |
| **Rapid Streaming Writes (`loading` / debounce)** | Agent streams multiple writes to `$DESIGNOS_LIVE_SPEC` within `16ms`. | Coalesce writes by content hash so only distinct valid payloads are hot-applied, maintaining `120fps` (`<=8.33ms` frame budget). |

---

## 4. UX Decisions Table (Grounded in UX Laws)

| UX Law | Application in `S03`, `S06`, `S11` | Measurable Target |
|---|---|---|
| **Jakob's Law** | Mirror Figma Desktop `126.9.13` UI3 entry points 1:1: bottom-docked `Actions (⌘K)` (`480×320px`), `AI` tab, and right-click AI actions so Figma users have zero learning curve. | 99% layout & token match against `com.figma.Desktop` |
| **Hick's Law** | Group AI capabilities into 6 clear, high-leverage actions and 5 named CLI runtime pills (`Claude`, `Codex`, `Antigravity`, `Grok`, `$SHELL`) rather than a wall of 40 ungrouped settings. | `<= 2` clicks or `⌘K + Enter` to trigger any AI action |
| **Fitts's Law** | Anchor `Actions (⌘K)` directly above the `528×48px` bottom floating toolbar (`68px` from bottom edge) and keep runtime selector pills at `28px` height with clear `6px` radius hit targets. | `< 150ms` pointer acquisition from canvas center |
| **Doherty Threshold (`<400ms`)** | Execute local deterministic AI actions in `<16ms` and hot-apply `$DESIGNOS_LIVE_SPEC` changes in `<50ms` with immediate visual toast feedback. | `< 50ms` hot-apply latency; `120fps` canvas interactivity |

---

## 5. Design System & Appetite Check

- **Figma UI3 Dark Palette**: `#1E1E1E` canvas, `#2C2C2C` panel/modal surface, `#383838` input well, `#444444` border, `#0D99FF` primary accent, `#9747FF` component/AI accent, `#24CB71` Dev Mode & `INSTALLED`/`ACTIVE PTY` status green.
- **Brand Typography Rule (`BR-14`)**: Always **`design`:os studio** (all-lowercase, bold **`design`**, regular `:os studio`).
- **Rust Performance (`es:rust` Law 10)**: Active tab synchronization uses dirty-generation tracking (`sync_active_tab`) rather than cloning the entire `Document` on every 120fps GUI frame.
