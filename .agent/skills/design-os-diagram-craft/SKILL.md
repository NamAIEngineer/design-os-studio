---
name: design-os-diagram-craft
description: "Use when the user asks for a structural, sequential, or hierarchical diagram (or asks to generate or update one) and needs it delivered as a rendered design artifact rather than prose or a text description. Charts of quantities route to chart-craft instead."
---

Follow the runtime-neutral skill at:
`/opt/homebrew/lib/node_modules/ease-design/templates/skills/diagram-craft.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

