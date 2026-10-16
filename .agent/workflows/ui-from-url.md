---
description: "ease-design ui-from-url — Extract a portable DESIGN.md spec folder from a live URL. Use when the user points at a website and wants its design system captured for reuse."
---

# ui-from-url

Follow the runtime-neutral workflow at:
`/opt/homebrew/lib/node_modules/ease-design/templates/workflows/from-url.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

When the workflow calls for a `ui` command, run it via the shell:

// turbo
```bash
ui from-url "$ARGS"
```
