"""Regression: c/ tests include <cJSON.h>; CMake must expose cjson's source dir to consumers."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

_REPO = Path(__file__).resolve().parent.parent.parent


class TestCCmakeIncludesCjson(unittest.TestCase):
    def test_acceptance_test_has_cjson_include_dir(self) -> None:
        text = (_REPO / "c" / "CMakeLists.txt").read_text(encoding="utf-8")
        self.assertIn("${cjson_SOURCE_DIR}", text)
        self.assertIn("target_include_directories(acceptance_test", text)
        self.assertIn("target_include_directories(pal_stdin", text)


if __name__ == "__main__":
    unittest.main()
