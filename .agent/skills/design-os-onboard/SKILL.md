---
name: design-os-onboard
description: "Onboard a project onto design:os end to end: pick the right entry point for what already exists (nothing yet, a codebase, a URL, a Figma file, a token set, or reference shots), verify the toolchain with ui doctor / design-os doctor, compile or import the design system, declare the soul, wire agents + heartbeat, and open the Figma plugin. Use when the user says 'onboard', 'set up design:os', 'install this', 'wire up my project', 'get started with design:os', or right after ui init."
---

Follow the runtime-neutral skill at:
`/opt/homebrew/lib/node_modules/ease-design/templates/journeys/onboard.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

