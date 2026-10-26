"""Regression checks for the homepage's JavaScript-owned entrance animation."""
import re
import unittest
from pathlib import Path


ROOT = Path(__file__).parents[2]
STORY_CSS = ROOT / "docs" / "assets" / "v2-story.css"
CORE_JS = ROOT / "docs" / "assets" / "v2-core.js"


class HomepageMotionTests(unittest.TestCase):
    def test_nav_links_are_not_hidden_by_the_static_preflight(self):
        css = STORY_CSS.read_text()
        preflight = re.search(
            r"@media \(prefers-reduced-motion: no-preference\) \{(?P<body>.*?)\n\}",
            css,
            re.DOTALL,
        )
        self.assertIsNotNone(preflight)
        self.assertNotIn("html.v2-pre .topbar .navlinks li", preflight.group("body"))

    def test_gsap_entrance_still_owns_nav_link_animation(self):
        js = CORE_JS.read_text()
        self.assertIn(
            "pick('.topbar .brand, .topbar .navlinks li, .topbar .btn')",
            js,
        )


if __name__ == "__main__":
    unittest.main()
