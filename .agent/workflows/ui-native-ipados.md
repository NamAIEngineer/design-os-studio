---
description: "ease-design ui-native-ipados — Build a first-class native iPadOS workspace through resizable SwiftUI composition, split navigation, mixed input, and provisional evidence. Use when the requested artifact is an iPad or iPadOS application."
---

# ui-native-ipados

Follow the runtime-neutral workflow at:
`/opt/homebrew/lib/node_modules/ease-design/templates/workflows/native-ipados.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

Create the typed activation request required by that workflow, then run its real entry check:

// turbo
```bash
ui knowledge activate capability-activation-request.json --json > capability-activation.json
```
