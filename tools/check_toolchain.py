#!/usr/bin/env python3
"""Verify CLI tools on PATH that tools/run_all_tests.py and thin CLIs need."""

from __future__ import annotations

import shutil
import subprocess
import sys


def _which_python() -> tuple[str, str] | None:
    for name in ("python3", "python"):
        path = shutil.which(name)
        if path:
            return name, path
    return None


def main() -> int:
    rows: list[tuple[str, str, str]] = []

    py = _which_python()
    if py:
        name, path = py
        try:
            ver = subprocess.run(
                [name, "-c", "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')"],
                capture_output=True,
                text=True,
                timeout=10,
                check=True,
            ).stdout.strip()
            rows.append((name, f"ok ({ver})", path))
        except (OSError, subprocess.SubprocessError):
            rows.append((name, "ok", path))
    else:
        rows.append(("python", "MISSING", ""))

    for cmd in ("cargo", "rustc", "node", "npm", "dotnet", "cmake", "ctest"):
        path = shutil.which(cmd)
        rows.append((cmd, "ok" if path else "MISSING", path or ""))

    w = max(len(r[0]) for r in rows)
    bad = False
    for cmd, status, path in rows:
        print(f"{cmd.ljust(w)}  {status:12}  {path}")
        if "MISSING" in status:
            bad = True

    if bad:
        print(
            "\nInstall missing tools (see CONTRIBUTING.md) or open the repo in the Dev Container."
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
