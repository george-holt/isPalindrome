"""Regression: CMake build trees must not be reused across Windows vs POSIX hosts."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path
from unittest.mock import patch

_PY = Path(__file__).resolve().parent.parent
if str(_PY) not in sys.path:
    sys.path.insert(0, str(_PY))

from is_palindrome.cmake_cache_compat import cache_text_mismatch_reason, native_build_mismatch_message


class TestCacheTextMismatch(unittest.TestCase):
    def test_visual_studio_generator_rejected_on_linux(self) -> None:
        text = "CMAKE_GENERATOR:INTERNAL=Visual Studio 17 2022\n"
        with patch("platform.system", return_value="Linux"):
            self.assertIsNotNone(cache_text_mismatch_reason(text))

    def test_msvc_compiler_path_rejected_on_linux(self) -> None:
        text = (
            "CMAKE_GENERATOR:INTERNAL=Ninja\n"
            'CMAKE_CXX_COMPILER:FILEPATH=C:/Program Files/Microsoft Visual Studio/2022/Enterprise/VC/Tools/MSVC/14.40.33807/bin/Hostx64/x64/cl.exe\n'
        )
        with patch("platform.system", return_value="Linux"):
            self.assertIsNotNone(cache_text_mismatch_reason(text))

    def test_unix_build_ok_on_linux(self) -> None:
        text = (
            "# For build in directory: /workspaces/foo/cpp/build\n"
            "CMAKE_GENERATOR:INTERNAL=Ninja\n"
            "CMAKE_CXX_COMPILER:FILEPATH=/usr/bin/c++\n"
        )
        with patch("platform.system", return_value="Linux"):
            self.assertIsNone(cache_text_mismatch_reason(text))

    def test_build_dir_comment_windows_path_rejected_on_linux(self) -> None:
        text = "# For build in directory: c:/Users/x/cpp/build\nCMAKE_GENERATOR:INTERNAL=Ninja\n"
        with patch("platform.system", return_value="Linux"):
            self.assertIsNotNone(cache_text_mismatch_reason(text))


class TestNativeBuildMismatchMessage(unittest.TestCase):
    def test_missing_dir(self) -> None:
        p = Path("/nonexistent/is_palindrome_build_xyz")
        msg = native_build_mismatch_message(p)
        self.assertIsNotNone(msg)
        self.assertIn("Missing build directory", msg or "")


if __name__ == "__main__":
    unittest.main()
