"""Run shared acceptance cases through `fixtures.cli check` (end-to-end)."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

from fixtures.cli._paths import REPO_ROOT


def _unicode_scalar_to_string(manifest_scalar: str) -> str:
    prefix = "U+"
    if not manifest_scalar.upper().startswith(prefix):
        raise ValueError(manifest_scalar)
    cp = int(manifest_scalar[len(prefix) :], 16)
    return chr(cp)


def _parse_custom(case: dict) -> set[int] | None:
    opts = case.get("options") or {}
    if opts.get("invalid_mode") != "custom":
        return None
    hex_arr = opts.get("invalid_bytes_hex")
    if not hex_arr:
        return None
    s = {int(h, 16) for h in hex_arr}
    return s if s else None


def _applies_to_impl(case: dict, impl: str) -> bool:
    applies = case.get("applies_to")
    if not applies:
        return True
    legacy = {
        "python": "py",
        "csharp": "cs",
        "dotnet": "cs",
        "javascript": "nodejs",
    }
    for x in applies:
        if x == impl or legacy.get(x) == impl:
            return True
    return False


def _build_argv(case: dict) -> tuple[list[str], str]:
    """Argv for `check` after `--impl`, and stdin string."""
    custom = _parse_custom(case) or set()
    extra: list[str] = []
    for b in sorted(custom):
        extra.extend(["--custom", f"{b:02x}"])

    if "input_ascii" in case:
        s = case["input_ascii"]
        if s == "":
            return extra, ""
        return extra + [s], ""

    if "input_hex" in case:
        return extra + ["--hex", case["input_hex"]], ""

    if "input_unicode_scalar" in case:
        st = _unicode_scalar_to_string(case["input_unicode_scalar"])
        return extra + [st], ""

    raise ValueError("no input field")


def run_acceptance_main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(prog="fixtures.cli acceptance")
    p.add_argument(
        "--impl",
        default="py",
        help="Backend id (py, cpp, c, rust, cs, nodejs)",
    )
    p.add_argument(
        "--manifest",
        default=None,
        help="Path to acceptance_manifest.json",
    )
    p.add_argument(
        "--verbose",
        action="store_true",
        help="Log manifest path on stderr before running cases",
    )
    ns = p.parse_args(argv)
    impl = ns.impl
    manifest_path = (
        Path(ns.manifest) if ns.manifest else REPO_ROOT / "fixtures" / "acceptance_manifest.json"
    )
    if not manifest_path.is_file():
        sys.stderr.write(f"missing manifest: {manifest_path}\n")
        return 2

    if ns.verbose:
        sys.stderr.write(f"manifest: {manifest_path}\n")

    root = json.loads(manifest_path.read_text(encoding="utf-8"))
    failed = 0
    for case in root["cases"]:
        cid = case["id"]
        if cid == "pal-stream-note-001":
            continue
        if not _applies_to_impl(case, impl):
            continue
        sys.stderr.write(f"case {cid}\n")
        exp = case["expected"]
        kind = exp["kind"]

        try:
            check_argv, stdin_text = _build_argv(case)
        except ValueError as e:
            sys.stderr.write(f"{cid}: {e}\n")
            failed += 1
            continue

        full = [sys.executable, "-m", "fixtures.cli", "check", "--impl", impl, *check_argv]
        r = subprocess.run(
            full,
            input=stdin_text,
            text=True,
            capture_output=True,
            cwd=str(REPO_ROOT),
            timeout=120,
        )
        out = r.stdout
        err = r.stderr
        rc = r.returncode

        if kind == "boolean":
            want = exp["value"]
            expect_rc = 0 if want else 1
            expect_out = "true\n" if want else "false\n"
            if rc != expect_rc or out != expect_out:
                sys.stderr.write(
                    f"FAIL {cid}: want rc={expect_rc} stdout={expect_out!r} "
                    f"got rc={rc} stdout={out!r} stderr={err!r}\n"
                )
                failed += 1
        elif kind == "error":
            code = exp["code"]
            if rc != 2 or code not in err:
                sys.stderr.write(
                    f"FAIL {cid}: want error rc=2 code in stderr got rc={rc} stderr={err!r}\n"
                )
                failed += 1
        else:
            sys.stderr.write(f"FAIL {cid}: unknown kind {kind}\n")
            failed += 1

    if failed:
        sys.stderr.write(f"acceptance: {failed} case(s) failed\n")
        return 1
    sys.stdout.write("acceptance: ok\n")
    return 0
