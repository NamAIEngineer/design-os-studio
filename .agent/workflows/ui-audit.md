---
description: "ease-design ui-audit — Audit a Figma file against its design system and normalize the violations. Use when the user says 'audit', 'fix violations', 'normalize to the DS', or wants raw hex / detached instances / raw icons / off-grid spacing / deprecated components cleaned up."
---

# ui-audit

Follow the runtime-neutral workflow at:
`/opt/homebrew/lib/node_modules/ease-design/templates/workflows/audit.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

When the workflow calls for a `ui` command, run it via the shell:

// turbo
```bash
ui audit "$ARGS"
```
