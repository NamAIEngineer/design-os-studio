# Use Cases & Acceptance Criteria: 1:1 Figma Binary Parity & Agentic Spec Reader

## MoSCoW Prioritization

### Must Have
- **UC-1: 1:1 `.fig` Import Fidelity (`designos-fig`)** — Asymmetric Auto-Layout padding, `parameterConsumptionMap` variable bindings, `FONT_STYLE` variable aliases, `symbolOverrides` variable/fill/stroke/layout/swap overrides, and `CANVAS.handoffStatusMap` Dev Mode statuses.
- **UC-2: Native `.fig` Binary Writer (`export_fig` / `save_path` / `--export-fig` / MCP `export_fig_file`)** — Reverse-engineered `fig-kiwi` v106 binary schema + `Message` + `NodeChange` + `commandsBlob` + `meta.json` + `images/*` ZIP archive.
- **UC-3: 1:1 Figma UI3 Icon Set (`designos-app`)** — Authentic 16x16 Figma geometry for `Frame`, `AutoLayoutH`, `AutoLayoutV`, `Component`, `ComponentSet`, `Instance`, `Section`, `Group`, `Mask`, `Radius`, `PaddingH`, `PaddingV`, `Spacing`, `StrokeSides`, and `Specs`.
- **UC-4: Native EaseUI-Styled Spec Reader (`RailItem::Specs`)** — In-app Spec Explorer & Split Reader Drawer rendering `brainstorm.md`, `usecases.md`, `plan.md`, and `.spec.json` with 1-click Canvas Frame focus, Spec Apply/Export, and `.fig` Export.

### Should Have
- **UC-5: Live Screen Frame Linking & Pin-Comment Badges** — Automatic matching between spec sections/frames and Canvas frames, displaying live open comment counts per screen.

---

## User Stories & Gherkin Acceptance Criteria

### US-1: Lossless `.fig` Import & Instance Override Resolution
**As a** product designer importing a complex Figma `.fig` file (`VSF - PCP.fig`),
**I want** every Auto-Layout frame, variable-bound text/padding token, swapped nested icon instance, and overridden badge color to match Figma 100%,
**So that** I can review and iterate on production design systems without visual regressions.

```gherkin
Scenario: Instance child overrides a master fill/stroke variable binding
  Given a master Component child with a bound variable on "fills/0" or "strokes/0"
  When an Instance overrides "fillPaints" or "strokePaints" in symbolOverrides
  Then the stale master "fills/*" or "strokes/*" binding is cleared before applying the override's paint and colorVar
  And the rendered instance child displays the overridden color or variable token

Scenario: Auto-Layout frame uses parameterConsumptionMap and asymmetric padding
  Given a NodeChange or symbolOverride with stackHorizontalPadding, stackVerticalPadding, stackPaddingRight, stackPaddingBottom, and parameterConsumptionMap
  When import_fig decodes the document
  Then all 4 padding sides and parameterConsumptionMap bindings (STACK_SPACING, STACK_PADDING_*, FONT_*) are preserved
```

### US-2: Save & Export Directly to `.fig` Binary
**As a** designer or AI agent working in DESIGN:OS Studio,
**I want** to save any document directly as a `.fig` file via File -> Save As, CLI `--export-fig`, or MCP `export_fig_file`,
**So that** `.fig` is a first-class bidirectional format alongside `.designos`.

```gherkin
Scenario: Roundtrip Document -> .fig binary -> Document
  Given a Document containing pages, auto-layout frames, vectors, text runs, components, instances, variables, and images
  When export_fig serializes the Document into a .fig ZIP archive
  And import_fig reads the generated .fig bytes back
  Then all pages, node hierarchies, geometry blobs, text runs, components, instances, and variables roundtrip losslessly
```

### US-3: Native EaseUI Spec Reader & Agentic Hub
**As a** human reviewer collaborating with an AI coding agent,
**I want** an EaseUI-styled Spec Reader inside the editor (`Specs` rail tab),
**So that** I can read User Stories and Gherkin criteria side-by-side with the Canvas, jump to linked screens, and apply or export `.spec.json` blueprints in one click.

```gherkin
Scenario: Opening the Spec Reader from the Left Rail or CLI
  Given workspace specs under plans/specs/ and examples/specs/
  When the user clicks the "Specs" rail item or launches with "--panel specs"
  Then the left sidebar lists discovered Spec Packages and Declarative .spec.json files
  And the EaseUI Spec Reader card view renders Overview, Use Cases (Gherkin), Plan, and NodeSpec tabs with 1-click Canvas actions
```
