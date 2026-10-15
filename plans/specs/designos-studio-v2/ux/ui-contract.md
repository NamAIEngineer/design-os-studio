---
id: ui-contract
type: ui-contract
title: "Behavioral Selectors & UI Contract (12-Column Matrix)"
updated: 2026-10-15
---

| Screen | State | Route | Roles | Scope | US | Element | Kind | Components | Patterns | Target | SCN |
|---|---|---|---|---|---|---|---|---|---|---|---|
| S01 | normal | /studio/canvas | Designer, Dev | workspace | US-01 | id=hero-apply-spec; label=Apply Spec to Canvas | button | designos/ActionButton | primary-action | Apply active NodeSpec (.spec.json) to current page | SCN-01-03 |
| S01 | empty | /studio/canvas | Designer, Dev | workspace | US-01 | id=canvas-empty-cta; label=Apply Spec to Canvas | button | designos/EmptyStateCard | empty-single-cta | Populate empty canvas with starter NodeSpec frame | SCN-01-03 |
| S01 | loading | /studio/canvas | Designer, Dev | workspace | US-01 | id=fig-import-status; label=Loading .fig v106 chunks | status | designos/StatusPill | skeleton-progress | Decode Kiwi binary container and preserve raw_fields | SCN-01-01 |
| S01 | error | /studio/canvas | Designer, Dev | workspace | US-01 | id=fig-error-toast; label=Spec error banner | alert | designos/ToastBanner | inline-recovery | Report malformed .fig or .spec.json without data loss | SCN-01-01 |
| S01 | disabled | /studio/canvas | Designer, Dev | workspace | US-01 | id=specs-export-json-btn; label=Export .spec.json | button | designos/ActionButton | disabled-reason | Disable frame export when no valid frame exists | SCN-01-02 |
| S01 | focus | /studio/canvas | Designer, Dev | workspace | US-01 | id=grid-inspector-tracks; label=2D CSS Grid Tracks | input | designos/GridTrackEditor | focus-ring-visible | Edit 1fr/2fr/px/auto grid tracks and col_span/row_span | SCN-01-02 |
| S02 | normal | /studio/specs | PO, Dev, QA | spec-bundle | US-02 | id=spec-tab- switcher; label=Role View Switcher | tablist | pcp/RoleTabStrip | operate-overview | Switch between PO View, Dev View, QA, Wireframes, Gates | SCN-02-01 |
| S02 | empty | /studio/specs | PO, Dev, QA | spec-bundle | US-02 | id=spec-legacy-fallback; label=Synthesized v2 Model | region | pcp/EmptyFallback | graceful-degradation | Synthesize PO/Dev/QA views from legacy brainstorm/usecases | SCN-02-01 |
| S02 | loading | /studio/specs | PO, Dev, QA | spec-bundle | US-02 | id=spec-bundle-loader; label=Scanning spec.manifest.json | status | pcp/SkeletonRows | doherty-400ms | Load local and external Spec Format v2 bundles | SCN-02-01 |
| S02 | error | /studio/specs | PO, Dev, QA | spec-bundle | US-02 | id=spec-missing-badge; label=MISSING artifact pill | badge | pcp/StatusPill | explicit-missing | Highlight missing or stale spec evidence inline | SCN-02-03 |
| S02 | disabled | /studio/specs | PO, Dev, QA | spec-bundle | US-02 | id=gate-export-disabled; label=Approve Gate (Pending) | button | pcp/GateActionBtn | fail-closed-gate | Keep gate approval locked while mandatory criteria < 2 | SCN-02-03 |
| S02 | focus | /studio/specs | PO, Dev, QA | spec-bundle | US-02 | id=copy-for-ai-btn; label=Copy cho AI | button | pcp/CopyCitationBtn | citation-clipboard | Copy File:path:start-end + US/AC context to clipboard | SCN-02-02 |
| S03 | normal | /studio/terminal | Dev, QA | pty-session | US-03 | id=pty-agent-bar; label=Agent Preset Launcher | toolbar | designos/TerminalHeader | pty-session-mux | Spawn or switch claude, codex, gemini, or shell PTY | SCN-03-01 |
| S03 | empty | /studio/terminal | Dev, QA | pty-session | US-03 | id=pty-empty-start; label=Start Claude / Codex | button | designos/EmptyTerminal | onboarding-cta | Launch first interactive agent session in workspace | SCN-03-01 |
| S03 | loading | /studio/terminal | Dev, QA | pty-session | US-03 | id=pty-spawn-indicator; label=Connecting PTY pipe | status | designos/StatusPill | non-blocking-io | Stream ANSI output from background thread at 120fps | SCN-03-01 |
| S03 | error | /studio/terminal | Dev, QA | pty-session | US-03 | id=pty-binary-error; label=Agent CLI not found | alert | designos/TerminalAlert | fallback-to-shell | Offer $SHELL fallback when agent binary is absent | SCN-03-02 |
| S03 | disabled | /studio/terminal | Dev, QA | pty-session | US-03 | id=pty-restart-btn; label=Restart Session | button | designos/ActionButton | process-exit-guard | Disable stdin send when child PTY exited; offer restart | SCN-03-03 |
| S03 | focus | /studio/terminal | Dev, QA | pty-session | US-03 | id=pty-paste-citation; label=Paste Spec Citation | input | designos/TerminalInput | live-spec-bridge | Inject last_ai_copy citation payload into active PTY | SCN-03-02, SCN-03-03 |
