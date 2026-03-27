"""tools/run_all_tests argv selection (default vs argv_verbose)."""

from __future__ import annotations

import importlib.util
import json
import sys
import unittest
from pathlib import Path

_REPO = Path(__file__).resolve().parent.parent.parent


def _load_run_all_tests():
    path = _REPO / "tools" / "run_all_tests.py"
    spec = importlib.util.spec_from_file_location("_run_all_tests_mod", path)
    mod = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(mod)
    return mod


class TestSelectArgv(unittest.TestCase):
    def test_verbose_uses_argv_verbose_when_present(self) -> None:
        mod = _load_run_all_tests()
        suite = {"argv": ["a", "b"], "argv_verbose": ["x", "y", "z"]}
        self.assertEqual(mod._select_argv(suite, False), ["a", "b"])
        self.assertEqual(mod._select_argv(suite, True), ["x", "y", "z"])

    def test_verbose_falls_back_when_no_argv_verbose(self) -> None:
        mod = _load_run_all_tests()
        suite = {"argv": ["only"]}
        self.assertEqual(mod._select_argv(suite, True), ["only"])


class TestTestSuitesJson(unittest.TestCase):
    def test_each_suite_defines_argv_verbose(self) -> None:
        p = _REPO / "tools" / "test_suites.json"
        data = json.loads(p.read_text(encoding="utf-8"))
        for s in data["suites"]:
            with self.subTest(suite=s["id"]):
                self.assertIn("argv_verbose", s)
                self.assertIsInstance(s["argv_verbose"], list)
                self.assertGreater(len(s["argv_verbose"]), 0)


if __name__ == "__main__":
    unittest.main()
