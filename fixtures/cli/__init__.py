"""Tooling package: acceptance runner and native matrix (``python -m fixtures.cli``).

The **palindrome** CLI (argv parsing, ``--impl``, exit codes) is **Rust**:
``bazel run //CLI:is_palindrome_cli``.
"""

from fixtures.cli.acceptance_cmd import run_acceptance_main

__all__ = ["run_acceptance_main"]
