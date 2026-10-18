#!/usr/bin/env python3
"""Unit and contract tests for `scripts/check_vendor_skills.py` (E15-46 §10 / #636)."""

from __future__ import annotations

import hashlib
import shutil
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import check_vendor_skills as cvs  # noqa: E402


class TestCheckVendorSkills(unittest.TestCase):
    def _copy_vendor_fixture(self) -> Path:
        tmp = Path(tempfile.mkdtemp(prefix="dos-636-test-"))
        self.addCleanup(lambda: shutil.rmtree(tmp, ignore_errors=True))
        shutil.copytree(
            REPO_ROOT / "skills" / "vendor",
            tmp / "skills" / "vendor",
        )
        return tmp

    def test_licence_and_pin_match_upstream(self) -> None:
        """AC1: `verify` exits 0; LICENSE SHA-1 equals recorded upstream hash; commit equals 4c66b1f…"""
        code, errors = cvs.cmd_verify(REPO_ROOT, online=False)
        self.assertEqual(code, 0, f"Expected verify exit 0, got errors: {errors}")

        jakub_dir = REPO_ROOT / "skills" / "vendor" / "jakubkrehel-skills"
        upstream_meta = tomllib.loads((jakub_dir / "UPSTREAM.toml").read_text(encoding="utf-8"))
        self.assertTrue(upstream_meta["commit"].startswith("4c66b1f"))
        self.assertEqual(upstream_meta["commit"], cvs.PINNED_COMMIT)

        lic_sha1 = hashlib.sha1((jakub_dir / "LICENSE").read_bytes()).hexdigest()
        upstream_lic_sha1 = hashlib.sha1((jakub_dir / "upstream" / "LICENSE").read_bytes()).hexdigest()
        self.assertEqual(lic_sha1, upstream_meta["license_sha1"])
        self.assertEqual(upstream_lic_sha1, upstream_meta["license_sha1"])

    def test_every_rule_mapped(self) -> None:
        """AC2: `map-coverage` exits 0; deleting one entry -> `unmapped-rule`."""
        code, errors = cvs.cmd_map_coverage(REPO_ROOT)
        self.assertEqual(code, 0, f"Expected map-coverage exit 0, got errors: {errors}")

        tmp = self._copy_vendor_fixture()
        map_path = tmp / "skills" / "vendor" / "jakubkrehel-skills" / "rule-map.toml"
        raw = map_path.read_text(encoding="utf-8")
        # Remove the last [[rule]] block
        last_idx = raw.rfind("[[rule]]")
        self.assertGreater(last_idx, 0)
        map_path.write_text(raw[:last_idx], encoding="utf-8")

        code_del, errors_del = cvs.cmd_map_coverage(tmp)
        self.assertEqual(code_del, 1)
        self.assertTrue(
            any("unmapped-rule" in e for e in errors_del),
            f"Expected 'unmapped-rule' in {errors_del}",
        )

    def test_no_unlicensed_skill_bundled(self) -> None:
        """AC3: adding `skills/vendor/transitions-dev/x.md` -> `verify` exits 1 `unlicensed-file`."""
        tmp = self._copy_vendor_fixture()
        bad_dir = tmp / "skills" / "vendor" / "transitions-dev"
        bad_dir.mkdir(parents=True, exist_ok=True)
        (bad_dir / "x.md").write_text("# Unlicensed transition skill\n", encoding="utf-8")

        code, errors = cvs.cmd_verify(tmp, online=False)
        self.assertEqual(code, 1)
        self.assertTrue(
            any("unlicensed-file" in e and "transitions-dev/x.md" in e for e in errors),
            f"Expected 'unlicensed-file' for transitions-dev/x.md in {errors}",
        )

    def test_unknown_lint_id_fails(self) -> None:
        """Mapping a rule to `lint:not-a-rule` -> exit 1 `unknown-lint-id`."""
        tmp = self._copy_vendor_fixture()
        map_path = tmp / "skills" / "vendor" / "jakubkrehel-skills" / "rule-map.toml"
        raw = map_path.read_text(encoding="utf-8")
        mutated = raw.replace('map = "lint:contrast"', 'map = "lint:not-a-rule"', 1)
        self.assertNotEqual(raw, mutated)
        map_path.write_text(mutated, encoding="utf-8")

        code, errors = cvs.cmd_map_coverage(tmp)
        self.assertEqual(code, 1)
        self.assertTrue(
            any("unknown-lint-id" in e and "not-a-rule" in e for e in errors),
            f"Expected 'unknown-lint-id' in {errors}",
        )

    def test_canvas_rules_generated(self) -> None:
        """`canvas-rules.md` equals the generated text and begins with the attribution header."""
        jakub_dir = REPO_ROOT / "skills" / "vendor" / "jakubkrehel-skills"
        map_doc = tomllib.loads((jakub_dir / "rule-map.toml").read_text(encoding="utf-8"))
        expected = cvs.render_canvas_rules(map_doc["rule"])
        actual = (jakub_dir / "canvas-rules.md").read_text(encoding="utf-8")
        self.assertTrue(actual.startswith(f"<!-- {cvs.ATTRIBUTION_HEADER} -->"))
        self.assertEqual(actual, expected)

        code, msgs = cvs.cmd_extract(REPO_ROOT, write=False)
        self.assertEqual(code, 0, f"Expected extract exit 0, got: {msgs}")

    def test_fail_before_when_vendor_folder_missing(self) -> None:
        """When `skills/vendor` does not exist, `verify` exits 2."""
        tmp = Path(tempfile.mkdtemp(prefix="dos-636-empty-"))
        self.addCleanup(lambda: shutil.rmtree(tmp, ignore_errors=True))
        code, _ = cvs.cmd_verify(tmp, online=False)
        self.assertEqual(code, 2)


if __name__ == "__main__":
    unittest.main()
