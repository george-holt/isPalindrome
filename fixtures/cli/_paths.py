"""Repository root and ``is_palindrome_cli`` resolution for ``fixtures.cli`` tooling."""

from __future__ import annotations

import os
from pathlib import Path

from python.runfiles import runfiles

# fixtures/cli/_paths.py -> fixtures -> repo root
REPO_ROOT = Path(__file__).resolve().parent.parent.parent

# Bzlmod main-repo runfiles prefix for targets under the workspace root (see runfiles manifest).
_MAIN_CLI = "_main/CLI/is_palindrome_cli"


def resolve_is_palindrome_cli(repo_root: Path | None = None) -> Path | None:
    """Path to the Rust ``is_palindrome_cli`` binary.

    Resolution order:

    1. ``IS_PALINDROME_CLI`` (absolute path to the executable).
    2. ``python.runfiles`` lookup for ``_main/CLI/is_palindrome_cli`` when run under Bazel
       (e.g. ``bazel run //fixtures:cli``). No ``rglob`` or workspace-relative fallbacks.

    Outside Bazel runfiles (no manifest / ``RUNFILES_DIR``), returns ``None`` unless
    ``IS_PALINDROME_CLI`` is set — use ``bazel run //fixtures:cli`` or set the env var.
    """
    _ = repo_root  # reserved for API compatibility; REPO_ROOT used implicitly via runfiles only
    env = os.environ.get("IS_PALINDROME_CLI")
    if env:
        p = Path(env).expanduser()
        if p.is_file():
            return p.resolve()

    rf = runfiles.Create()
    if rf is None:
        return None
    loc = rf.Rlocation(_MAIN_CLI)
    if not loc:
        return None
    path = Path(loc)
    if path.is_file() and os.access(path, os.X_OK):
        return path.resolve()
    return None
