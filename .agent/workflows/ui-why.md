---
description: "ease-design ui-why — Answer why a design decision is the way it is — trace picks, edits, verdicts, and token changes with provenance from the project's design memory. Use when the user asks why a color/spacing/persona/component is the way it is, what was decided, or when something changed."
---

# ui-why

Follow the runtime-neutral workflow at:
`/opt/homebrew/lib/node_modules/ease-design/templates/workflows/why.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

When the workflow calls for a `ui` command, run it via the shell:

// turbo
```bash
ui why "$ARGS"
```
