#!/usr/bin/env python3
"""Unit tests for scripts/check_eval_licences.py (Issue #611 / Task E15-23)."""

from __future__ import annotations

import pathlib
import shutil
import sys
import tempfile
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import check_eval_licences as cel  # noqa: E402

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent


class TestCheckEvalLicences(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.tmp.name)
        (self.root / "docs").mkdir(parents=True, exist_ok=True)
        shutil.copy2(REPO_ROOT / "docs" / "eval-licences.md", self.root / "docs" / "eval-licences.md")
        cel.check_repo(self.root, write_third_party=True)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_repo_register_is_valid(self) -> None:
        code, findings = cel.check_repo(REPO_ROOT)
        self.assertEqual(code, 0, f"Unexpected findings on repo root: {findings}")

    def test_missing_register_exits_two(self) -> None:
        (self.root / "docs" / "eval-licences.md").unlink()
        code, findings = cel.check_repo(self.root)
        self.assertEqual(code, 2)
        self.assertTrue(any(f.startswith("parse:") for f in findings))

    def test_every_eval_source_registered(self) -> None:
        s1 = self.root / "crates" / "designos-eval" / "suites" / "s1"
        s1.mkdir(parents=True, exist_ok=True)
        (s1 / "tasks.toml").write_text(
            '[[task]]\nid = "t1"\nsource = "design2code"\n[[task]]\nid = "t2"\nsource = "uicrit"\n',
            encoding="utf-8",
        )
        code, findings = cel.check_repo(self.root)
        self.assertEqual(code, 0, findings)

        (s1 / "tasks.toml").write_text(
            '[[task]]\nid = "t1"\nsource = "unknown-dataset"\n',
            encoding="utf-8",
        )
        code, findings = cel.check_repo(self.root)
        self.assertEqual(code, 1)
        self.assertTrue(any(f.startswith("task-source-registered:") for f in findings), findings)

    def test_unlicensed_marked_excluded(self) -> None:
        reg = self.root / "docs" / "eval-licences.md"
        text = reg.read_text(encoding="utf-8")
        # Change transitions-dev (which has licence = "NONE") to use = "eval-only-cache"
        bad = text.replace(
            '| `transitions-dev` | transitions.dev skills | `skills` | `NONE` | `excluded` |',
            '| `transitions-dev` | transitions.dev skills | `skills` | `NONE` | `eval-only-cache` |',
        ).replace(
            'id = "transitions-dev"\nname = "transitions.dev skills"\nkind = "skills"\nurl = "https://transitions.dev"\npinned = ""\nlicence = "NONE"\nuse = "excluded"',
            'id = "transitions-dev"\nname = "transitions.dev skills"\nkind = "skills"\nurl = "https://transitions.dev"\npinned = ""\nlicence = "NONE"\nuse = "eval-only-cache"',
        )
        reg.write_text(bad, encoding="utf-8")
        code, findings = cel.check_repo(self.root)
        self.assertEqual(code, 1)
        self.assertTrue(any(f.startswith("none-means-excluded:") for f in findings), findings)

    def test_client_fixture_path_fails(self) -> None:
        s3 = self.root / "crates" / "designos-eval" / "suites" / "s3"
        s3.mkdir(parents=True, exist_ok=True)
        (s3 / "tasks.toml").write_text(
            '[[task]]\nid = "s3-1"\nsource = "shadcn-ui"\npath = "fixtures/VSF - PCP.fig"\n',
            encoding="utf-8",
        )
        code, findings = cel.check_repo(self.root)
        self.assertEqual(code, 1)
        self.assertTrue(any(f.startswith("client-file-ban:") for f in findings), findings)

    def test_home_path_fails(self) -> None:
        s3 = self.root / "crates" / "designos-eval" / "suites" / "s3"
        s3.mkdir(parents=True, exist_ok=True)
        (s3 / "tasks.toml").write_text(
            '[[task]]\nid = "s3-1"\nsource = "shadcn-ui"\npath = "/Users/x/Downloads/kit.fig"\n',
            encoding="utf-8",
        )
        code, findings = cel.check_repo(self.root)
        self.assertEqual(code, 1)
        self.assertTrue(any(f.startswith("client-file-ban:") for f in findings), findings)

        (s3 / "tasks.toml").write_text(
            '[[task]]\nid = "s3-1"\nsource = "shadcn-ui"\npath = "C:\\\\Users\\\\x\\\\Downloads\\\\gen-kit.fig"\n',
            encoding="utf-8",
        )
        code_win, findings_win = cel.check_repo(self.root)
        self.assertEqual(code_win, 1)
        self.assertTrue(any(f.startswith("client-file-ban:") for f in findings_win), findings_win)

    def test_word_containing_pcp_passes(self) -> None:
        s1 = self.root / "crates" / "designos-eval" / "suites" / "s1" / "specs"
        s1.mkdir(parents=True, exist_ok=True)
        (s1 / "pcp-free.md").write_text("# Clean spec\nPath: specs/pcp-free.md and gen-sample.fig\n", encoding="utf-8")
        code, findings = cel.check_repo(self.root)
        self.assertEqual(code, 0, findings)

    def test_third_party_file_stale(self) -> None:
        tp = self.root / "THIRD_PARTY_LICENSES.md"
        tp.write_text(tp.read_text(encoding="utf-8") + "\n<!-- stale edit -->\n", encoding="utf-8")
        code, findings = cel.check_repo(self.root)
        self.assertEqual(code, 1)
        self.assertTrue(any(f.startswith("third-party-current:") for f in findings), findings)

    def test_approved_without_review_date_fails(self) -> None:
        reg = self.root / "docs" / "eval-licences.md"
        text = reg.read_text(encoding="utf-8").replace('reviewed_on = "2026-10-19"', 'reviewed_on = ""', 1)
        reg.write_text(text, encoding="utf-8")
        code, findings = cel.check_repo(self.root)
        self.assertEqual(code, 1)
        self.assertTrue(any(f.startswith("schema:") for f in findings), findings)

    def test_table_mismatch_fails(self) -> None:
        reg = self.root / "docs" / "eval-licences.md"
        text = reg.read_text(encoding="utf-8").replace("| `ODC-By-1.0` |", "| `MIT` |", 1)
        reg.write_text(text, encoding="utf-8")
        code, findings = cel.check_repo(self.root)
        self.assertEqual(code, 1)
        self.assertTrue(any(f.startswith("table-matches-toml:") for f in findings), findings)


if __name__ == "__main__":
    unittest.main()
