"""Stdin JSON adapter for ``is_palindrome_cli --impl py`` (library only; not fixtures.cli)."""

from __future__ import annotations

import json
import sys
from typing import Any

from is_palindrome.palindrome import (
    PalindromeException,
    is_palindrome,
    is_palindrome_from_utf8,
)


def main() -> None:
    raw = sys.stdin.read()
    try:
        req: dict[str, Any] = json.loads(raw.strip())
    except json.JSONDecodeError as e:
        print(e, file=sys.stderr)
        sys.exit(2)

    mode = req.get("mode", "")
    custom_raw = req.get("custom", [])
    custom: set[int] | None = None
    if custom_raw:
        custom = set(custom_raw)

    if mode == "hex":
        hex_s = req.get("hex")
        if hex_s is None:
            print("missing hex", file=sys.stderr)
            sys.exit(2)
        data = bytes(int(hex_s[i : i + 2], 16) for i in range(0, len(hex_s), 2))
        r = is_palindrome(data, custom)
        print("true" if r else "false")
        sys.exit(0 if r else 1)
    if mode == "string":
        text = req.get("text")
        if text is None:
            print("missing text", file=sys.stderr)
            sys.exit(2)
        try:
            r = is_palindrome_from_utf8(text, custom)
        except PalindromeException as e:
            print(e.error_code, file=sys.stderr)
            print("Input contains a scalar value > U+007F.", file=sys.stderr)
            sys.exit(2)
        print("true" if r else "false")
        sys.exit(0 if r else 1)

    print("unknown mode", file=sys.stderr)
    sys.exit(2)


if __name__ == "__main__":
    main()
