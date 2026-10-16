# design:os studio — Self-Evolving Design Learning Loop

`design:os studio` runs a closed-loop, evidence-backed design evolution cycle powered by the `ui` (`design-os`) CLI kernel. Every visual audit, reverse-engineering study, token refinement, and production gate feeds back into project memory and soul ratification.

```
[Audit / RE / Gate Failure]
        │
        ▼
1. Record Events (`ui memory record` → `design/memory.events.jsonl`)
   - taste_verdict, token_change, user_pick, gap, harvested, insight
        │
        ▼
2. Compile Memory Graph (`ui memory compile` → `design/memory.graph.json`)
   - Links raw events into causal insights (`--refs`)
        │
        ▼
3. Harvest Reports (`design-os harvest --emit-packet` → `design/harvest-inbox/`)
   - Extracts durable lessons from `plans/reports/*.md`
        │
        ▼
4. Scheduled Heartbeat (`design-os heartbeat --force` → `design/heartbeat-state.json`)
   - Runs `ds-a11y`, `specimen`, `harvest`, and `reflect` tasks
        │
        ▼
5. Ratify into Soul & Gates (`design/soul.md`, `scripts/zero_fake_ui_gate.py`)
   - Converts recurring gaps into permanent `Never`/`Always` clauses and `Z01..Z12` checks
        │
        ▼
6. Verify Living-Agent Proof (`design-os evolution` → `ALIVE` / `IMPROVING`)
   - Audits `design/evolution-proof.json` for causal applications, gated defects, and blind A/B gains
```

## Operational Commands

| Stage | Command | Artifact Updated |
|---|---|---|
| Token & Role Sync | `ui ds import design/tokens.json --force` / `ui ds change-token` | `design/design.tokens.json`, `design/ds.manifest.json` |
| Contrast Gate | `ui ds a11y --dir .` | Verifies 0 pairs below WCAG AA (4.5:1) |
| Component Specimen | `ui ds specimen --dir .` & `ui ds preview --dir .` | `design/component-registry.json`, `design/specimen.html` |
| Soul Lint | `ui ds soul check --dir .` | Verifies `status: ratified` & 0 structural findings in `design/soul.md` |
| Agent Roster Check | `ui agents check --dir .` | Verifies soul-bound designer, curator, figma agents + studio librarian |
| Memory Compilation | `ui memory compile --dir .` | Compiles `design/memory.events.jsonl` → `design/memory.graph.json` |
| Report Harvest | `design-os harvest --dir . --emit-packet` | Populates `design/harvest-inbox/` from `plans/reports/` |
| Heartbeat Execution | `design-os heartbeat --dir . --force` | Executes `design/heartbeat.json` tasks → `design/heartbeat-state.json` |
| Evolution Verdict | `design-os evolution --dir .` | Validates `design/evolution-proof.json` → `ALIVE` (`IMPROVING`) |
