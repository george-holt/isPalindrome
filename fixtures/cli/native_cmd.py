"""Delegate to `tools/run_all_tests.py` for the native test matrix."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

from fixtures.cli._paths import REPO_ROOT


def run_test_native(_argv: list[str]) -> int:
    script = REPO_ROOT / "tools" / "run_all_tests.py"
    if not script.is_file():
        sys.stderr.write(f"missing {script}\n")
        return 2
    r = subprocess.run([sys.executable, str(script)], cwd=str(REPO_ROOT))
    return r.returncode
