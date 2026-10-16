---
description: "ease-design ui-iterate — Apply a focused vibe-word edit to an existing variant. Use when the user asks to tweak, adjust, or nudge a generated design without redesigning it."
---

# ui-iterate

Follow the runtime-neutral workflow at:
`/opt/homebrew/lib/node_modules/ease-design/templates/workflows/iterate.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

When the workflow calls for a `ui` command, run it via the shell:

// turbo
```bash
ui iterate "$ARGS"
```
