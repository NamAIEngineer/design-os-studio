---
description: "ease-design ui-evidence — Turn a transcript, interview, survey, or notes file into verified user-evidence findings and record them in the ledger the coverage gate cites. Use when the user says 'log evidence', 'add a finding', 'ingest this transcript/interview/survey', or wants an acceptance criterion backed by a real user quote or metric."
---

# ui-evidence

Follow the runtime-neutral workflow at:
`/opt/homebrew/lib/node_modules/ease-design/templates/workflows/evidence.md`

The workflow reads files under `knowledge/`. Resolve every such path against this absolute base: `/opt/homebrew/lib/node_modules/ease-design/knowledge` (e.g. `knowledge/persona-index.md` → `/opt/homebrew/lib/node_modules/ease-design/knowledge/persona-index.md`).

When the workflow calls for a `ui` command, run it via the shell:

// turbo
```bash
ui evidence "$ARGS"
```
