"""Parity of non-colour design tokens (radius, spacing, size) with `theme.rs`.

G01 in `design_os_self_evolve.py` already pins the colour constants. This module
extends it to every dimensional token that has a Rust constant. The mapping is
explicit: `font-size-title` is 12 while `FONT_SIZE_TITLE` is 20, so names are
never matched by similarity.

A token with no entry in `TOKEN_TO_RUST` is listed as unmapped (information
only). A mapped constant that is not a plain numeric literal fails, so an alias
or expression cannot silently escape the check.
"""

from __future__ import annotations

import re
from typing import Any, Dict, List, Optional

from rust_literal_scan import parse_theme_consts

# (token group, token name) -> Rust constants that must all equal the token value.
TOKEN_TO_RUST: Dict[str, List[str]] = {
    "radius.radius-xs": ["RADIUS_SMALL"],
    "radius.radius-md": ["RADIUS_MEDIUM"],
    "radius.radius-pill": ["RADIUS_LARGE"],
    "typography.font-size-small": ["FONT_SIZE_SMALL"],
    "typography.font-size-body": ["FONT_SIZE_BODY"],
    "typography.font-size-heading": ["FONT_SIZE_LARGE"],
    # The token is named hero; the constant is still called TITLE (it is 20 px).
    "typography.font-size-hero": ["FONT_SIZE_TITLE"],
    "spacing.rail-width": ["RAIL_WIDTH"],
    "spacing.panel-width": ["LEFT_PANEL_WIDTH", "RIGHT_PANEL_WIDTH"],
}

# Drift that exists today and is fixed by a later, separately ruled change.
# An entry passes only while both sides still hold exactly the recorded values,
# and fails as stale once they agree, so the entry has to be deleted with the fix.
KNOWN_DRIFT: Dict[str, Dict[str, Any]] = {
    "radius.radius-md": {
        "constant": "RADIUS_MEDIUM",
        "rust": 5.0,
        "token": 6.0,
        "reason": "control radius is 6 in the tokens and 5 in theme.rs; the pixel change ships with the radius standard",
    },
}

_PX = re.compile(r"^\s*(-?\d+(?:\.\d+)?)\s*(?:px)?\s*$")


def token_value(tokens: Dict[str, Any], path: str) -> Optional[float]:
    """Numeric value of a DTCG dimension token (`6px` -> 6.0); None if absent or not a number."""
    group, _, name = path.partition(".")
    leaf = tokens.get(group, {}).get(name)
    raw = leaf.get("$value") if isinstance(leaf, dict) else None
    m = _PX.match(str(raw)) if raw is not None else None
    return float(m.group(1)) if m else None


def _all_dimension_tokens(tokens: Dict[str, Any]) -> List[str]:
    out = []
    for group in ("radius", "spacing", "typography"):
        for name, leaf in tokens.get(group, {}).items():
            if isinstance(leaf, dict) and leaf.get("$type") == "dimension":
                out.append(f"{group}.{name}")
    return out


def check_non_colour_parity(
    theme_src: str,
    tokens: Dict[str, Any],
    mapping: Optional[Dict[str, List[str]]] = None,
    known_drift: Optional[Dict[str, Dict[str, Any]]] = None,
) -> Dict[str, Any]:
    """Compare mapped dimension tokens with `theme.rs`; pure, so tests can inject drift."""
    mapping = TOKEN_TO_RUST if mapping is None else mapping
    known_drift = KNOWN_DRIFT if known_drift is None else known_drift
    consts = parse_theme_consts(theme_src)
    verified: Dict[str, str] = {}
    drifts: List[str] = []
    known: List[str] = []
    stale: List[str] = []
    problems: List[str] = []

    for path, names in mapping.items():
        tok = token_value(tokens, path)
        if tok is None:
            problems.append(f"{path} missing or not numeric in design.tokens.json")
            continue
        for name in names:
            rust = consts.get(name)
            if rust is None:
                problems.append(f"{name} is not a plain numeric constant in theme.rs")
                continue
            exempt = known_drift.get(path)
            if exempt and exempt["constant"] == name:
                if rust == exempt["rust"] and tok == exempt["token"]:
                    known.append(f"{name} {rust:g} != {path} {tok:g} ({exempt['reason']})")
                elif rust == tok:
                    stale.append(f"{path}: {name} now matches the token; delete its KNOWN_DRIFT entry")
                else:
                    drifts.append(
                        f"{name} ({rust:g}) != {path} ({tok:g}); the pinned drift was {exempt['rust']:g} vs {exempt['token']:g}"
                    )
                continue
            if rust != tok:
                drifts.append(f"{name} ({rust:g}) != {path} ({tok:g})")
            else:
                verified[name] = f"{rust:g} == {path}"
    for path in known_drift:
        if path not in mapping:
            stale.append(f"{path}: KNOWN_DRIFT entry has no mapping row")

    mapped = set(mapping)
    return {
        "ok": not (drifts or stale or problems),
        "verified": verified,
        "drifts": drifts,
        "known_drift": known,
        "stale_exemptions": stale,
        "problems": problems,
        "unmapped_tokens": sorted(set(_all_dimension_tokens(tokens)) - mapped),
    }
