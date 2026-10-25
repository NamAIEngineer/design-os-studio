"""Regression coverage for the TEXT design-token parity contract (#1037)."""

import json
import sys
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parent.parent
REPO = SCRIPTS.parent
sys.path.insert(0, str(SCRIPTS))

from design_os_self_evolve import (  # noqa: E402
    CORE_RUST_CONSTANTS,
    RUST_TO_DESIGN_TOKEN_MAP,
    check_rust_token_parity,
    extract_design_token_colors,
    parse_rust_theme_constants,
)


class ThemeTokenParityTests(unittest.TestCase):
    def test_text_is_checked_against_the_canonical_foreground_token(self):
        theme = (REPO / "crates/designos-app/src/theme.rs").read_text(encoding="utf-8")
        tokens = json.loads((REPO / "design/design.tokens.json").read_text(encoding="utf-8"))
        rust = parse_rust_theme_constants(theme)
        colors = extract_design_token_colors(tokens)

        self.assertIn("TEXT", CORE_RUST_CONSTANTS)
        self.assertEqual(RUST_TO_DESIGN_TOKEN_MAP["TEXT"], "bg-foreground")
        self.assertEqual(rust["TEXT"], colors["bg-foreground"])

    def test_parity_check_rejects_text_drift(self):
        theme = (REPO / "crates/designos-app/src/theme.rs").read_text(encoding="utf-8")
        tokens = json.loads((REPO / "design/design.tokens.json").read_text(encoding="utf-8"))
        drifted = theme.replace("pub const TEXT: Color32 = rgb(0xF5F5F7);", "pub const TEXT: Color32 = rgb(0xF4F4F5);")
        ok, summary, details = check_rust_token_parity(
            theme_source=drifted,
            token_data=tokens,
        )
        self.assertFalse(ok, (summary, details))
        self.assertIn("TEXT", summary)
        self.assertTrue(any("TEXT" in drift for drift in details["token_drifts"]))

    def test_repository_parity_gate_passes(self):
        ok, summary, details = check_rust_token_parity()
        self.assertTrue(ok, (summary, details))


if __name__ == "__main__":
    unittest.main()
