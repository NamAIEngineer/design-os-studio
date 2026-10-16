---
description: "ease-design ui-figma-comments — Pull Figma comments scoped to what the owner points at, resolve which screen and element each pin means, decide them one at a time, and read the owner's verdict back off the threads you post. Use when reviewing, triaging, or closing the loop on Figma design feedback."
---

# ui-figma-comments

Follow the runtime-neutral workflow at:
`/opt/homebrew/lib/node_modules/ease-design/templates/workflows/figma-comments.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

When the workflow calls for a `ui` command, run it via the shell:

// turbo
```bash
ui figma-comments "$ARGS"
```
