"""Helpers to load ``acceptance_manifest.json`` and iterate cases for direct in-process tests.

The JSON manifest is the single behavioral source of truth; each language has a thin
wrapper that uses these helpers (Python) or mirrors the same rules (Rust, C, C++, Node).
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Iterator, Optional, Set


def load_manifest(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def iter_cases_for_lang(manifest: dict[str, Any], lang_id: str) -> Iterator[dict[str, Any]]:
    """Cases with ``expected``, honoring optional ``applies_to``."""
    for case in manifest.get("cases", []):
        if "expected" not in case:
            continue
        applies = case.get("applies_to")
        if applies is not None and lang_id not in applies:
            continue
        yield case


def parse_custom_bytes(case: dict[str, Any]) -> Optional[Set[int]]:
    """Custom invalid bytes, or ``None`` when default alnum validity applies."""
    opts = case.get("options") or {}
    if opts.get("invalid_mode") != "custom":
        return None
    hex_arr = opts.get("invalid_bytes_hex")
    if hex_arr is None or len(hex_arr) == 0:
        return None
    return {int(h, 16) for h in hex_arr}


def unicode_scalar_to_str(spec: str) -> str:
    p = spec.strip().upper()
    if not p.startswith("U+"):
        raise ValueError(spec)
    return chr(int(p[2:], 16))


def uses_string_api(case: dict[str, Any]) -> bool:
    if "input_unicode_scalar" in case:
        return True
    return case.get("category") == "string_api"
