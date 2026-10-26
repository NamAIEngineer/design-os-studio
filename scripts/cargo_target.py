#!/usr/bin/env python3
"""Resolve the Cargo target directory for this checkout."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path


def resolve(repo_root: Path) -> Path:
    configured = os.environ.get("CARGO_TARGET_DIR")
    if configured:
        return Path(configured).expanduser().resolve()

    try:
        result = subprocess.run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1"],
            cwd=repo_root,
            check=True,
            capture_output=True,
            text=True,
        )
        return Path(json.loads(result.stdout)["target_directory"]).resolve()
    except (FileNotFoundError, OSError, subprocess.CalledProcessError, KeyError, json.JSONDecodeError):
        return (repo_root.parent / ".design-os-studio-target").resolve()


if __name__ == "__main__":
    root = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else Path.cwd()
    print(resolve(root))
