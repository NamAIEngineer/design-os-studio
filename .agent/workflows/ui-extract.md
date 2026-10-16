---
description: "ease-design ui-extract — Extract a design system from existing HTML. Use when the user provides HTML and wants its tokens and components harvested into a project design system."
---

# ui-extract

Follow the runtime-neutral workflow at:
`/opt/homebrew/lib/node_modules/ease-design/templates/workflows/extract.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

When the workflow calls for a `ui` command, run it via the shell:

// turbo
```bash
ui extract "$ARGS"
```
