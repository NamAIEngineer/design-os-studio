"""Regression tests for release Latest selection."""
from __future__ import annotations

import os
import subprocess
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
SCRIPT = ROOT / "scripts" / "resolve-release-latest.sh"
WORKFLOW = ROOT / ".github" / "workflows" / "release.yml"


class ReleaseLatestTests(unittest.TestCase):
    def resolve(self, current: str, tags: str) -> str:
        env = os.environ | {"RELEASE_TAGS": tags, "GH_REPO": "jangtrinh/design-os-studio"}
        result = subprocess.run([str(SCRIPT), current], env=env, capture_output=True, text=True, check=True)
        return result.stdout.strip()

    def test_backfilled_older_stable_release_does_not_become_latest(self):
        self.assertEqual(self.resolve("0.7.1", "v0.7.5\nv0.7.0\nv0.6.5"), "false")

    def test_newer_stable_release_becomes_latest(self):
        self.assertEqual(self.resolve("0.8.0", "v0.7.5\nv0.7.0"), "true")

    def test_prerelease_never_becomes_latest(self):
        self.assertEqual(self.resolve("0.8.0-rc.1", "v0.7.5"), "false")

    def test_workflow_passes_computed_flag_to_release_action(self):
        text = WORKFLOW.read_text()
        self.assertIn("scripts/resolve-release-latest.sh", text)
        self.assertIn("id: latest", text)
        self.assertIn("make_latest: ${{ steps.latest.outputs.make_latest }}", text)


if __name__ == "__main__":
    unittest.main()
