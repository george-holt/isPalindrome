"""Run the full Bazel test matrix (`bazel test //...`)."""

from __future__ import annotations

import shutil
import subprocess
import sys

from fixtures.cli._paths import REPO_ROOT


def run_test_native(_argv: list[str]) -> int:
    for cmd in ("bazelisk", "bazel"):
        path = shutil.which(cmd)
        if path:
            return subprocess.run([path, "test", "//..."], cwd=str(REPO_ROOT)).returncode
    sys.stderr.write("bazel or bazelisk not on PATH\n")
    return 2
