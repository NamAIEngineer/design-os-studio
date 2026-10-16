---
name: design-os-check-consistency
description: "Score the Consistency axis — design-system token and component reuse. Use when auditing whether generated HTML uses the project's DS tokens and registered components instead of ad-hoc values."
---

Follow the runtime-neutral skill at:
`/opt/homebrew/lib/node_modules/ease-design/templates/skills/check-consistency.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

