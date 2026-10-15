# Phase 5 Use Cases & Gherkin Acceptance Criteria

## US-1: Embedded PTY Agent Terminal (`claude`, `codex`, `grok`, `$SHELL`)
- **Priority**: MUST
- **As a** Product Engineer / Designer working in DESIGN:OS Studio,
- **I want** a native PTY Terminal docked inside the studio where I can log into `claude` CLI, `codex` CLI, or `grok` CLI and pipe Canvas selections, Pin-Comments, and `.spec.json` results back and forth,
- **So that** I never have to context-switch out of DESIGN:OS Studio to collaborate with AI coding/design agents.

```gherkin
Feature: Embedded PTY Agent Terminal & Bidirectional Canvas Bridge
  Scenario: Spawn Claude CLI or Shell with auto-injected DESIGNOS environment
    Given DESIGN:OS Studio is open with a document on Canvas
    When the user opens the Terminal workspace (`--panel terminal` or `Ctrl+\``) and clicks "+ Claude CLI" or "+ Shell"
    Then a real PTY session is spawned with `DESIGNOS_BIN`, `DESIGNOS_FILE`, and `TERM=xterm-256color` exported
    And VT100/ANSI TrueColor output and keystrokes are rendered interactively at 60fps

  Scenario: Send Canvas selection and unresolved Pin-Comments to Agent CLI
    Given a Frame with unresolved Pin-Comments is selected on the Canvas
    When the user clicks "Send Selection & Comments" in the Terminal header
    Then the selected node spec and comment threads are written to `/tmp/designos-live-selection.spec.json` and piped into the active PTY stdin

  Scenario: Apply Agent JSON NodeSpec output directly to Canvas
    Given an Agent CLI in the PTY outputs a fenced JSON `NodeSpec` or mutates `$DESIGNOS_FILE`
    When the user clicks "Apply Spec from Output" or the file watcher fires
    Then the new/updated screen frame is laid out via Auto-Layout 5.0 and rendered on the GPU Canvas
```

## US-2: Auto-Layout 2D CSS Grid (`LayoutDirection::Grid`)
- **Priority**: MUST
- **As a** UI Architect importing or building bento/dashboard grids,
- **I want** 2D Auto-Layout `Grid` with configurable columns, rows, gap, and child `col_span` / `row_span`,
- **So that** 2D grid layouts work 1:1 with Figma 2025 Grid and export cleanly to CSS Grid & Tailwind.

```gherkin
Feature: Auto-Layout 2D CSS Grid
  Scenario: Solve 2D Grid layout with Flex, Fixed, and Hug tracks
    Given a Frame has `AutoLayout` with `direction = Grid` and `grid_columns = 3`
    When child nodes with `col_span = 2` and `col_span = 1` are laid out
    Then `layout::compute` places children into `(row, col)` cells and exports `display: grid` in CSS and `grid grid-cols-3` in Tailwind
```

## US-3: Component Slots, Scoped Variables & Variable Font Axes
- **Priority**: SHOULD
- **As a** Design Systems Engineer,
- **I want** `PropKind::Slot` preservation during instance sync, `Variable::scopes` filtering, and Variable Font axis (`wght`, `wdth`, `opsz`, `slnt`) + OpenType (`tnum`, `liga`, `ss01`) controls,
- **So that** complex design systems round-trip 100% losslessly through `.fig` v106 files.

```gherkin
Feature: Lossless .fig v106 Export & Design System Parity
  Scenario: Round-trip Component Props, Instance Overrides, and OpenType features through .fig
    Given a Document contains Components with props, Instances with overrides, and Text with OpenType features and variation axes
    When `export_fig` encodes the Document and `import_fig` reloads the `.fig` archive
    Then all component props, instance overrides, and typography axes/features are preserved 1:1
```
