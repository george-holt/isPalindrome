"""Smoke tests for fixtures.cli."""

from __future__ import annotations

import io
import unittest

from fixtures.cli.check import run_check


class CheckPyTests(unittest.TestCase):
    def test_aba_true(self) -> None:
        out = io.StringIO()
        err = io.StringIO()
        rc = run_check(["aba"], io.StringIO(), out, err)
        self.assertEqual(rc, 0)
        self.assertEqual(out.getvalue(), "true\n")

    def test_ab_false(self) -> None:
        out = io.StringIO()
        err = io.StringIO()
        rc = run_check(["ab"], io.StringIO(), out, err)
        self.assertEqual(rc, 1)
        self.assertEqual(out.getvalue(), "false\n")


if __name__ == "__main__":
    unittest.main()
