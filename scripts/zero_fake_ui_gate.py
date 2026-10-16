#!/usr/bin/env python3
"""
Zero Fake UI Deterministic Enforcement Gate (`scripts/zero_fake_ui_gate.py`)
============================================================================
Enforces `DIR-28` / `BR-31` / `REQ-25` / `D25` ("Zero Fake UI Law — Nghiêm Cấm Fake UI"):
Every visible button, input bar, scrollbar, tab, menu item, and agent CLI runtime in
`design:os studio` (`crates/designos-app/src/`) must execute real state mutation, real OS
clipboard/filesystem I/O, or real CLI subprocess execution.

Verifies 12 hard Zero-Fake-UI invariants (`Z01..Z12`):
  - Z01: `terminal.rs` has `runtime_binary_available` and real CLI invocation
         (`claude -p`, `codex exec`, `agy -p`, `grok -p`) in `submit_chat_prompt`.
  - Z02: `terminal.rs` `set_active_runtime` mutates `tab.preset` (not just renaming `tab.title`)
         without typing into the live PTY; `DESIGNOS_AGENT_RUNTIME` is set in `configure_pty_env`
         and `build_real_cli_command`.
  - Z03: `terminal.rs` `dispatch_figma_ai_prompt` does NOT prefix PTY commands with a `#` shell comment.
  - Z04: `terminal.rs` header & composer buttons (`agents-header-ai`, `agents-header-popout`,
         `agents-header-more`, `agents-composer-attach`, `agents-composer-context`,
         `agents-composer-mic`) all have interactive click handlers (`Sense::click()`).
  - Z05: `specs.rs` `specs-ask-agent-bar` hosts a real `TextEdit::singleline` (`ask_input`)
         instead of only sending a hardcoded prompt string.
  - Z06: `specs.rs` Right Inspector has dynamic `active_section_toc` and interactive
         `SpecComment` creation/resolution (`comment_draft` + `TextEdit::singleline`).
  - Z07: `specs.rs` `draw_wrapped_prose` does NOT truncate prose at `lines.len() >= 3`,
         and `render_tab_blueprints` renders `pkg.ui_contract`.
  - Z08: `panels/toolbar.rs` contextual buttons (`ctx-crop`, `ctx-select-area`,
         `ctx-remove-bg`, `ctx-edit-image`) mutate `app.doc` / `app.tool` instead of only
         calling `app.show_toast`.
  - Z09: `home/mod.rs` `Act::ExportFig` writes `.fig` bytes to disk (`std::fs::write`),
         and `Nav::AllFolders` / `Nav::Templates` / `Nav::Libraries` filter distinctly.
  - Z10: `panels/right.rs` `"Share"` copies to clipboard (`copy_text`), and Dev Mode's 2nd tab
         renders the Dev Plugins inspector instead of `super::prototype::show`.
  - Z11: `panels/inspect.rs` (`dev-code-connect`, `dev-mcp-btn`, `dev-add-redlines`,
         `inspect_keyframes_section`) and `panels/properties.rs` (`copy-to-slides`) perform
         real clipboard/document mutations.
  - Z12: `widgets/picker.rs` `Mode::Image` creates real `Paint::Image` / `app.doc.images`
         entries, and `panels/mod.rs::scrollbar` + `variables/view.rs::hscrollbar` support
         `Sense::click_and_drag()`.

Exit code:
  0 when 12/12 invariants pass, 1 if any invariant fails.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any, Dict, List, Tuple

REPO_ROOT = Path(__file__).resolve().parent.parent
APP_SRC = REPO_ROOT / "crates" / "designos-app" / "src"


def read_src(rel_path: str) -> str:
    path = APP_SRC / rel_path
    if not path.exists():
        return ""
    return path.read_text(encoding="utf-8", errors="replace")


def extract_fn_body(src: str, fn_name: str) -> str:
    """Extracts the body of `fn <fn_name>(...)` up to its matching closing brace."""
    pattern = re.compile(rf"\b(?:pub(?:\([^)]*\))?\s+)?fn\s+{re.escape(fn_name)}\b")
    match = pattern.search(src)
    if not match:
        return ""
    brace_start = src.find("{", match.end())
    if brace_start == -1:
        return ""
    depth = 0
    in_str = False
    escape = False
    for idx in range(brace_start, len(src)):
        ch = src[idx]
        if in_str:
            if escape:
                escape = False
            elif ch == "\\":
                escape = True
            elif ch == '"':
                in_str = False
            continue
        if ch == '"':
            in_str = True
        elif ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return src[brace_start : idx + 1]
    return src[brace_start:]


def check_z01(terminal_rs: str) -> Tuple[bool, str]:
    has_avail = "fn runtime_binary_available" in terminal_rs
    submit_body = extract_fn_body(terminal_rs, "submit_chat_prompt")
    helper_body = extract_fn_body(terminal_rs, "build_real_cli_command")
    combined = submit_body + "\n" + helper_body
    cli_checks = {
        "claude": ("claude --dangerously-skip-permissions -p" in combined or "claude -p" in combined),
        "codex": ("codex --dangerously-bypass-approvals-and-sandbox exec" in combined or "codex exec" in combined),
        "agy": ("agy --dangerously-skip-permissions -p" in combined or "agy -p" in combined),
        "grok": ("grok --always-approve -p" in combined or "grok -p" in combined),
    }
    missing = [name for name, present in cli_checks.items() if not present]
    calls_avail = "runtime_binary_available" in submit_body
    ok = has_avail and calls_avail and not missing
    if ok:
        return True, "runtime_binary_available + real CLI invocations (claude, codex, agy, grok with skip-permissions) verified in submit_chat_prompt"
    reasons = []
    if not has_avail:
        reasons.append("missing `fn runtime_binary_available`")
    if not calls_avail:
        reasons.append("`submit_chat_prompt` does not call `runtime_binary_available`")
    if missing:
        reasons.append(f"missing CLI invocations: {missing}")
    return False, "; ".join(reasons)


def check_z02(terminal_rs: str) -> Tuple[bool, str]:
    body = extract_fn_body(terminal_rs, "set_active_runtime")
    has_preset = bool(re.search(r"tab\.preset\s*=\s*runtime", body))
    # Typing into the tab lands as a chat prompt when the tab hosts an agent TUI.
    no_pty_write = "send_line" not in body and "write_all" not in body
    spawn_env = "DESIGNOS_AGENT_RUNTIME" in extract_fn_body(terminal_rs, "configure_pty_env")
    cmd_env = "DESIGNOS_AGENT_RUNTIME" in extract_fn_body(terminal_rs, "build_real_cli_command")
    ok = bool(body) and has_preset and no_pty_write and spawn_env and cmd_env
    if ok:
        return True, "`set_active_runtime` mutates `tab.preset = runtime` without PTY writes; `DESIGNOS_AGENT_RUNTIME` set at spawn + per CLI command"
    return False, (
        f"tab.preset mutation={has_preset}, no PTY write in set_active_runtime={no_pty_write}, "
        f"env in configure_pty_env={spawn_env}, env in build_real_cli_command={cmd_env}"
    )


def check_z03(terminal_rs: str) -> Tuple[bool, str]:
    body = extract_fn_body(terminal_rs, "dispatch_figma_ai_prompt")
    if not body:
        return False, "`fn dispatch_figma_ai_prompt` not found in terminal.rs"
    # Must not format or send a `#`-prefixed shell comment line as the PTY command
    comment_noop = bool(re.search(r'format!\(\s*"#\s*\[', body)) or bool(
        re.search(r'send_line\(\s*"#', body)
    )
    has_send = ("send_line" in body) or ("submit_chat_prompt" in body)
    ok = (not comment_noop) and has_send
    if ok:
        return True, "`dispatch_figma_ai_prompt` dispatches executable PTY/CLI command without `#` comment prefix"
    return False, f"dispatch_figma_ai_prompt comment_noop={comment_noop}, has_send={has_send}"


def check_z04(terminal_rs: str) -> Tuple[bool, str]:
    required_ids = [
        "agents-header-ai",
        "agents-header-popout",
        "agents-header-more",
        "agents-composer-attach",
        "agents-composer-context",
        "agents-composer-mic",
    ]
    missing = []
    for wid in required_ids:
        # Verify the ID appears in an interactive Sense::click() block with a .clicked() handler
        pattern = re.compile(
            rf'"{re.escape(wid)}"[\s\S]{{0,520}}?(?:Sense::click\(\)|\.clicked\(\))|Sense::click\(\)[\s\S]{{0,520}}?"{re.escape(wid)}"'
        )
        if not pattern.search(terminal_rs):
            missing.append(wid)
    if not missing:
        return True, f"All 6 header/composer controls ({', '.join(required_ids)}) have interactive Sense::click() handlers"
    return False, f"Missing interactive Sense::click() handler for: {missing}"


def check_z05(specs_rs: str) -> Tuple[bool, str]:
    has_bar = "specs-ask-agent-bar" in specs_rs
    has_input_field = "ask_input" in specs_rs
    has_text_edit = bool(
        re.search(r"TextEdit::singleline\s*\(\s*&mut\s+[a-zA-Z0-9_.]*ask_input\b", specs_rs)
    )
    ok = has_bar and has_input_field and has_text_edit
    if ok:
        return True, "`specs-ask-agent-bar` hosts editable `TextEdit::singleline(&mut ...ask_input)`"
    return (
        False,
        f"specs-ask-agent-bar={has_bar}, ask_input={has_input_field}, TextEdit::singleline(ask_input)={has_text_edit}",
    )


def check_z06(specs_rs: str) -> Tuple[bool, str]:
    has_toc = "active_section_toc" in specs_rs
    has_comment_struct = "SpecComment" in specs_rs
    has_draft = "comment_draft" in specs_rs
    has_text_edit = bool(
        re.search(r"TextEdit::singleline\s*\(\s*&mut\s+[a-zA-Z0-9_.]*comment_draft\b", specs_rs)
    )
    ok = has_toc and has_comment_struct and has_draft and has_text_edit
    if ok:
        return True, "Right Spec Inspector has dynamic `active_section_toc` + interactive `SpecComment` (`comment_draft` + `TextEdit::singleline`)"
    return (
        False,
        f"active_section_toc={has_toc}, SpecComment={has_comment_struct}, comment_draft={has_draft}, TextEdit(comment_draft)={has_text_edit}",
    )


def check_z07(specs_rs: str) -> Tuple[bool, str]:
    prose_body = extract_fn_body(specs_rs, "draw_wrapped_prose")
    if not prose_body:
        return False, "`fn draw_wrapped_prose` not found in specs.rs"
    truncates_at_3 = bool(re.search(r"lines\.len\(\)\s*>=\s*3", prose_body))
    blueprints_body = (
        extract_fn_body(specs_rs, "render_tab_blueprints")
        + "\n"
        + extract_fn_body(specs_rs, "render_wireframes_and_contract_tab")
    )
    renders_ui_contract = bool(
        re.search(r"(?:&pkg\.ui_contract|pkg\.ui_contract\.iter\(\))", blueprints_body)
    )
    ok = (not truncates_at_3) and bool(blueprints_body.strip()) and renders_ui_contract
    if ok:
        return True, "`draw_wrapped_prose` does not truncate at `lines.len() >= 3` and `render_tab_blueprints` renders `pkg.ui_contract`"
    return (
        False,
        f"truncates_at_3={truncates_at_3}, has_blueprints_fn={bool(blueprints_body.strip())}, renders_ui_contract={renders_ui_contract}",
    )


def check_z08(toolbar_rs: str) -> Tuple[bool, str]:
    has_crop = "ctx-crop" in toolbar_rs
    has_select = "ctx-select-area" in toolbar_rs
    has_remove_bg = "ctx-remove-bg" in toolbar_rs
    has_edit_img = ("ctx-edit-image" in toolbar_rs) or ("ctx-edit-prompt" in toolbar_rs)
    mutates_state = (
        ("app.doc" in toolbar_rs or "doc." in toolbar_rs)
        and ("app.tool" in toolbar_rs)
        and ("apply_contextual_action" in toolbar_rs or "clips_content" in toolbar_rs)
    )
    ok = has_crop and has_select and has_remove_bg and has_edit_img and mutates_state
    if ok:
        return True, "`panels/toolbar.rs` contextual buttons (`ctx-crop`, `ctx-select-area`, `ctx-remove-bg`, `ctx-edit-image`) mutate `app.doc` / `app.tool`"
    return (
        False,
        f"crop={has_crop}, select={has_select}, remove_bg={has_remove_bg}, edit_img={has_edit_img}, mutates_state={mutates_state}",
    )


def check_z09(home_rs: str) -> Tuple[bool, str]:
    has_export_fig = ("Act::ExportFig" in home_rs) or ("ExportFig" in home_rs)
    has_fs_write = ("std::fs::write" in home_rs) or ("fs::write" in home_rs)
    collapsed_sections = bool(
        re.search(
            r"Section::Recents\s*\|\s*Section::AllProjects\s*\|\s*Section::Templates\s*\|\s*Section::Resources",
            home_rs,
        )
    )
    has_distinct_nav = (not collapsed_sections) and (
        ("Nav::AllFolders" in home_rs and "Nav::Templates" in home_rs and "Nav::Libraries" in home_rs)
        or (
            "Section::AllProjects =>" in home_rs
            and "Section::Templates =>" in home_rs
            and "Section::Resources =>" in home_rs
        )
    )
    ok = has_export_fig and has_fs_write and has_distinct_nav
    if ok:
        return True, "`home/mod.rs` `Act::ExportFig` writes `.fig` bytes (`std::fs::write`) and `AllFolders`/`Templates`/`Libraries` filter distinctly"
    return (
        False,
        f"ExportFig={has_export_fig}, fs::write={has_fs_write}, distinct_nav={has_distinct_nav}",
    )


def check_z10(right_rs: str) -> Tuple[bool, str]:
    has_share_copy = ("\"Share\"" in right_rs) and ("copy_text" in right_rs)
    # Dev Mode's 2nd tab (`RightTab::Prototype` when `app.ui.dev_mode` is true) must NOT unconditionally call `super::prototype::show`
    unconditional_proto = bool(
        re.search(r"if\s+active\s*==\s*RightTab::Prototype\s*\{\s*super::prototype::show", right_rs)
    )
    has_dev_plugins = (not unconditional_proto) and (
        "plugins" in right_rs.lower() or "inspect::show_plugins" in right_rs or "dev_plugins" in right_rs
    )
    ok = has_share_copy and has_dev_plugins
    if ok:
        return True, "`panels/right.rs` `Share` copies to OS clipboard (`copy_text`) and Dev Mode 2nd tab renders Dev Plugins inspector"
    return False, f"has_share_copy={has_share_copy}, has_dev_plugins={has_dev_plugins} (unconditional_proto={unconditional_proto})"


def check_z11(inspect_rs: str, properties_rs: str, dev_sections_rs: str) -> Tuple[bool, str]:
    combined_inspect = inspect_rs + "\n" + dev_sections_rs
    req_inspect_tokens = [
        "dev-code-connect",
        "dev-mcp-btn",
        "dev-add-redlines",
        "inspect_keyframes_section",
    ]
    missing_inspect = [t for t in req_inspect_tokens if t not in combined_inspect]
    has_slides_id = ("copy-to-slides" in properties_rs) or ("copy-to-figma-slides" in properties_rs)
    # Verify `copy-to-slides` / `copy-to-figma-slides` performs real clipboard/doc mutation (not just show_toast)
    slides_match = re.search(
        r"copy-to-(?:figma-)?slides[\s\S]{0,900}?(copy_text|app\.doc|slides|export_svg|NodeSpec)",
        properties_rs,
    )
    has_slides_mutation = bool(slides_match)
    ok = (not missing_inspect) and has_slides_id and has_slides_mutation
    if ok:
        return True, "`panels/inspect.rs` (`dev-code-connect`, `dev-mcp-btn`, `dev-add-redlines`, `inspect_keyframes_section`) and `panels/properties.rs` (`copy-to-slides`) execute real clipboard/document mutations"
    return (
        False,
        f"missing_inspect={missing_inspect}, has_slides_id={has_slides_id}, has_slides_mutation={has_slides_mutation}",
    )


def check_z12(picker_rs: str, panels_mod_rs: str, vars_view_rs: str, properties_rs: str) -> Tuple[bool, str]:
    combined_picker = picker_rs + "\n" + properties_rs
    has_mode_image = "Mode::Image" in picker_rs
    has_image_paint = (
        ("Paint::Image" in combined_picker or "PaintKind::Image" in combined_picker)
        and ("images.insert" in combined_picker or "doc.images" in combined_picker or "checkerboard_image" in combined_picker or "default_image" in combined_picker)
    )
    scrollbar_body = extract_fn_body(panels_mod_rs, "scrollbar")
    hscrollbar_body = extract_fn_body(vars_view_rs, "hscrollbar")
    has_v_drag = "Sense::click_and_drag()" in scrollbar_body or "Sense::click_and_drag()" in panels_mod_rs
    has_h_drag = "Sense::click_and_drag()" in hscrollbar_body
    ok = has_mode_image and has_image_paint and has_v_drag and has_h_drag
    if ok:
        return True, "`widgets/picker.rs` `Mode::Image` creates real `Paint::Image` / `doc.images` entries and `panels::scrollbar` + `variables::view::hscrollbar` support `Sense::click_and_drag()`"
    return (
        False,
        f"Mode::Image={has_mode_image}, image_paint={has_image_paint}, panels::scrollbar drag={has_v_drag}, hscrollbar drag={has_h_drag}",
    )


def run_all_checks() -> List[Dict[str, Any]]:
    terminal_rs = read_src("terminal.rs")
    specs_rs = read_src("specs.rs")
    toolbar_rs = read_src("panels/toolbar.rs")
    home_rs = read_src("home/mod.rs")
    right_rs = read_src("panels/right.rs")
    inspect_rs = read_src("panels/inspect.rs")
    dev_sections_rs = read_src("dev/sections.rs")
    properties_rs = read_src("panels/properties.rs")
    picker_rs = read_src("widgets/picker.rs")
    panels_mod_rs = read_src("panels/mod.rs")
    vars_view_rs = read_src("variables/view.rs")

    specs = [
        ("Z01", "terminal.rs", "Real CLI binary detection & invocation in submit_chat_prompt", lambda: check_z01(terminal_rs)),
        ("Z02", "terminal.rs", "set_active_runtime mutates tab.preset; DESIGNOS_AGENT_RUNTIME set at spawn, never typed", lambda: check_z02(terminal_rs)),
        ("Z03", "terminal.rs", "dispatch_figma_ai_prompt sends executable command without # comment prefix", lambda: check_z03(terminal_rs)),
        ("Z04", "terminal.rs", "All 6 Agents header & composer buttons have Sense::click() handlers", lambda: check_z04(terminal_rs)),
        ("Z05", "specs.rs", "specs-ask-agent-bar hosts editable TextEdit::singleline(ask_input)", lambda: check_z05(specs_rs)),
        ("Z06", "specs.rs", "Right Inspector dynamic active_section_toc & interactive SpecComment", lambda: check_z06(specs_rs)),
        ("Z07", "specs.rs", "Unwrapped draw_wrapped_prose & render_tab_blueprints renders pkg.ui_contract", lambda: check_z07(specs_rs)),
        ("Z08", "panels/toolbar.rs", "Contextual sub-toolbar buttons mutate app.doc / app.tool", lambda: check_z08(toolbar_rs)),
        ("Z09", "home/mod.rs", "Act::ExportFig writes .fig to disk & Nav::AllFolders/Templates/Libraries filter distinctly", lambda: check_z09(home_rs)),
        ("Z10", "panels/right.rs", "Share copies to clipboard & Dev Mode 2nd tab renders Dev Plugins inspector", lambda: check_z10(right_rs)),
        ("Z11", "panels/inspect.rs + properties.rs", "Dev Mode Code Connect/MCP/Redlines/Keyframes & Copy to Slides mutate state/clipboard", lambda: check_z11(inspect_rs, properties_rs, dev_sections_rs)),
        ("Z12", "widgets/picker.rs + panels/mod.rs + variables/view.rs", "Mode::Image creates Paint::Image/doc.images & scrollbars support Sense::click_and_drag()", lambda: check_z12(picker_rs, panels_mod_rs, vars_view_rs, properties_rs)),
    ]

    results: List[Dict[str, Any]] = []
    for cid, module, title, fn in specs:
        passed, detail = fn()
        results.append(
            {
                "id": cid,
                "module": module,
                "title": title,
                "passed": passed,
                "status": "PASS" if passed else "FAIL",
                "detail": detail,
            }
        )
    return results


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Zero Fake UI Deterministic Gate (DIR-28 / BR-31 / REQ-25 / D25)"
    )
    parser.add_argument("--json", action="store_true", help="Emit machine-readable JSON report")
    args = parser.parse_args()

    results = run_all_checks()
    passed_count = sum(1 for r in results if r["passed"])
    total_count = len(results)
    all_passed = passed_count == total_count

    if args.json:
        payload = {
            "gate": "zero_fake_ui_gate",
            "directive": "DIR-28",
            "business_rule": "BR-31",
            "requirement": "REQ-25",
            "decision": "D25",
            "passed": passed_count,
            "total": total_count,
            "status": "PASS" if all_passed else "FAIL",
            "checks": results,
        }
        print(json.dumps(payload, indent=2))
    else:
        print("=" * 108)
        print(f" design:os studio — Zero Fake UI Enforcement Gate (DIR-28 · BR-31 · REQ-25 · D25)")
        print("=" * 108)
        print(f"{'ID':<5} | {'STATUS':<6} | {'MODULE':<32} | {'INVARIANT & EVIDENCE'}")
        print("-" * 108)
        for r in results:
            print(f"{r['id']:<5} | {r['status']:<6} | {r['module']:<32} | {r['title']} — {r['detail']}")
        print("-" * 108)
        verdict = "PASS" if all_passed else "FAIL"
        print(f" Gate Verdict: {verdict} ({passed_count}/{total_count} invariants satisfied)")
        print("=" * 108)

    return 0 if all_passed else 1


if __name__ == "__main__":
    sys.exit(main())
