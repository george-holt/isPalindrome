"""`check` subcommand: `--impl` dispatcher."""

from __future__ import annotations

import sys
from typing import TextIO

from fixtures.cli.engine import execute_check_py
from fixtures.cli.parse import parse_check_argv
from fixtures.cli.thin import run_thin, validate_impl

CHECK_HELP = """Usage:
  python -m fixtures.cli check [--impl BACKEND] [options] [TEXT...]

Backends (--impl): py, cpp, c, rust, cs, nodejs  (default: py)

Options:
  --help, -h     Show this help.
  --stdin        Read UTF-8 text from stdin (string API).
  --hex HEX      Byte mode: lowercase hex pairs (e.g. 61ff62).
  --custom XX    Repeatable: extra delimiter byte (hex pair), for string or --hex input.
  --             End of options; remaining words are TEXT.

If no TEXT is given and --hex is not used, input is read from stdin (or empty).

Exit: 0 = true, 1 = false, 2 = error (e.g. NON_ASCII_STRING_INPUT).
Stdout: one line, "true" or "false".

"""


def run_check(
    argv: list[str],
    stdin: TextIO | None = None,
    stdout: TextIO | None = None,
    stderr: TextIO | None = None,
) -> int:
    stdin = stdin if stdin is not None else sys.stdin
    stdout = stdout if stdout is not None else sys.stdout
    stderr = stderr if stderr is not None else sys.stderr

    impl = "py"
    rest = argv
    if len(rest) >= 1 and rest[0] == "--impl":
        if len(rest) < 2:
            stderr.write("--impl requires a backend name\n")
            return 2
        impl = rest[1]
        rest = rest[2:]

    if not validate_impl(impl):
        stderr.write(
            f"unknown --impl {impl!r}; expected one of: py, cpp, c, rust, cs, nodejs\n"
        )
        return 2

    try:
        parsed = parse_check_argv(rest)
    except (ValueError, OSError) as ex:
        stderr.write(str(ex) + "\n")
        return 2

    if parsed[0]:
        stdout.write(CHECK_HELP)
        return 0

    if impl == "py":
        return execute_check_py(parsed, stdin, stdout, stderr)
    return run_thin(impl, parsed, stdin, stdout, stderr)
