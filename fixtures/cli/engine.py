"""In-process check using the Python sample (`py/`)."""

from __future__ import annotations

import sys
from pathlib import Path
from typing import TextIO

_PY = Path(__file__).resolve().parent.parent.parent / "py"
if str(_PY) not in sys.path:
    sys.path.insert(0, str(_PY))

from is_palindrome.palindrome import PalindromeException, from_bytes, from_string

from fixtures.cli.parse import ParsedCheck, decode_hex, resolve_text


def execute_check_py(
    parsed: ParsedCheck,
    stdin: TextIO,
    stdout: TextIO,
    stderr: TextIO,
) -> int:
    show_help, hex_mode, hex_payload, use_stdin, positional, custom_bytes = parsed
    if show_help:
        raise AssertionError("help handled by caller")
    try:
        custom_opt = custom_bytes if custom_bytes else None
        if hex_mode:
            assert hex_payload is not None
            data = decode_hex(hex_payload)
            result = from_bytes(data, custom_opt)
            stdout.write("true\n" if result else "false\n")
            return 0 if result else 1
        text = resolve_text(stdin, use_stdin, hex_mode, positional)
        result = from_string(text, custom_opt)
        stdout.write("true\n" if result else "false\n")
        return 0 if result else 1
    except PalindromeException as ex:
        stderr.write(ex.error_code + "\n")
        stderr.write(str(ex) + "\n")
        return 2
    except (ValueError, OSError) as ex:
        stderr.write(str(ex) + "\n")
        return 2
