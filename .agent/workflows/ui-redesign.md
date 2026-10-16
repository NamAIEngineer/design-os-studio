---
description: "ease-design ui-redesign — Radical contra-persona redesign of an existing variant. Use when the user rejects the current direction and asks for something completely different."
---

# ui-redesign

Follow the runtime-neutral workflow at:
`/opt/homebrew/lib/node_modules/ease-design/templates/workflows/redesign.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

When the workflow calls for a `ui` command, run it via the shell:

// turbo
```bash
ui redesign "$ARGS"
```
