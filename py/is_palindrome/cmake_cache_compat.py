"""Detect CMake build trees generated on another OS so native tests fail with a clear fix."""

from __future__ import annotations

import re
from pathlib import Path

_STALE_HINT = (
    "This CMake build tree was generated on a different OS or toolchain. "
    "Remove this build directory and reconfigure on this machine. "
    "Example: rm -rf cpp/build && cmake -S cpp -B cpp/build && cmake --build cpp/build"
)


def native_build_mismatch_message(build_dir: Path) -> str | None:
    """Return a human-readable error if ``build_dir`` is missing or not usable on this host."""
    if not build_dir.is_dir():
        return (
            f"Missing build directory {build_dir}. Configure and build from the repo root "
            f"(see CONTRIBUTING.md), e.g. cmake -S <src> -B {build_dir} && cmake --build {build_dir}"
        )
    cache = build_dir / "CMakeCache.txt"
    if not cache.is_file():
        return (
            f"No CMakeCache.txt in {build_dir}. Run cmake to configure this build directory "
            f"on this machine (see CONTRIBUTING.md)."
        )
    text = cache.read_text(encoding="utf-8", errors="replace")
    return cache_text_mismatch_reason(text)


def cache_text_mismatch_reason(text: str) -> str | None:
    """Return an error string if *text* looks like a CMakeCache from another platform."""
    import platform

    sysname = platform.system()

    m = re.search(r"^CMAKE_GENERATOR:INTERNAL=(.+)$", text, re.MULTILINE)
    if m:
        gen = m.group(1).strip()
        if sysname != "Windows" and ("Visual Studio" in gen or "NMake" in gen):
            return _STALE_HINT
        if sysname == "Windows" and gen == "Unix Makefiles":
            # Possible WSL/Linux-only tree; compiler path check below refines.
            pass

    m = re.search(r"^CMAKE_CXX_COMPILER:(?:FILEPATH|STRING)=(.+)$", text, re.MULTILINE)
    if m:
        comp = m.group(1).strip()
        if sysname != "Windows":
            if _looks_like_windows_compiler_path(comp):
                return _STALE_HINT
        else:
            if comp.startswith("/usr/") or comp.startswith("/home/") or comp.startswith("/opt/"):
                return _STALE_HINT

    m = re.search(r"^# For build in directory:\s*(.+)$", text, re.MULTILINE)
    if m:
        bd = m.group(1).strip()
        if sysname != "Windows" and re.match(r"^[cC]:[\\/]", bd):
            return _STALE_HINT
        if sysname == "Windows" and bd.startswith(("/", "\\")) and not bd.startswith("//"):
            if bd.startswith("/home/") or bd.startswith("/usr/"):
                return _STALE_HINT

    return None


def _looks_like_windows_compiler_path(comp: str) -> bool:
    if re.match(r"^[A-Za-z]:[\\/]", comp):
        return True
    if "Microsoft Visual Studio" in comp or "\\\\" in comp.replace("/", ""):
        return True
    return False
