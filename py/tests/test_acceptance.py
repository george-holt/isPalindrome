"""Loads fixtures/acceptance_manifest.json (SPEC §4)."""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

_PY = Path(__file__).resolve().parent.parent
if str(_PY) not in sys.path:
    sys.path.insert(0, str(_PY))

from is_palindrome.palindrome import PalindromeException, from_bytes, from_string

_FIXTURES = Path(__file__).resolve().parent.parent.parent / "fixtures"


def _unicode_scalar_to_string(manifest_scalar: str) -> str:
    prefix = "U+"
    if not manifest_scalar.upper().startswith(prefix):
        raise ValueError(manifest_scalar)
    cp = int(manifest_scalar[len(prefix) :], 16)
    return chr(cp)


def _decode_hex(hex_str: str) -> bytes:
    return bytes(int(hex_str[i : i + 2], 16) for i in range(0, len(hex_str), 2))


def _parse_custom_delimiters(case: dict) -> set[int] | None:
    opts = case.get("options") or {}
    if opts.get("invalid_mode") != "custom":
        return None
    hex_arr = opts.get("invalid_bytes_hex")
    if not hex_arr:
        return None
    s = {int(h, 16) for h in hex_arr}
    return s if s else None


def _applies_to_python(case: dict) -> bool:
    applies = case.get("applies_to")
    if not applies:
        return True
    return "py" in applies or "python" in applies


class AcceptanceManifestTests(unittest.TestCase):
    def test_all_cases(self) -> None:
        path = _FIXTURES / "acceptance_manifest.json"
        self.assertTrue(path.is_file(), msg=str(path))
        root = json.loads(path.read_text(encoding="utf-8"))
        for case in root["cases"]:
            cid = case["id"]
            if cid == "pal-stream-note-001":
                continue
            if not _applies_to_python(case):
                continue
            custom = _parse_custom_delimiters(case)
            exp = case["expected"]
            kind = exp["kind"]
            if kind == "boolean":
                want = exp["value"]
                inp = self._build_input(case)
                if inp[0] == "bytes":
                    got = from_bytes(inp[1], custom)
                else:
                    got = from_string(inp[1], custom)
                self.assertEqual(want, got, msg=cid)
            elif kind == "error":
                code = exp["code"]
                inp = self._build_input(case, string_api=True)
                self.assertEqual(inp[0], "string")
                with self.assertRaises(PalindromeException) as ctx:
                    from_string(inp[1], custom)
                self.assertEqual(code, ctx.exception.error_code, msg=cid)
            else:
                self.fail(f"unknown kind {kind} in {cid}")

    def _build_input(
        self, case: dict, *, string_api: bool | None = None
    ) -> tuple[str, bytes] | tuple[str, str]:
        if string_api is None:
            string_api = bool(case.get("applies_to"))
        if "input_ascii" in case:
            s = case["input_ascii"]
            if string_api:
                return ("string", s)
            return ("bytes", s.encode("latin-1"))
        if "input_hex" in case:
            return ("bytes", _decode_hex(case["input_hex"]))
        if "input_unicode_scalar" in case:
            return ("string", _unicode_scalar_to_string(case["input_unicode_scalar"]))
        raise ValueError("no input field")


if __name__ == "__main__":
    unittest.main()
