"""Repository root resolution for ``fixtures.cli`` tooling (acceptance runner, etc.)."""

from __future__ import annotations

import os
import shutil
import stat
from pathlib import Path

# fixtures/cli/_paths.py -> fixtures -> repo root
REPO_ROOT = Path(__file__).resolve().parent.parent.parent


def resolve_is_palindrome_cli(repo_root: Path | None = None) -> Path | None:
    """Path to the Rust ``is_palindrome_cli`` binary for acceptance / tooling.

    Resolution order:

    1. ``IS_PALINDROME_CLI`` (absolute path to the executable)
    2. Under ``RUNFILES_DIR`` / ``TEST_SRCDIR`` (Bazel runfiles layout)
    3. ``<repo>/bazel-bin/CLI/is_palindrome_cli``
    4. ``<repo>/CLI/target/release/is_palindrome_cli``
    5. ``<repo>/CLI/target/debug/is_palindrome_cli``
    6. ``is_palindrome_cli`` on ``PATH``
    """
    root = repo_root or REPO_ROOT
    env = os.environ.get("IS_PALINDROME_CLI")
    if env:
        p = Path(env).expanduser()
        if p.is_file():
            return p.resolve()

    for runfiles_root in (os.environ.get("RUNFILES_DIR"), os.environ.get("TEST_SRCDIR")):
        if not runfiles_root:
            continue
        p = Path(runfiles_root)
        if not p.is_dir():
            continue
        for f in p.rglob("is_palindrome_cli"):
            if not f.is_file():
                continue
            try:
                if f.stat().st_mode & (stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH):
                    return f.resolve()
            except OSError:
                continue
            return f.resolve()

    for rel in (
        "bazel-bin/CLI/is_palindrome_cli",
        "CLI/target/release/is_palindrome_cli",
        "CLI/target/debug/is_palindrome_cli",
    ):
        p = root / rel
        if p.is_file():
            return p.resolve()
    which = shutil.which("is_palindrome_cli")
    if which:
        return Path(which).resolve()
    return None
