---
name: design-os-gflow-flight
description: "Generate a multi-leg camera flight or a conditioned still through the gflow hand (Google Flow / Veo / Imagen on a subscription), using the verify-then-spend adapter so a stranded render never costs a second credit. Use when a brief needs authored video legs whose seams must be frame-identical; do not use for one-off text-to-video where the official API path is simpler, and never inside CI or an unattended build."
---

Follow the runtime-neutral skill at:
`/opt/homebrew/lib/node_modules/ease-design/templates/skills/gflow-flight.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

