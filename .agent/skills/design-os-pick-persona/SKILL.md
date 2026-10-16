---
name: design-os-pick-persona
description: "Choose one or more design personas for a generation, redesign, or extraction task. Use when a workflow step says to select N personas from intent, pick a contrasting persona, or resolve a persona the user named to a concrete slug."
---

Follow the runtime-neutral skill at:
`/opt/homebrew/lib/node_modules/ease-design/templates/skills/pick-persona.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

