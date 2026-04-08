#!/usr/bin/env python3
"""Normalize LCOV SF: lines and Cobertura filenames to a virtual tree for ReportGenerator.

Creates symlinks under --virtual (cli/, lang-c/, …) pointing at the real repo so
HTML reports can group by language/area while source links still resolve.
"""
from __future__ import annotations

import argparse
import os
import sys
import xml.etree.ElementTree as ET


# Longest repo-relative prefix first (avoid src/c matching src/cpp).
_BUCKET_PREFIXES: list[tuple[str, str]] = [
    ("src/rust/is_palindrome", "lang-rust"),
    ("src/nodejs", "lang-nodejs"),
    ("src/cpp", "lang-cpp"),
    ("src/cs", "lang-cs"),
    ("src/py", "lang-py"),
    ("src/c", "lang-c"),
    ("CLI", "cli"),
]


def _abspath_repo(repo: str, path: str) -> str:
    path = path.strip()
    if path.startswith("/proc/self/cwd/"):
        path = os.path.join(repo, path[len("/proc/self/cwd/") :])
    elif not os.path.isabs(path):
        path = os.path.join(repo, path)
    return os.path.normpath(path)


def virtualize_path(repo: str, virtual: str, abs_path: str) -> str:
    """Map a filesystem path to virtual/<bucket>/… for ReportGenerator grouping."""
    repo = os.path.normpath(os.path.abspath(repo))
    virtual = os.path.normpath(os.path.abspath(virtual))
    ap = os.path.normpath(abs_path)

    if ap.startswith(repo + os.sep) or ap == repo:
        rel = os.path.relpath(ap, repo)
        for prefix, bucket in _BUCKET_PREFIXES:
            if rel == prefix or rel.startswith(prefix + os.sep):
                sub = os.path.relpath(ap, os.path.join(repo, prefix))
                return os.path.join(virtual, bucket, sub)
        return os.path.join(virtual, "workspace", rel)

    # Outside repo (e.g. Bazel external): keep a stable tree under virtual/deps
    tail = ap
    if "/external/" in tail:
        tail = tail.split("/external/", 1)[1]
    tail = tail.replace("~", "_").replace(":", "_").lstrip(os.sep)
    if not tail or tail == ".":
        tail = "unknown"
    return os.path.join(virtual, "deps", tail)


def ensure_layout(repo: str, virtual: str) -> None:
    repo = os.path.abspath(repo)
    virtual = os.path.abspath(virtual)
    os.makedirs(virtual, exist_ok=True)
    for prefix, bucket in _BUCKET_PREFIXES:
        src = os.path.join(repo, prefix)
        dst = os.path.join(virtual, bucket)
        if os.path.lexists(dst):
            os.remove(dst)
        if os.path.isdir(src):
            os.symlink(src, dst, target_is_directory=True)
    ws = os.path.join(virtual, "workspace")
    if os.path.lexists(ws):
        os.remove(ws)
    os.symlink(repo, ws, target_is_directory=True)
    deps = os.path.join(virtual, "deps")
    os.makedirs(deps, exist_ok=True)


def normalize_lcov(repo: str, virtual: str, src_path: str, dst_path: str) -> None:
    with open(src_path, encoding="utf-8", errors="replace") as f:
        text = f.read()
    out_lines: list[str] = []
    for line in text.splitlines(keepends=True):
        if line.startswith("SF:"):
            raw = line[3:].strip()
            ap = _abspath_repo(repo, raw)
            new_p = virtualize_path(repo, virtual, ap)
            out_lines.append(f"SF:{new_p}\n")
        else:
            out_lines.append(line if line.endswith("\n") else line + "\n")
    os.makedirs(os.path.dirname(dst_path) or ".", exist_ok=True)
    with open(dst_path, "w", encoding="utf-8") as f:
        f.writelines(out_lines)


def _bucket_from_virtual(virtual_root: str, virtual_path: str) -> str | None:
    virtual_root = os.path.abspath(virtual_root)
    vp = os.path.abspath(virtual_path)
    if not vp.startswith(virtual_root + os.sep):
        return None
    rel = os.path.relpath(vp, virtual_root)
    parts = rel.split(os.sep)
    return parts[0] if parts else None


def normalize_cobertura(repo: str, virtual: str, src_path: str, dst_path: str) -> None:
    repo = os.path.abspath(repo)
    virtual = os.path.abspath(virtual)
    tree = ET.parse(src_path)
    root = tree.getroot()
    for package in root.iter("package"):
        buckets: list[str] = []
        for cls in package.iter("class"):
            fn = cls.get("filename")
            if not fn:
                continue
            ap = _abspath_repo(repo, fn)
            vp = virtualize_path(repo, virtual, ap)
            cls.set("filename", vp)
            b = _bucket_from_virtual(virtual, vp)
            if b:
                buckets.append(b)
        if buckets and len(set(buckets)) == 1:
            package.set("name", buckets[0])

    os.makedirs(os.path.dirname(dst_path) or ".", exist_ok=True)
    tree.write(dst_path, encoding="UTF-8", xml_declaration=True)


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description=__doc__)
    sub = p.add_subparsers(dest="cmd", required=True)

    lay = sub.add_parser("ensure-layout", help="Create virtual/ symlinks")
    lay.add_argument("--repo", required=True)
    lay.add_argument("--virtual", required=True)

    lc = sub.add_parser("lcov", help="Normalize one LCOV file")
    lc.add_argument("--repo", required=True)
    lc.add_argument("--virtual", required=True)
    lc.add_argument("input")
    lc.add_argument("output")

    cb = sub.add_parser("cobertura", help="Normalize one Cobertura XML file")
    cb.add_argument("--repo", required=True)
    cb.add_argument("--virtual", required=True)
    cb.add_argument("input")
    cb.add_argument("output")

    args = p.parse_args(argv)
    if args.cmd == "ensure-layout":
        ensure_layout(args.repo, args.virtual)
        return 0
    if args.cmd == "lcov":
        normalize_lcov(args.repo, args.virtual, args.input, args.output)
        return 0
    if args.cmd == "cobertura":
        normalize_cobertura(args.repo, args.virtual, args.input, args.output)
        return 0
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
