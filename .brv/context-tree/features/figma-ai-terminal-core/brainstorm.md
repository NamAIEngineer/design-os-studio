# Feature Context: 1:1 Figma AI Agent Surface + Native PTY Terminal Core (`US-11` / `EP-11`)

- **Product**: **`design`:os studio** (always all-lowercase, bold **`design`**, regular `:os studio` — `BR-14`)
- **Spec Bundle**: `plans/specs/designos-studio-v2/` (`brainstorm.md`, `usecases.md`, `us/US-11.md`)
- **Approved Architecture (`A1`..`A3` / `D12`..`D13`)**:
  1. **A1 — Hybrid Dual-Mode Architecture (`D12`, `BR-16`)**:
     - Preserves 1:1 Figma UI3 AI surfaces (`Actions ⌘K` `S06`, `Agents` Left Rail `S11`, and Canvas Context Menu AI) that execute immediate undoable canvas transformations AND pipe structured prompts + live selection context into the active PTY Terminal session (`S03`).
     - Supports 6 canonical Figma AI Actions:
       1. `Make Design (First Draft)`
       2. `Make Prototype (Auto-Flow Wiring)`
       3. `Rename Layers`
       4. `Rewrite / Translate / Shorten Text`
       5. `Populate Realistic Data`
       6. `Design Gate Audit & Auto-Fix`
  2. **A2 — Live Hot-Apply (`<50ms`) + 1-Step Undo (`Cmd+Z`) + Diff Toast (`D13`, `BR-17`)**:
     - Auto-injects `$DESIGNOS_BIN`, `$DESIGNOS_FILE`, `$DESIGNOS_MCP_BIN`, `$DESIGNOS_LIVE_SPEC`, and `$DESIGNOS_SELECTION_JSON` into every spawned PTY session.
     - Watches `$DESIGNOS_LIVE_SPEC` (and stdout `DESIGNOS_SPEC_JSON:` frames) and hot-applies changes onto the active `ProjectTab` in `<50ms` as a single coalesced `History` undo step (`Cmd+Z`) with a diff toast notification.
  3. **A3 — 5 First-Class CLI Runtime Presets (`D12`, `BR-16`)**:
     - `Claude Code` (`claude`), `Codex` (`codex`), `Antigravity` (`agy`), `Grok` (`grok`), and `$SHELL` (`zsh`/`bash`) with `$PATH` binary detection and `INSTALLED` / `ACTIVE PTY` badges.
  4. **Figma `126.9.13` 1:1 Gap Closure (`DIR-14`, `BR-18`)**:
     - `Type settings` per-tab content filtering (`Basic | Details | Variable`), Color Picker WCAG AA/AAA contrast bar + dotted curve, Dev Mode 5-tool bottom dock (`Select | Annotate | Measure | Comment | Actions`) + `Re-center ⇧2`, Variables `/` group headers, Canvas Rulers default + blue `#1E3A5F`/`#0D99FF` selection span, Top Multi-Tab Dev Mode green `</>` badge + `es:rust` Law 10 zero-clone per-frame tab sync, and `.fig` v106 `pluginData` / `hyperlink` / `symbolDescription` / `proportionsConstrained`.
