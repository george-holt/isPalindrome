"""Invoke thin stdin-JSON adapters for non-Python backends."""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
from pathlib import Path
from typing import TextIO

from fixtures.cli._paths import REPO_ROOT
from fixtures.cli.parse import ParsedCheck, resolve_text

_IMPL_CHOICES = frozenset({"py", "cpp", "c", "rust", "cs", "nodejs"})


def parsed_to_payload(
    parsed: ParsedCheck,
    stdin: TextIO,
) -> dict:
    """Build JSON request for thin CLIs."""
    show_help, hex_mode, hex_payload, use_stdin, positional, custom_bytes = parsed
    if show_help:
        raise AssertionError
    custom = sorted(custom_bytes)
    if hex_mode:
        assert hex_payload is not None
        return {"mode": "hex", "hex": hex_payload, "custom": custom}
    text = resolve_text(stdin, use_stdin, hex_mode, positional)
    return {"mode": "string", "text": text, "custom": custom}


def run_thin(
    impl: str,
    parsed: ParsedCheck,
    stdin: TextIO,
    stdout: TextIO,
    stderr: TextIO,
) -> int:
    """Run a thin backend: JSON on stdin; stdout is true/false line; stderr on error."""
    payload = parsed_to_payload(parsed, stdin)
    raw = json.dumps(payload, ensure_ascii=False)

    if impl == "rust":
        rust_dir = REPO_ROOT / "rust" / "is_palindrome"
        cargo = shutil.which("cargo")
        if not cargo:
            stderr.write("cargo not found on PATH (required for --impl rust)\n")
            return 127
        p = subprocess.run(
            [cargo, "run", "--quiet", "--bin", "pal_stdin"],
            cwd=rust_dir,
            input=raw,
            text=True,
            capture_output=True,
            timeout=120,
        )
        stdout.write(p.stdout)
        stderr.write(p.stderr)
        return p.returncode

    if impl == "nodejs":
        script = REPO_ROOT / "nodejs" / "is-palindrome" / "pal-cli.mjs"
        node = shutil.which("node")
        if not node:
            stderr.write("node not found on PATH (required for --impl nodejs)\n")
            return 127
        p = subprocess.run(
            [node, str(script)],
            cwd=REPO_ROOT / "nodejs" / "is-palindrome",
            input=raw,
            text=True,
            capture_output=True,
            timeout=60,
        )
        stdout.write(p.stdout)
        stderr.write(p.stderr)
        return p.returncode

    if impl == "cs":
        proj = REPO_ROOT / "cs" / "PalCli" / "PalCli.csproj"
        if not proj.is_file():
            stderr.write(f"missing PalCli project: {proj}\n")
            return 127
        dotnet = shutil.which("dotnet")
        if not dotnet:
            stderr.write("dotnet not found on PATH (required for --impl cs)\n")
            return 127
        p = subprocess.run(
            [dotnet, "run", "--project", str(proj)],
            cwd=REPO_ROOT / "cs",
            input=raw,
            text=True,
            capture_output=True,
            timeout=180,
        )
        stdout.write(p.stdout)
        stderr.write(p.stderr)
        return p.returncode

    if impl == "cpp":
        exe = _find_cpp_pal_stdin()
        if not exe:
            stderr.write(
                "pal_stdin not found: build cpp (cmake) so cpp/build or cpp/out contains pal_stdin\n"
            )
            return 127
        p = subprocess.run(
            [str(exe)],
            input=raw,
            text=True,
            capture_output=True,
            timeout=60,
        )
        stdout.write(p.stdout)
        stderr.write(p.stderr)
        return p.returncode

    if impl == "c":
        exe = _find_c_pal_stdin()
        if not exe:
            stderr.write(
                "pal_stdin not found: build c (cmake) so c/build contains pal_stdin\n"
            )
            return 127
        p = subprocess.run(
            [str(exe)],
            input=raw,
            text=True,
            capture_output=True,
            timeout=60,
        )
        stdout.write(p.stdout)
        stderr.write(p.stderr)
        return p.returncode

    stderr.write(f"unknown impl for thin runner: {impl}\n")
    return 2


def _find_cpp_pal_stdin() -> Path | None:
    root = REPO_ROOT / "cpp"
    candidates = [
        root / "build" / "pal_stdin.exe",
        root / "build" / "pal_stdin",
        root / "build" / "Release" / "pal_stdin.exe",
        root / "build" / "Debug" / "pal_stdin.exe",
        root / "out" / "build" / "pal_stdin.exe",
    ]
    for c in candidates:
        if c.is_file():
            return c
    return None


def _find_c_pal_stdin() -> Path | None:
    root = REPO_ROOT / "c" / "build"
    candidates = [
        root / "pal_stdin.exe",
        root / "pal_stdin",
        root / "Release" / "pal_stdin.exe",
        root / "Debug" / "pal_stdin.exe",
    ]
    for c in candidates:
        if c.is_file():
            return c
    return None


def validate_impl(name: str) -> bool:
    return name in _IMPL_CHOICES
