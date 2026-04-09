"""Regression: ``resolve_is_palindrome_cli`` uses rules_python runfiles (``bazel test`` only)."""

from __future__ import annotations

import os
import unittest
from pathlib import Path

from fixtures.cli._paths import resolve_is_palindrome_cli


class CliPathsTest(unittest.TestCase):
    def test_resolve_is_palindrome_cli_from_runfiles(self) -> None:
        self.assertTrue(
            os.environ.get("RUNFILES_MANIFEST_FILE") or os.environ.get("RUNFILES_DIR"),
            "expected Bazel test runfiles environment",
        )
        p = resolve_is_palindrome_cli()
        self.assertIsNotNone(p, "expected //CLI:is_palindrome_cli as data dep")
        assert p is not None
        self.assertTrue(Path(p).is_file())
        self.assertTrue(os.access(p, os.X_OK))


if __name__ == "__main__":
    # ``unittest.main()`` calls ``sys.exit()`` internally; that ``SystemExit`` confuses
    # rules_python's coverage wrapper. Use ``exit=False``, and do not ``sys.exit(0)`` on
    # success — that still raises ``SystemExit`` and breaks LCOV. Exit 1 only on failure.
    import sys

    prog = unittest.main(exit=False)
    if not prog.result.wasSuccessful():
        sys.exit(1)
