---
description: "ease-design ui-refine — Self-correction pass that fixes execution quality without redesigning. Use when a variant scores low on craft axes or the user asks to polish or clean up a design."
---

# ui-refine

Follow the runtime-neutral workflow at:
`/opt/homebrew/lib/node_modules/ease-design/templates/workflows/refine.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

When the workflow calls for a `ui` command, run it via the shell:

// turbo
```bash
ui refine "$ARGS"
```
