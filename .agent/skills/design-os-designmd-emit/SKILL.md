---
name: design-os-designmd-emit
description: "Compose a DESIGN.md file from harvested tokens, observed components, and a chosen persona family. Use inside the from-url workflow when writing the 8-section Google-Labs-spec DESIGN.md."
---

Follow the runtime-neutral skill at:
`/opt/homebrew/lib/node_modules/ease-design/templates/skills/designmd-emit.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

