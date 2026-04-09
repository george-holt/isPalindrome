"""``bazel run //fixtures:cli`` entry: acceptance runner | test-native (tooling only).

The user-facing palindrome CLI is the Rust binary ``is_palindrome_cli``; this driver uses Bazel
runfiles to find it (``data`` on ``//fixtures:cli``). For direct CLI use: ``bazel run //CLI:is_palindrome_cli``.
"""

from __future__ import annotations

import sys

from fixtures.cli.acceptance_cmd import run_acceptance_main
from fixtures.cli.native_cmd import run_test_native

ROOT_HELP = """Usage:
  bazel run //fixtures:cli -- <command> ...

Commands:
  acceptance     Run fixtures/acceptance_manifest.json via is_palindrome_cli (per case)
  test-native    Run `bazel test //...` (full native matrix)

Examples:
  bazel run //fixtures:cli -- acceptance --impl rust
  bazel run //CLI:is_palindrome_cli -- aba
  bazel test //fixtures:acceptance_manifest_cli

See SPEC.md for the canonical CLI contract (Rust ``is_palindrome_cli``).
"""


def main(argv: list[str] | None = None) -> int:
    argv = argv if argv is not None else sys.argv[1:]
    if not argv or argv[0] in ("-h", "--help"):
        sys.stdout.write(ROOT_HELP)
        return 0
    cmd = argv[0]
    if cmd == "acceptance":
        return run_acceptance_main(argv[1:])
    if cmd == "test-native":
        return run_test_native(argv[1:])
    sys.stderr.write(f"unknown command {cmd!r}; try: bazel run //fixtures:cli -- -h\n")
    return 2
