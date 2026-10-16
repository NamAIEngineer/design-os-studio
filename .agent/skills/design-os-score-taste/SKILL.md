---
name: design-os-score-taste
description: "Evaluate a generated design against the 6+1-axis taste rubric. Use when a workflow gate asks to score a variant, run the taste gate, or judge whether a design clears the ship bar."
---

Follow the runtime-neutral skill at:
`/opt/homebrew/lib/node_modules/ease-design/templates/skills/score-taste.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

