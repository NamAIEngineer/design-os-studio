---
description: "ease-design ui-learn — Teach ease-design an existing project's design system — scan the repo, harvest tokens/components/interaction states from real evidence, and compile the DS so generation matches the product. Use when initializing ease-design on a project that already has UI, or to re-learn after a major redesign."
---

# ui-learn

Follow the runtime-neutral workflow at:
`/opt/homebrew/lib/node_modules/ease-design/templates/workflows/learn.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

When the workflow calls for a `ui` command, run it via the shell:

// turbo
```bash
ui learn "$ARGS"
```
