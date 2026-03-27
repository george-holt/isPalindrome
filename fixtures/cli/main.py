"""`python -m fixtures.cli` entry: subcommands check | acceptance | test-native."""

from __future__ import annotations

import sys

from fixtures.cli.acceptance_cmd import run_acceptance_main
from fixtures.cli.check import run_check
from fixtures.cli.native_cmd import run_test_native

ROOT_HELP = """Usage:
  python -m fixtures.cli <command> ...

Commands:
  check          Palindrome check (--impl selects backend; default py)
  acceptance     Run fixtures/acceptance_manifest.json cases via `check`
  test-native    Run native library test matrix (CMake, cargo, dotnet, npm, unittest)

Examples:
  python -m fixtures.cli check --impl py aba
  python -m fixtures.cli check --impl rust --hex 61ff62
  python -m fixtures.cli acceptance --impl py
  python -m fixtures.cli test-native

See also: python -m fixtures.cli check --help
"""


def main(argv: list[str] | None = None) -> int:
    argv = argv if argv is not None else sys.argv[1:]
    if not argv or argv[0] in ("-h", "--help"):
        sys.stdout.write(ROOT_HELP)
        return 0
    cmd = argv[0]
    if cmd == "check":
        return run_check(argv[1:])
    if cmd == "acceptance":
        return run_acceptance_main(argv[1:])
    if cmd == "test-native":
        return run_test_native(argv[1:])
    sys.stderr.write(f"unknown command {cmd!r}; try: python -m fixtures.cli -h\n")
    return 2
