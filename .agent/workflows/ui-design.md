---
description: "ease-design ui-design — Design something NEW from a requirement, on the Figma canvas — a whole screen or a single component. Detects the scope and runs the matching discipline, driven by an understand-until-decision-ready loop. Use when the Figma canvas is named as the target AND the thing is still to be decided — only then: 'design a screen/page in Figma', 'I want a new <X> on the canvas', 'add a variant to <component> on the canvas'. Otherwise route per G2 in knowledge/need-routing.md (no canvas named → generate; formed intent to construct → to-figma)."
---

# ui-design

Follow the runtime-neutral workflow at:
`/opt/homebrew/lib/node_modules/ease-design/templates/workflows/design.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

When the workflow calls for a `ui` command, run it via the shell:

// turbo
```bash
ui design "$ARGS"
```
