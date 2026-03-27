"""run_all_tests log banners include cwd and resolved command."""

from __future__ import annotations

import importlib.util
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


class TestSuiteBanner(unittest.TestCase):
    def test_banner_includes_suite_cwd_command(self) -> None:
        mod = _load_run_all_tests()
        banner = mod._suite_banner(
            "cpp_acceptance",
            "CTest for cpp (Catch2 acceptance_tests)",
            Path("/workspaces/isPalindrome/cpp/build"),
            ["/usr/bin/ctest", "--output-on-failure", "-V"],
        )
        self.assertIn("suite=cpp_acceptance", banner)
        self.assertIn("description=CTest for cpp", banner)
        self.assertIn("cwd=/workspaces/isPalindrome/cpp/build", banner)
        self.assertIn("command=", banner)
        self.assertIn("ctest", banner)


if __name__ == "__main__":
    unittest.main()
