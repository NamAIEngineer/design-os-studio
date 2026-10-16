---
description: "ease-design ui-diagram — Runtime-neutral diagram workflow for ease-design. Use when a request needs a structural, sequential, or hierarchical picture and prose/tables would lose structure a reader needs to trace. Charts of quantities belong to the chart workflow."
---

# ui-diagram

Follow the runtime-neutral workflow at:
`/opt/homebrew/lib/node_modules/ease-design/templates/workflows/diagram.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

When the workflow calls for a `ui` command, run it via the shell:

// turbo
```bash
ui diagram "$ARGS"
```
