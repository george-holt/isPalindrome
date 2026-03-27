"""Core palindrome check (SPEC §2–§3). Standard library only."""

from __future__ import annotations

from typing import Optional, Set


class PalindromeException(Exception):
    """String API: scalar > U+007F (SPEC §3)."""

    def __init__(self, error_code: str, message: str) -> None:
        self.error_code = error_code
        super().__init__(message)


def _is_ascii_alnum(b: int) -> bool:
    return (
        ord("a") <= b <= ord("z")
        or ord("A") <= b <= ord("Z")
        or ord("0") <= b <= ord("9")
    )


def _is_ascii_letter(b: int) -> bool:
    return ord("a") <= b <= ord("z") or ord("A") <= b <= ord("Z")


def _bytes_match(a: int, b: int) -> bool:
    if _is_ascii_letter(a) and _is_ascii_letter(b):
        return (a | 32) == (b | 32)
    return a == b


def _is_valid_byte(b: int, custom_delimiter_bytes: Optional[Set[int]]) -> bool:
    if not _is_ascii_alnum(b):
        return False
    if custom_delimiter_bytes and b in custom_delimiter_bytes:
        return False
    return True


def from_bytes(data: bytes, custom_delimiter_bytes: Optional[Set[int]] = None) -> bool:
    l, r = 0, len(data) - 1
    if len(data) == 0:
        return True
    while True:
        while l <= r and not _is_valid_byte(data[l], custom_delimiter_bytes):
            l += 1
        while l <= r and not _is_valid_byte(data[r], custom_delimiter_bytes):
            r -= 1
        if l >= r:
            return True
        if not _bytes_match(data[l], data[r]):
            return False
        l += 1
        r -= 1


def from_string(text: str, custom_delimiter_bytes: Optional[Set[int]] = None) -> bool:
    for c in text:
        if ord(c) > 0x7F:
            raise PalindromeException(
                "NON_ASCII_STRING_INPUT",
                "Input contains a scalar value > U+007F.",
            )
    if len(text) == 0:
        return from_bytes(b"", custom_delimiter_bytes)
    return from_bytes(text.encode("latin-1"), custom_delimiter_bytes)
