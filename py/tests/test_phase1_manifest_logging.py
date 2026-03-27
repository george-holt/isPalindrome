"""Phase 1: manifest-driven harnesses emit one progress line per manifest case (stderr)."""

from __future__ import annotations

import unittest
from pathlib import Path

_REPO = Path(__file__).resolve().parent.parent.parent


class TestManifestLogging(unittest.TestCase):
    def test_c_acceptance_logs_each_case_to_stderr(self) -> None:
        text = (_REPO / "c" / "tests" / "acceptance_manifest.c").read_text(encoding="utf-8")
        # Progress line before assertions (not only error paths).
        self.assertIn('fprintf(stderr, "case %s\\n", id->valuestring);', text)

    def test_cpp_acceptance_logs_each_case_to_stderr(self) -> None:
        text = (_REPO / "cpp" / "tests" / "acceptance_tests.cpp").read_text(encoding="utf-8")
        self.assertIn("std::cerr", text)
        self.assertIn('"case "', text)

    def test_cli_acceptance_logs_each_case(self) -> None:
        text = (_REPO / "fixtures" / "cli" / "acceptance_cmd.py").read_text(encoding="utf-8")
        self.assertIn('sys.stderr.write(f"case {cid}\\n")', text)
        self.assertIn("stderr", text)


if __name__ == "__main__":
    unittest.main()
