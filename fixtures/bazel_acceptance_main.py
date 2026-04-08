"""Bazel entry: run manifest acceptance via ``is_palindrome_cli`` (see ``//fixtures:acceptance_manifest_cli``)."""

from __future__ import annotations

import sys
from pathlib import Path


def main() -> None:
    repo = Path(__file__).resolve().parent.parent
    if str(repo) not in sys.path:
        sys.path.insert(0, str(repo))
    from fixtures.cli.acceptance_cmd import run_acceptance_main

    sys.exit(run_acceptance_main(sys.argv[1:]))


if __name__ == "__main__":
    main()
