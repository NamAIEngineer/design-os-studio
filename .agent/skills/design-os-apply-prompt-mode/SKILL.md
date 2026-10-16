---
name: design-os-apply-prompt-mode
description: "Decide how faithfully to track a reference input. Use when a workflow consumes a screenshot, URL, or Figma frame and must choose between faithful reproduction and inspired reinterpretation."
---

Follow the runtime-neutral skill at:
`/opt/homebrew/lib/node_modules/ease-design/templates/skills/apply-prompt-mode.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

