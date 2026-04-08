# Phase 1 — Acceptance fixtures

Canonical acceptance data for `isPalindrome` v1. Copy this folder into each language project (duplicate files; no shared runtime dependency).

## Implementation process (strict TDD)

All implementations in this repo must use **strict test-driven development**:

1. **Red** — Do not add or change behavior unless **at least one acceptance-level test would fail without it**. Typically: add or change a case in `acceptance_manifest.json` (and the language harness that runs it), or add a test that asserts the new behavior, and **confirm the test fails** before implementing.
2. **Green** — Write the **minimum** code that makes that test pass.
3. **Refactor** — Only with a full green suite; keep tests passing.

“Acceptance-level” means tests driven by this manifest (or equivalent shared cases), not ad-hoc tests that introduce behavior the manifest never required.

## Files

| File | Purpose |
|------|---------|
| [acceptance_manifest.json](acceptance_manifest.json) | **Source of truth**: all test cases (machine-readable JSON). |
| [acceptance_matrix.md](acceptance_matrix.md) | Requirement → test traceability. |
| [manifest_cases.py](manifest_cases.py) | Small helpers for Python `acceptance_test` (same iteration rules as other languages). |

Edit **`acceptance_manifest.json`** directly when adding or changing cases. Quick validation:

```bash
python -m json.tool acceptance_manifest.json
```

## Canonical tests and CLI

**Primary:** each language has a Bazel **`acceptance_test`** that reads **`acceptance_manifest.json`** and calls that language’s library **in-process** (`bazel test //...`).

**CLI (optional / teaching):** the Rust binary **`is_palindrome_cli`** multiplexes backends via subprocesses and thin adapters:

```bash
bazel run //CLI:is_palindrome_cli -- aba
bazel run //CLI:is_palindrome_cli -- --impl py aba
bazel test //fixtures:acceptance_manifest_cli   # manual: subprocess matrix
```

- **`is_palindrome_cli`** — argv parsing, **`--impl`**, **`--hex`**, **`--custom`**, exit codes **0** / **1** / **2**. See [`SPEC.md`](../SPEC.md) §1.
- **`//fixtures:acceptance_manifest_cli`** — six parallel **`sh_test`** shards; tagged **`manual`** so default **`bazel test //...`** runs the direct **`acceptance_test`** targets instead.

**Python helpers** under [`cli/`](cli/) (optional):

```bash
PYTHONPATH=. python3 -m fixtures.cli acceptance --impl rust
python3 -m fixtures.cli test-native    # runs: bazel test //...
```

## Harness rules

1. Parse `acceptance_manifest.json` with the language’s JSON library.
2. Build bytes: `input_ascii` → UTF-8 / Latin-1 as documented; `input_hex` → decode hex pairs (lowercase hex in manifest).
3. **`invalid_mode` / `invalid_bytes_hex`:** if `invalid_mode` is `custom` and `invalid_bytes_hex` is **non-empty**, those bytes are custom delimiters (§2). If **empty or absent**, use **default** ASCII alnum validity (same as `default` mode).
4. Skip cases where `applies_to` is present and your port is not listed (e.g. skip `pal-str-*` if you only expose a byte API).
5. **`expected.kind`:** `"boolean"` → assert result; `"error"` → assert exception / error code `expected.code`.
6. **`pal-stream-note-001`:** not automated—verify file-stream and S3-stream match in-memory for `pal-delim-001` and `pal-high-003`.

## String API cases (`pal-str-*`)

Construct strings from `input_ascii` (JSON `\u` escapes apply), or from `input_unicode_scalar` / `input_utf8_hex` as documented per case. Byte-only implementations may skip these cases. The `applies_to` field lists which backends implement the string API (`py`, `cs`, `nodejs`, `cpp`, `rust`, `c`, …).

**Toolchains and dev container:** see [`../CONTRIBUTING.md`](../CONTRIBUTING.md).
