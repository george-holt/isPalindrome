"""Parse argv for `check` (same grammar for all backends)."""

from __future__ import annotations

from typing import TextIO

ParsedCheck = tuple[bool, bool, str | None, bool, list[str], set[int]]


def parse_hex_byte(h: str) -> int:
    if len(h) != 2:
        raise ValueError("Each --custom value must be two hex digits.")
    return int(h, 16)


def decode_hex(hex_str: str) -> bytes:
    if len(hex_str) % 2 != 0:
        raise ValueError("Hex input must have an even number of characters.")
    return bytes(int(hex_str[i : i + 2], 16) for i in range(0, len(hex_str), 2))


def parse_check_argv(
    args: list[str],
) -> tuple[bool, bool, str | None, bool, list[str], set[int]]:
    """Returns (show_help, hex_mode, hex_payload, use_stdin, positional, custom_bytes)."""
    show_help = False
    hex_mode = False
    hex_payload: str | None = None
    use_stdin = False
    positional: list[str] = []
    custom: set[int] = set()
    i = 0
    while i < len(args):
        a = args[i]
        if a in ("-h", "--help"):
            show_help = True
            return show_help, False, None, False, [], set()
        if a == "--stdin":
            use_stdin = True
            i += 1
            continue
        if a == "--hex":
            if i + 1 >= len(args):
                raise ValueError("--hex requires a hex string.")
            hex_mode = True
            i += 1
            hex_payload = args[i]
            i += 1
            continue
        if a == "--custom":
            if i + 1 >= len(args):
                raise ValueError("--custom requires a two-digit hex byte.")
            i += 1
            custom.add(parse_hex_byte(args[i]))
            i += 1
            continue
        if a == "--":
            i += 1
            while i < len(args):
                positional.append(args[i])
                i += 1
            break
        positional.append(a)
        i += 1
    return show_help, hex_mode, hex_payload, use_stdin, positional, custom


def resolve_text(
    stdin: TextIO, use_stdin: bool, hex_mode: bool, positional: list[str]
) -> str:
    if use_stdin or (len(positional) == 0 and not hex_mode):
        return stdin.read()
    return " ".join(positional)
