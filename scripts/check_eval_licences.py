#!/usr/bin/env python3
"""Validate docs/eval-licences.md, THIRD_PARTY_LICENSES.md, and client-file ban (Issue #611 / E15-23)."""

from __future__ import annotations

import argparse
import pathlib
import re
import sys
from typing import Any

try:
    import tomllib  # Python 3.11+
except ModuleNotFoundError:  # pragma: no cover
    tomllib = None  # type: ignore[assignment]

VALID_KINDS = {"dataset", "weights", "kit", "skills", "knowledge", "font"}
VALID_USES = {"eval-only-cache", "eval-redistribute", "ship-in-app", "ship-in-skills", "excluded"}
VALID_REVIEWS = {"approved", "pending", "rejected"}
REQUIRED_FIELDS = (
    "id",
    "name",
    "kind",
    "url",
    "pinned",
    "licence",
    "use",
    "attribution",
    "review",
    "reviewer",
    "reviewed_on",
    "notes",
    "consumers",
)
CONSUMER_RE = re.compile(r"^(?:E\d+-\d+|T\d+\.\d+|RD-\d+|#\d+)$")
DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
TOML_FENCE_RE = re.compile(r"```toml\s*\n(.*?)\n```", re.DOTALL)


def _fallback_parse_toml(text: str) -> dict[str, Any]:
    """Minimal TOML parser fallback for Python < 3.11 supporting [[source]] and [ban]."""
    data: dict[str, Any] = {}
    current_table: dict[str, Any] | None = None
    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("[[") and line.endswith("]]"):
            sec = line[2:-2].strip()
            if not sec:
                raise ValueError("Empty [[array]] header")
            arr = data.setdefault(sec, [])
            if not isinstance(arr, list):
                raise ValueError(f"Conflict for {sec}")
            current_table = {}
            arr.append(current_table)
            continue
        if line.startswith("[") and line.endswith("]"):
            sec = line[1:-1].strip()
            if not sec:
                raise ValueError("Empty [table] header")
            current_table = data.setdefault(sec, {})
            continue
        if "=" not in line or current_table is None:
            raise ValueError(f"Cannot parse TOML line: {raw_line}")
        k, v = line.split("=", 1)
        key = k.strip()
        val = v.strip()
        if val.startswith("[") and val.endswith("]"):
            inner = val[1:-1].strip()
            items: list[str] = []
            if inner:
                for token in re.findall(r"'[^']*'|\"[^\"]*\"", inner):
                    if token.startswith("'"):
                        items.append(token[1:-1])
                    else:
                        items.append(bytes(token[1:-1], "utf-8").decode("unicode_escape"))
            current_table[key] = items
        elif (val.startswith('"') and val.endswith('"')) or (val.startswith("'") and val.endswith("'")):
            if val.startswith("'"):
                current_table[key] = val[1:-1]
            else:
                current_table[key] = val[1:-1].replace('\\"', '"').replace("\\\\", "\\")
        else:
            raise ValueError(f"Unsupported TOML value in line: {raw_line}")
    return data


def parse_toml_str(text: str) -> dict[str, Any]:
    if tomllib is not None:
        return tomllib.loads(text)
    return _fallback_parse_toml(text)


def extract_register(md_text: str) -> tuple[dict[str, Any], list[tuple[str, str, str]]]:
    matches = TOML_FENCE_RE.findall(md_text)
    if len(matches) != 1:
        raise ValueError(f"Expected exactly 1 fenced ```toml block, found {len(matches)}")
    parsed = parse_toml_str(matches[0])

    # Parse Markdown summary table rows: | `id` | Name | `kind` | `licence` | `use` | ...
    before_toml = md_text.split("```toml", 1)[0]
    table_rows: list[tuple[str, str, str]] = []
    in_table = False
    for line in before_toml.splitlines():
        s = line.strip()
        if not s.startswith("|") or not s.endswith("|"):
            continue
        cells = [c.strip().strip("`") for c in s.strip("|").split("|")]
        if len(cells) < 5:
            continue
        if cells[0].lower() == "id" or set(cells[0]) <= {"-", ":"}:
            in_table = True
            continue
        if in_table:
            table_rows.append((cells[0], cells[3], cells[4]))
    return parsed, table_rows


def render_third_party_licenses(sources: list[dict[str, Any]]) -> str:
    ship_sources = [s for s in sources if str(s.get("use", "")).startswith("ship-")]
    lines = [
        "# Third-Party Licenses & Attributions",
        "",
        "This file is generated from `docs/eval-licences.md` by `python3 scripts/check_eval_licences.py --write`.",
        "Do not edit by hand.",
        "",
    ]
    for src in ship_sources:
        lines.extend(
            [
                f"## {src['name']} (`{src['id']}`)",
                "",
                f"- **Kind:** `{src['kind']}`",
                f"- **Distribution scope:** `{src['use']}`",
                f"- **SPDX Licence:** `{src['licence']}`",
                f"- **Upstream URL:** {src['url']}",
                f"- **Pinned Revision:** `{src['pinned']}`",
                f"- **Attribution:** {src['attribution']}",
                "",
            ]
        )
    return "\n".join(lines)


def _check_path_against_bans(
    candidate: str,
    rel_file: str,
    ban_regexes: list[tuple[str, re.Pattern[str]]],
    findings: list[str],
) -> None:
    unescaped = candidate.replace("\\\\", "\\")
    norm = unescaped.replace("\\", "/")
    for pat_str, pat in ban_regexes:
        if pat.search(candidate) or pat.search(unescaped) or pat.search(norm):
            findings.append(
                f"client-file-ban: {rel_file} references banned client/home pattern {pat_str!r} in {candidate!r}"
            )
            return
    lower = norm.lower()
    if lower.endswith(".fig") or lower.endswith(".designos"):
        basename = norm.rsplit("/", 1)[-1]
        if not basename.startswith("gen-"):
            findings.append(
                f"client-file-ban: {rel_file} references non-generated design file {candidate!r} (must use 'gen-' prefix)"
            )


def check_repo(root: pathlib.Path, write_third_party: bool = False) -> tuple[int, list[str]]:
    reg_path = root / "docs" / "eval-licences.md"
    if not reg_path.is_file():
        return 2, ["parse: docs/eval-licences.md does not exist"]

    try:
        md_text = reg_path.read_text(encoding="utf-8")
        parsed, table_rows = extract_register(md_text)
    except Exception as exc:
        return 2, [f"parse: failed to parse docs/eval-licences.md: {exc}"]

    sources = parsed.get("source")
    ban_cfg = parsed.get("ban")
    if not isinstance(sources, list) or not sources:
        return 2, ["parse: docs/eval-licences.md contains no [[source]] entries"]
    if not isinstance(ban_cfg, dict) or not isinstance(ban_cfg.get("patterns"), list) or not ban_cfg["patterns"]:
        return 2, ["parse: docs/eval-licences.md contains no [ban].patterns list"]

    findings: list[str] = []
    ban_regexes: list[tuple[str, re.Pattern[str]]] = []
    for pat_str in ban_cfg["patterns"]:
        try:
            raw_pat = str(pat_str)
            # Treat hyphenated words (e.g. "pcp-free.md") as a single token for whole-token \b...\b patterns.
            compiled_pat = re.sub(r"\\b(\w+)\\b", r"(?<![A-Za-z0-9_-])\1(?![A-Za-z0-9_-])", raw_pat)
            ban_regexes.append((raw_pat, re.compile(compiled_pat)))
        except re.error as exc:
            findings.append(f"schema: invalid [ban] regex {pat_str!r}: {exc}")

    by_id: dict[str, dict[str, Any]] = {}
    toml_triples: list[tuple[str, str, str]] = []

    for idx, src in enumerate(sources):
        if not isinstance(src, dict):
            findings.append(f"schema: source[{idx}] is not a table")
            continue
        missing = [f for f in REQUIRED_FIELDS if f not in src]
        if missing:
            findings.append(f"schema: source[{idx}] missing required fields: {', '.join(missing)}")
            continue

        sid = str(src["id"]).strip()
        licence = str(src["licence"]).strip()
        use_cls = str(src["use"]).strip()
        kind = str(src["kind"]).strip()
        review = str(src["review"]).strip()
        reviewer = str(src["reviewer"]).strip()
        reviewed_on = str(src["reviewed_on"]).strip()
        pinned = str(src["pinned"]).strip()
        attribution = str(src["attribution"]).strip()
        consumers = src["consumers"]

        if not sid:
            findings.append(f"schema: source[{idx}] has empty id")
            continue
        if sid in by_id:
            findings.append(f"schema: duplicate source id {sid!r}")
        by_id[sid] = src
        toml_triples.append((sid, licence, use_cls))

        if kind not in VALID_KINDS:
            findings.append(f"schema: {sid} has invalid kind {kind!r}")
        if use_cls not in VALID_USES:
            findings.append(f"schema: {sid} has invalid use {use_cls!r}")
        if review not in VALID_REVIEWS:
            findings.append(f"schema: {sid} has invalid review {review!r}")
        if not isinstance(consumers, list):
            findings.append(f"schema: {sid}.consumers must be a list")
            consumers = []

        # Rule 4: licence = "NONE" forces use = "excluded"
        if licence == "NONE" or not licence:
            if use_cls != "excluded":
                findings.append(
                    f"none-means-excluded: {sid} has licence {licence!r} but use = {use_cls!r}; must be 'excluded'"
                )
            if consumers:
                findings.append(f"none-means-excluded: {sid} has licence NONE and must have empty consumers")

        if use_cls == "excluded" and consumers:
            findings.append(f"consumer-exists: excluded source {sid} must not list consumers ({consumers})")

        for c in consumers:
            if not isinstance(c, str) or not CONSUMER_RE.match(c):
                findings.append(f"consumer-exists: {sid} has invalid consumer task id {c!r}")

        # Edge case 4: approved entries require reviewer, reviewed_on (YYYY-MM-DD), pinned, and attribution
        if review == "approved":
            if not reviewer or not DATE_RE.match(reviewed_on):
                findings.append(
                    f"schema: approved entry {sid} must have non-empty reviewer and YYYY-MM-DD reviewed_on"
                )
            if use_cls != "excluded" and (not pinned or not attribution):
                findings.append(f"schema: approved entry {sid} must specify pinned revision and attribution")

    # Rule 1: human summary table matches TOML [[source]] entries
    if table_rows != toml_triples:
        findings.append(
            "table-matches-toml: Markdown summary table rows (id, licence, use) do not match [[source]] entries"
        )

    # Scan crates/designos-eval and skills/vendor if present
    eval_dir = root / "crates" / "designos-eval"
    if eval_dir.exists():
        for path in sorted(eval_dir.rglob("*")):
            rel = path.relative_to(root).as_posix()
            if path.is_dir():
                continue
            _check_path_against_bans(rel, rel, ban_regexes, findings)

            if path.suffix in {".toml", ".json", ".jsonl", ".md", ".rs", ".txt"}:
                try:
                    content = path.read_text(encoding="utf-8")
                except UnicodeDecodeError:
                    continue

                # Check quoted strings and path assignments against client-file ban
                for quoted in re.findall(r'\'([^\'\n]+)\'|"([^"\n]+)"', content):
                    val = quoted[0] or quoted[1]
                    if "/" in val or "\\" in val or val.lower().endswith((".fig", ".designos")) or "vsf" in val.lower() or "pcp" in val.lower() or "untitled ui" in val.lower():
                        _check_path_against_bans(val, rel, ban_regexes, findings)

                # Also check tasks.toml source references
                if path.name == "tasks.toml" or "suites/" in rel and path.suffix == ".toml":
                    for m in re.finditer(r'^\s*source\s*=\s*["\']([^"\']+)["\']', content, re.MULTILINE):
                        src_id = m.group(1).strip()
                        entry = by_id.get(src_id)
                        if entry is None:
                            findings.append(
                                f"task-source-registered: {rel} references unregistered source {src_id!r}"
                            )
                        else:
                            if entry.get("licence") == "NONE" or entry.get("use") == "excluded":
                                findings.append(
                                    f"none-means-excluded: {rel} references excluded/unlicensed source {src_id!r}"
                                )
                            elif entry.get("review") != "approved":
                                findings.append(
                                    f"task-source-registered: {rel} references unapproved source {src_id!r} (review={entry.get('review')!r})"
                                )

    vendor_dir = root / "skills" / "vendor"
    if vendor_dir.exists():
        for path in sorted(vendor_dir.rglob("*")):
            if not path.is_file():
                continue
            rel = path.relative_to(root).as_posix()
            try:
                content = path.read_text(encoding="utf-8")
            except UnicodeDecodeError:
                continue
            for sid, src in by_id.items():
                if (src.get("licence") == "NONE" or src.get("use") == "excluded") and sid in content:
                    findings.append(
                        f"unlicensed-in-use: {sid} is referenced by {rel} but has licence {src.get('licence')}; remove the reference"
                    )

    # Rule 7: THIRD_PARTY_LICENSES.md
    expected_tp = render_third_party_licenses(sources)
    tp_path = root / "THIRD_PARTY_LICENSES.md"
    if write_third_party:
        tp_path.write_text(expected_tp, encoding="utf-8")
    else:
        if not tp_path.is_file():
            findings.append(
                "third-party-current: THIRD_PARTY_LICENSES.md is missing (run python3 scripts/check_eval_licences.py --write)"
            )
        elif tp_path.read_text(encoding="utf-8") != expected_tp:
            findings.append(
                "third-party-current: THIRD_PARTY_LICENSES.md is stale (run python3 scripts/check_eval_licences.py --write)"
            )

    return (1 if findings else 0), findings


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true", help="Regenerate THIRD_PARTY_LICENSES.md")
    parser.add_argument(
        "--root",
        type=pathlib.Path,
        default=pathlib.Path(__file__).resolve().parent.parent,
        help="Repository root directory",
    )
    args = parser.parse_args(argv)
    code, findings = check_repo(args.root, write_third_party=args.write)
    for f in findings:
        print(f)
    return code


if __name__ == "__main__":
    sys.exit(main())
