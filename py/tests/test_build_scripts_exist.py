"""Regression: documented full/clean build entrypoints exist."""

from __future__ import annotations

import stat
import sys
import unittest
from pathlib import Path

_REPO = Path(__file__).resolve().parent.parent.parent


class TestBuildScripts(unittest.TestCase):
    def test_build_all_and_clean_scripts_exist(self) -> None:
        for name in ("build_all.sh", "clean_build_all.sh"):
            p = _REPO / "tools" / name
            self.assertTrue(p.is_file(), msg=f"missing {p}")
            mode = p.stat().st_mode
            self.assertTrue(mode & stat.S_IXUSR, msg=f"not executable: {p}")
            self.assertTrue(p.read_text(encoding="utf-8").startswith("#!/usr/bin/env bash\n"))


if __name__ == "__main__":
    unittest.main()
