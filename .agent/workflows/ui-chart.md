---
description: "ease-design ui-chart — Runtime-neutral chart workflow for ease-design. Use when a standalone chart of quantities is the deliverable and prose or a table would bury the comparison. Charts inside a page or deck belong to generate or slides."
---

# ui-chart

Follow the runtime-neutral workflow at:
`/opt/homebrew/lib/node_modules/ease-design/templates/workflows/chart.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

When the workflow calls for a `ui` command, run it via the shell:

// turbo
```bash
ui chart "$ARGS"
```
