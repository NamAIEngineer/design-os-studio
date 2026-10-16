---
name: design-os-figma-craft
description: "Pick and verify idiomatic Figma constructions when building or editing a canvas via the external figma-agent CLI. Use before building (choose auto-layout, component, and variable structure) and after building (run the construction lints)."
---

Follow the runtime-neutral skill at:
`/opt/homebrew/lib/node_modules/ease-design/templates/skills/figma-craft.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

