---
description: "ease-design ui-from-ref — Generate high-fidelity HTML from a reference screenshot or image. Use when the user provides an image to reproduce or draw inspiration from."
---

# ui-from-ref

Follow the runtime-neutral workflow at:
`/opt/homebrew/lib/node_modules/ease-design/templates/workflows/from-ref.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

When the workflow calls for a `ui` command, run it via the shell:

// turbo
```bash
ui from-ref "$ARGS"
```
