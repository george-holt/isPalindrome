#!/usr/bin/env python3
"""Run native test matrix from tools/test_suites.json; write timestamped HTML report.

Each suite is one subprocess (see ``description`` in test_suites.json). Logs include
the exact command and cwd so reports are self-explanatory.

Missing toolchains or failed suites cause a non-zero exit (strict: skip == fail).
"""

from __future__ import annotations

import argparse
import html
import json
import os
import shlex
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

_REPO = Path(__file__).resolve().parent.parent
_CONFIG = Path(__file__).resolve().parent / "test_suites.json"

_PY_ROOT = _REPO / "py"
if str(_PY_ROOT) not in sys.path:
    sys.path.insert(0, str(_PY_ROOT))

from is_palindrome.cmake_cache_compat import native_build_mismatch_message


def _select_argv(suite: dict, verbose: bool) -> list[str]:
    """Use ``argv_verbose`` when ``verbose`` is True and the suite defines it."""
    if verbose and suite.get("argv_verbose"):
        return list(suite["argv_verbose"])
    return list(suite["argv"])


def _substitute_argv(argv: list[str]) -> list[str]:
    out: list[str] = []
    for a in argv:
        if a == "__PYTHON__":
            out.append(sys.executable)
        else:
            out.append(a)
    return out


def _resolve_ctest_executable() -> str | None:
    """Find ``ctest`` on PATH or next to ``cmake`` (common on Windows)."""
    exe = shutil.which("ctest")
    if exe:
        return exe
    cmake = shutil.which("cmake")
    if not cmake:
        return None
    parent = Path(cmake).resolve().parent
    for name in ("ctest", "ctest.exe", "CTest.exe"):
        p = parent / name
        if p.is_file():
            return str(p)
    return None


def _prepare_argv(argv: list[str]) -> list[str]:
    argv = _substitute_argv(argv)
    if argv and argv[0] == "ctest":
        resolved = _resolve_ctest_executable()
        if resolved:
            argv = [resolved] + argv[1:]
    return argv


def _suite_banner(
    suite_id: str,
    description: str | None,
    work: Path,
    argv: list[str],
) -> str:
    lines = [f"suite={suite_id}"]
    if description:
        lines.append(f"description={description}")
    lines.append(f"cwd={work}")
    lines.append(f"command={' '.join(shlex.quote(str(a)) for a in argv)}")
    lines.append("---")
    return "\n".join(lines) + "\n"


def _run_suite(
    suite_id: str,
    cwd: Path,
    argv: list[str],
    log_file: Path,
    *,
    description: str | None = None,
) -> tuple[int, float]:
    import time

    t0 = time.perf_counter()
    log_file.parent.mkdir(parents=True, exist_ok=True)
    work = _REPO if cwd == Path(".") else (_REPO / cwd)
    argv_prepared = _prepare_argv(argv)
    banner = _suite_banner(suite_id, description, work, argv_prepared)
    if suite_id in ("cpp_acceptance", "c_acceptance"):
        msg = native_build_mismatch_message(work)
        if msg:
            log_file.write_text(f"{banner}FAILED: {msg}\n", encoding="utf-8")
            return 1, time.perf_counter() - t0
    try:
        p = subprocess.run(
            argv_prepared,
            cwd=str(work),
            capture_output=True,
            text=True,
            timeout=600,
        )
    except FileNotFoundError as e:
        log_file.write_text(f"{banner}FAILED: executable not found: {e}\n", encoding="utf-8")
        return 127, time.perf_counter() - t0
    except subprocess.TimeoutExpired:
        log_file.write_text(f"{banner}FAILED: timeout\n", encoding="utf-8")
        return 124, time.perf_counter() - t0
    duration = time.perf_counter() - t0
    log_file.write_text(
        f"{banner}exit={p.returncode}\n--- stdout ---\n{p.stdout}\n--- stderr ---\n{p.stderr}\n",
        encoding="utf-8",
    )
    return p.returncode, duration


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Run native test matrix from test_suites.json (see docs/testing.md).",
    )
    ap.add_argument(
        "--verbose",
        action="store_true",
        help="Use per-suite argv_verbose from test_suites.json when present",
    )
    args = ap.parse_args()
    verbose = bool(args.verbose or os.environ.get("VERBOSE") == "1")

    if not _CONFIG.is_file():
        print(f"missing {_CONFIG}", file=sys.stderr)
        return 2
    data = json.loads(_CONFIG.read_text(encoding="utf-8"))
    suites = data["suites"]
    ts = datetime.now(timezone.utc).strftime("%Y%m%d-%H%M%SZ")
    out_dir = _REPO / "reports" / ts
    out_dir.mkdir(parents=True, exist_ok=True)
    logs_dir = out_dir / "logs"
    logs_dir.mkdir(parents=True, exist_ok=True)

    rows: list[dict] = []
    overall_ok = True
    for suite in sorted(suites, key=lambda s: s["id"]):
        sid = suite["id"]
        cwd = Path(suite["cwd"])
        argv = _select_argv(suite, verbose)
        desc = suite.get("description")
        log_path = logs_dir / f"{sid}.log"
        rc, dur = _run_suite(sid, cwd, argv, log_path, description=desc)
        ok = rc == 0
        if not ok:
            overall_ok = False
        rows.append(
            {
                "id": sid,
                "description": desc or "",
                "status": "PASS" if ok else "FAIL",
                "exit_code": rc,
                "duration_ms": int(dur * 1000),
                "log": str(log_path.relative_to(out_dir)),
            }
        )

    env_lines = []
    for r in rows:
        env_lines.append(
            f"suite_id={r['id']} status={r['status']} duration_ms={r['duration_ms']} exit_code={r['exit_code']}"
        )
    (out_dir / "full.log").write_text("\n".join(env_lines) + "\n", encoding="utf-8")

    results = {"timestamp": ts, "overall": "PASS" if overall_ok else "FAIL", "suites": rows}
    (out_dir / "results.json").write_text(json.dumps(results, indent=2), encoding="utf-8")

    trs = []
    for r in rows:
        trs.append(
            "<tr><td>%s</td><td>%s</td><td>%s</td><td>%s</td><td><a href=\"%s\">log</a></td></tr>"
            % (
                html.escape(r["id"]),
                html.escape(r.get("description") or ""),
                html.escape(r["status"]),
                r["duration_ms"],
                html.escape(r["log"]),
            )
        )
    html_body = f"""<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"/><title>Test matrix {html.escape(ts)}</title>
<style>body{{font-family:sans-serif;margin:1rem}} table{{border-collapse:collapse}} td,th{{border:1px solid #ccc;padding:0.35rem}} td:nth-child(2){{max-width:36rem;font-size:0.9rem}}</style>
</head><body>
<h1>Native test matrix</h1>
<p>Timestamp: {html.escape(ts)} — Overall: <strong>{html.escape(results["overall"])}</strong></p>
<table><thead><tr><th>Suite</th><th>What runs</th><th>Status</th><th>Duration ms</th><th>Log</th></tr></thead><tbody>
{"".join(trs)}
</tbody></table>
</body></html>
"""
    (out_dir / "index.html").write_text(html_body, encoding="utf-8")
    print(str(out_dir / "index.html"))
    return 0 if overall_ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
