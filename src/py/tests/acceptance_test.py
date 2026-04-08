"""Direct manifest-driven tests: shared JSON cases executed in-process against ``is_palindrome``."""

from __future__ import annotations

import os
import sys
import unittest
from pathlib import Path

# Canonical coverage for the Python algorithm comes from this ``py_test``, not subprocess CLI runs.
from fixtures.manifest_cases import (
    iter_cases_for_lang,
    load_manifest,
    parse_custom_bytes,
    unicode_scalar_to_str,
    uses_string_api,
)
from is_palindrome.palindrome import PalindromeException, is_palindrome, is_palindrome_from_utf8


def _manifest_path() -> Path:
    """Resolve ``fixtures/acceptance_manifest.json`` under Bazel runfiles or repo root."""
    for key in ("RUNFILES_DIR", "TEST_SRCDIR"):
        root = os.environ.get(key)
        if not root:
            continue
        ws = os.environ.get("TEST_WORKSPACE", "_main")
        for candidate in (
            Path(root) / ws / "fixtures" / "acceptance_manifest.json",
            Path(root) / "fixtures" / "acceptance_manifest.json",
        ):
            if candidate.is_file():
                return candidate
    here = Path(__file__).resolve().parents[3]
    return here / "fixtures" / "acceptance_manifest.json"


class AcceptanceManifestTest(unittest.TestCase):
    def test_all_cases(self) -> None:
        manifest = load_manifest(_manifest_path())
        for case in iter_cases_for_lang(manifest, "py"):
            with self.subTest(case_id=case["id"]):
                self._run_case(case)

    def _run_case(self, case: dict) -> None:
        custom = parse_custom_bytes(case)
        exp = case["expected"]
        kind = exp["kind"]

        if uses_string_api(case):
            if "input_unicode_scalar" in case:
                s = unicode_scalar_to_str(case["input_unicode_scalar"])
            else:
                s = case["input_ascii"]
            try:
                got = is_palindrome_from_utf8(s, custom)
            except PalindromeException as e:
                if kind != "error":
                    self.fail(f"unexpected PalindromeException: {e.error_code}")
                self.assertEqual(exp["code"], e.error_code)
                return
            if kind == "error":
                self.fail(f"expected error {exp['code']}, got result {got!r}")
            self.assertEqual(exp["value"], got)
            return

        if "input_ascii" in case:
            data = case["input_ascii"].encode("latin-1")
        elif "input_hex" in case:
            data = bytes.fromhex(case["input_hex"])
        else:
            self.fail("case has no input_ascii, input_hex, or string-api fields")

        got = is_palindrome(data, custom)
        self.assertEqual("boolean", kind)
        self.assertEqual(exp["value"], got)


if __name__ == "__main__":
    unittest.main(module=__name__, argv=[sys.argv[0], "-v"], exit=True)
