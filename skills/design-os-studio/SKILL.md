---
name: design-os-studio
description: "Use when working with DESIGN:OS Studio (designos-studio, designos, designos-mcp, designos-diff) — native Rust/wgpu Figma .fig v106 read/write engine, Agentic PTY Chat UI, 11-stage Spec Reader, Dev Mode codegen, and headless GPU screenshot QA."
---

# DESIGN:OS Studio (`design-os-studio`)

Native GPU-accelerated interface design studio (`wgpu` Metal / Vulkan / DX12 + `tiny-skia`), enterprise Figma `.fig` v106 binary read/write engine, native Agentic PTY Chat UI (`Claude Code` · `Codex` · `Antigravity`), 11-stage `PRODUCT:OS Spec Driven v2` Reader, multi-target Dev Mode code generator, and 22-tool Model Context Protocol server (`designos-mcp`) built in 100% Rust.

## When to Use

Activate this skill when the user asks to:
- `"Apply a declarative NodeSpec JSON onto the DESIGN:OS Studio canvas, export a verified Figma .fig v106 binary archive, and capture a headless GPU screenshot."`
- `"Open the 11-stage PRODUCT:OS Spec Driven v2 Reader (--panel specs) or Agentic PTY Chat UI (--rail agents) and verify User Stories, Gherkin scenarios, and g1..g5 gates."`
- `"Inspect a .fig or .designos document in Dev Mode, generate React/Tailwind/CSS/SwiftUI/Compose code, or run perceptual visual diffing with designos-diff."`
- `"Package native cross-platform installers (.dmg, -setup.exe, .deb) for designos-studio, designos, designos-mcp, and designos-diff."`

## Progressive Disclosure Routing (`What loads when`)

1. **Always Loaded**: `skills/design-os-studio/SKILL.md` (this router — keeps context lean at ~120 lines).
2. **On Demand — Prompt & CLI Recipes**: Load `docs/cookbook.md` when selecting multi-step workflows, headless screenshot flags, or MCP server patterns.
3. **On Demand — Installation & Release Packaging**: Load `docs/install.md` and `docs/releasing.md` when building or installing `.dmg`, `-setup.exe`, or `.deb` packages.
4. **On Demand — Diagnostics**: Load `commands/doctor.md` or run `python3 scripts/self_check.py` when verifying repository or skill health.
5. **On Demand — Ecosystem Discovery**: Load `llms.txt` when cross-referencing companion `DESIGN:OS` engines.

## Core Binaries & CLI Surface

| Binary | Role | Key Flags / Subcommands |
| :--- | :--- | :--- |
| `designos-studio` | Native GPU/CPU Desktop Studio & Headless Visual QA Runner | `--open-id <id>`, `--select <name>`, `--zoom <pct>`, `--gpu`, `--size <WxH>`, `--screenshot <out.png>`, `--rail <files\|assets\|agents\|specs\|variables\|components>`, `--tab <design\|prototype\|dev>`, `--dev-lang <react\|tailwind\|css\|swiftui\|compose>`, `--panel <specs\|variables\|cmdk\|main-menu\|color-picker>`, `--tool-mode <comment>` |
| `designos` | Agent-First CLI for Spec Apply, `.fig` Binary Export & Comment Loops | `--apply-spec <spec.json>`, `--export-fig <out.fig>`, `--export-spec <out.spec.json>`, `--inspect-json`, `--comments-json`, `--add-comment <text>`, `--reply-comment <id> --message <msg> --resolve` |
| `designos-mcp` | Local stdio Model Context Protocol Server (22 tools) | `--workspace <dir>` (`designos_open_file`, `designos_apply_spec`, `designos_export_fig_file`, `designos_generate_code`, `designos_list_comments`, etc.) |
| `designos-diff` | Perceptual SSIM & Structural Diff CLI | `compare <ref.png> <actual.png> --out <diff.png>` |

## Core Execution Workflow

1. **Inspect Context**: Use `designos <file> --inspect-json` or `designos_get_document` via MCP to inspect pages, frames, tokens, and handoff status before mutating.
2. **Execute Deterministically**: Apply declarative `NodeSpec` JSON (`--apply-spec`) with Auto-Layout 5.0 (`relayout_subtree`), or roundtrip directly to standard Figma `.fig` v106 binary archives (`--export-fig`).
3. **Verify Visually & Epistemically**: Capture a headless GPU render (`--gpu --screenshot out.png`), run `designos-diff` when comparing against a reference, and run `python3 scripts/self_check.py`.

## Quality Bar & Anti-Scope

### It's working if...
- `python3 scripts/self_check.py` prints `ok: design-os-studio self-check passed (design-os-studio)` and exits `0`.
- `cargo test --workspace` passes all 823 automated unit, golden, and parity tests with 0 failures.
- Headless GPU/CPU screenshots render with 1:1 Figma UI3 dark chrome (`#1e1e1e`), unified Phosphor Icons, and `0.0%` static viewport CPU usage.
- Execution remains 100% local with zero external telemetry (`PRIVACY.md`).

### When NOT to use
- Multi-user cloud-hosted SaaS whiteboarding requiring real-time WebRTC cursor presence across browser tabs without local files.
- 3D rigid-body CAD modeling or mechanical STL slicing (use `design-os-3d-blender` instead).
- Paperboard folding dielines and spot-color pre-press PDF separations (use `design-os-origami` instead).
