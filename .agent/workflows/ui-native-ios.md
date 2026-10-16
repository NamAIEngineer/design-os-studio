---
description: "ease-design ui-native-ios — Build a first-class native iOS app through compact-first SwiftUI composition and provisional, evidence-bounded delivery. Use when the requested artifact is an iPhone or iOS application."
---

# ui-native-ios

Follow the runtime-neutral workflow at:
`/opt/homebrew/lib/node_modules/ease-design/templates/workflows/native-ios.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

Create the typed activation request required by that workflow, then run its real entry check:

// turbo
```bash
ui knowledge activate capability-activation-request.json --json > capability-activation.json
```
