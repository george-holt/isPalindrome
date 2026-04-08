"""Run shared acceptance cases by spawning ``is_palindrome_cli`` (supplementary subprocess path).

Canonical checks are the per-language ``acceptance_test`` targets that execute the same
manifest in-process. Use this driver only when validating the CLI multiplexer and adapters.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

from fixtures.cli._paths import REPO_ROOT, resolve_is_palindrome_cli

# Must match ``BACKENDS_ORDERED`` in ``CLI/src/lib.rs``.
BACKENDS_ORDERED = ("c", "cpp", "cs", "nodejs", "py", "rust")


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
    return impl in applies


def _build_argv(case: dict) -> tuple[list[str], str]:
    """Argv for ``is_palindrome_cli`` after ``--impl BACKEND``, and stdin string."""
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


def _impl_probe_failed(rc: int, stderr: str) -> bool:
    """True if ``--impl`` backend looks unavailable (strict CI: must not skip)."""
    if rc != 2:
        return False
    s = stderr.lower()
    return any(
        x in s
        for x in (
            "skipped",
            "not found",
            "missing",
            "no such file",
            "adapter not found",
            "python3 not found",
            "node not found",
            "dotnet not found",
        )
    )


def _run_case(
    cli_exe: Path,
    impl: str,
    cid: str,
    case: dict,
    *,
    cwd: Path,
) -> tuple[bool, str]:
    """Returns (ok, error_message)."""
    exp = case["expected"]
    kind = exp["kind"]

    try:
        check_argv, stdin_text = _build_argv(case)
    except ValueError as e:
        return False, f"{cid}: {e}"

    full = [str(cli_exe), "--impl", impl, *check_argv]
    r = subprocess.run(
        full,
        input=stdin_text,
        text=True,
        capture_output=True,
        cwd=str(cwd),
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
            return (
                False,
                f"FAIL {cid} ({impl}): want rc={expect_rc} stdout={expect_out!r} "
                f"got rc={rc} stdout={out!r} stderr={err!r}",
            )
    elif kind == "error":
        code = exp["code"]
        if rc != 2 or code not in err:
            return (
                False,
                f"FAIL {cid} ({impl}): want error rc=2 code in stderr got rc={rc} stderr={err!r}",
            )
    else:
        return False, f"FAIL {cid} ({impl}): unknown kind {kind}"

    return True, ""


def _probe_backend(cli_exe: Path, impl: str, cwd: Path) -> tuple[bool, str]:
    r = subprocess.run(
        [str(cli_exe), "--impl", impl, "aba"],
        text=True,
        capture_output=True,
        cwd=str(cwd),
        timeout=60,
    )
    if r.returncode == 0 and r.stdout.strip() == "true":
        return True, ""
    if _impl_probe_failed(r.returncode, r.stderr):
        return False, f"backend {impl!r} unavailable: rc={r.returncode} stderr={r.stderr!r}"
    return False, f"backend {impl!r} probe failed: rc={r.returncode} stdout={r.stdout!r} stderr={r.stderr!r}"


def run_acceptance_main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(prog="fixtures.cli acceptance")
    p.add_argument(
        "--impl",
        default=None,
        help="Single backend id (c, cpp, cs, nodejs, py, rust). Mutually exclusive with --all-backends.",
    )
    p.add_argument(
        "--all-backends",
        action="store_true",
        help=f"Run every case for each of {list(BACKENDS_ORDERED)} (strict: each backend must be available).",
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
    if ns.all_backends and ns.impl is not None:
        sys.stderr.write("use either --all-backends or --impl, not both\n")
        return 2
    if not ns.all_backends and ns.impl is None:
        ns.impl = "rust"
    manifest_path = (
        Path(ns.manifest) if ns.manifest else REPO_ROOT / "fixtures" / "acceptance_manifest.json"
    )
    if not manifest_path.is_file():
        sys.stderr.write(f"missing manifest: {manifest_path}\n")
        return 2

    cli_exe = resolve_is_palindrome_cli(REPO_ROOT)
    if cli_exe is None:
        sys.stderr.write(
            "is_palindrome_cli not found. Build it (e.g. "
            "`bazel build //CLI:is_palindrome_cli` or "
            "`cargo build --manifest-path CLI/Cargo.toml`) "
            "or set IS_PALINDROME_CLI to the executable path.\n"
        )
        return 2

    if ns.verbose:
        sys.stderr.write(f"manifest: {manifest_path}\n")

    root = json.loads(manifest_path.read_text(encoding="utf-8"))
    impls: tuple[str, ...] = BACKENDS_ORDERED if ns.all_backends else (ns.impl,)

    for impl in impls:
        ok, msg = _probe_backend(cli_exe, impl, REPO_ROOT)
        if not ok:
            sys.stderr.write(f"acceptance: {msg}\n")
            return 2

    failed = 0
    for impl in impls:
        for case in root["cases"]:
            cid = case["id"]
            if cid == "pal-stream-note-001":
                continue
            if not _applies_to_impl(case, impl):
                continue
            sys.stderr.write(f"case {cid} ({impl})\n")
            ok, msg = _run_case(cli_exe, impl, cid, case, cwd=REPO_ROOT)
            if not ok:
                sys.stderr.write(msg + "\n")
                failed += 1

    if failed:
        sys.stderr.write(f"acceptance: {failed} case(s) failed\n")
        return 1
    sys.stdout.write("acceptance: ok\n")
    return 0
