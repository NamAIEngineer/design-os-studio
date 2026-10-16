---
description: "ease-design ui-generate — Generate evidence-qualified marketing or landing-page HTML. Use when web-marketing capability activation succeeds for a plain-language request or visual reference."
---

# ui-generate

Follow the runtime-neutral workflow at:
`/opt/homebrew/lib/node_modules/ease-design/templates/workflows/generate.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

When the workflow calls for a `ui` command, run it via the shell:

// turbo
```bash
ui generate "$ARGS"
```
