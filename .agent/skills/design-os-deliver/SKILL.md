---
name: design-os-deliver
description: "Ship a design:os change: run the full-stack audit in the order that actually catches problems (flow, evidence, VR baseline, paired-tokens, rendered a11y, full audit), work the delivery checklist (specimen, docs, preview, vr-matrix, changelog) including the vr-matrix step most agents never discover, know exactly what a static a11y pass does not prove, and hand off the diff/changelog/PR. Use when preparing a release, a PR, a specimen page, or any 'is this ready to ship' question."
---

Follow the runtime-neutral skill at:
`/opt/homebrew/lib/node_modules/ease-design/templates/journeys/deliver.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

