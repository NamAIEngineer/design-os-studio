---
name: design-os-verify-canvas
description: "Prove a canvas change actually landed, with checks that can refuse rather than pass. Use when verifying any live Figma mutation, before reporting a task done or posting a handoff."
---

Follow the runtime-neutral skill at:
`/opt/homebrew/lib/node_modules/ease-design/templates/skills/verify-canvas.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

