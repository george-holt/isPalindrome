# Phase 1 — Acceptance fixtures

Canonical acceptance data for `isPalindrome` v1. Copy this folder into each language project (duplicate files; no shared runtime dependency).

## Implementation process (strict TDD)

All implementations in this repo must use **strict test-driven development**:

1. **Red** — Do not add a feature or production code unless **at least one acceptance-level test would fail without it**. Typically: add or change a case in `acceptance_manifest.json` (and the language harness that runs it), or add a test that asserts the new behavior, and **confirm the test fails** before implementing.
2. **Green** — Write the **minimum** code that makes that test pass.
3. **Refactor** — Only with a green suite; keep tests passing.

“Acceptance-level” means tests driven by this manifest (or equivalent shared cases), not ad-hoc unit tests that introduce behavior the manifest never required. Incidental refactors in production code still require justification via failing tests or regression risk covered by existing tests.

## Files

| File | Purpose |
|------|---------|
| [acceptance_manifest.json](acceptance_manifest.json) | **Source of truth**: all test cases (machine-readable JSON). |
| [acceptance_matrix.md](acceptance_matrix.md) | Requirement → test traceability. |

Edit **`acceptance_manifest.json`** directly when adding or changing cases. Quick validation:

```bash
python -m json.tool acceptance_manifest.json
```

(drops formatted JSON on stdout; exit code 0 means valid.)

## Harness rules

1. Parse `acceptance_manifest.json` with the language’s JSON library.
2. Build input bytes: `input_ascii` → encode as ASCII/Latin-1 bytes; `input_hex` → decode hex pairs (lowercase hex in manifest).
3. **`invalid_mode` / `invalid_bytes_hex`**: two-digit lowercase hex strings per byte (e.g. `"20"` = space). If `invalid_mode` is `custom` and `invalid_bytes_hex` is empty or absent, use **default** ASCII alnum validity (same as `default` mode).
4. Skip cases where `applies_to` is present and your port is not listed (e.g. skip `pal-str-*` if you only expose a byte API).
5. **`expected.kind`**: `"boolean"` → assert result; `"error"` → assert exception / `Result::Err` matching `expected.code`.
6. **`pal-stream-note-001`**: not automated—verify file-stream and S3-stream match in-memory for `pal-delim-001` and `pal-high-003`.

## String API cases (`pal-str-*`)

Construct strings from `input_ascii` (JSON `\u` escapes apply), or from `input_unicode_scalar` / `input_utf8_hex` as documented per case. Byte-only implementations may skip these cases.
