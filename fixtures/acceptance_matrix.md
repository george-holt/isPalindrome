# Requirement → test traceability (v1)

Cases live in [`acceptance_manifest.json`](acceptance_manifest.json).

| REQ ID | Document meaning | Test IDs |
|--------|------------------|----------|
| REQ-CORE-LOOP | Dual-cursor palindrome core | pal-basic-001..005, pal-delim-*, pal-case-001, pal-case-003, pal-high-* |
| REQ-DEFAULT-ALNUM | Default `isValidCharacter` = ASCII alnum | pal-basic-001, pal-delim-001 |
| REQ-DUAL-CURSOR | Delimiter runs misaligned left/right | pal-delim-001..004, pal-high-003 |
| REQ-VACUOUS-TRUE | Empty or only delimiters → `true` | pal-basic-003, pal-high-002, pal-vac-001, pal-vac-002, pal-str-003 |
| REQ-HIGH-BYTE-DELIM | Bytes ≥ 128 are never valid (delimiters) | pal-high-001..003 |
| REQ-ASCII-CASE-FOLD | ASCII letter pairs use case-folding | pal-case-001, pal-case-003 |
| REQ-CUSTOM-INVALID | Non-empty custom invalid byte set | pal-custom-001..003 |
| REQ-CUSTOM-EMPTY-FALLBACK | Empty or missing custom delimiter set → same as default ASCII alnum rule | pal-err-001 |
| REQ-STRING-API-ASCII | C#/JS string accepts ASCII | pal-str-001, pal-str-003 |
| REQ-STRING-API-NON-ASCII | C#/JS string rejects scalar > U+007F | pal-str-002, pal-str-004 |
| REQ-STREAM-BYTES | Streaming matches in-memory bytes | pal-stream-note-001 (manual) |

## Error codes

| Code | Cases |
|------|-------|
| `NON_ASCII_STRING_INPUT` | pal-str-002, pal-str-004, pal-str-malformed-001 (C/C++ byte-string only) |

## CLI tests

| REQ ID | Meaning | Test IDs |
|--------|---------|----------|
| REQ-CLI-CONTRACT | `is_palindrome_cli` and **`//fixtures:acceptance_manifest_cli`** (six parallel **`acceptance_manifest_cli_*`** shards) obey §1 exit codes and stdout/stderr | [`acceptance_manifest.json`](acceptance_manifest.json) via **`is_palindrome_cli`** (see [`fixtures/cli/acceptance_cmd.py`](cli/acceptance_cmd.py)) |

## Profiles

- **Core (all languages, byte API)**: all cases except those with `applies_to` set to string-only runtimes, unless you implement a string entry point.
- **string_api**: `pal-str-*` — `applies_to` lists runtimes (e.g. C#, JavaScript, Node, **cpp**, **python**, **rust**, **c**); skip if your port is not listed or you only implement `bytes`.
- **streaming_equivalence**: `pal-stream-note-001` — manual verification using file + S3 backends.
