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

Edit **`acceptance_manifest.json`** directly when adding or changing cases. Quick validation:

```bash
python -m json.tool acceptance_manifest.json
```

## Canonical CLI (`fixtures.cli`)

The **only** user-facing CLI is [`cli/`](cli/) — run from the **repository root**:

```bash
python -m fixtures.cli check --impl py aba
python -m fixtures.cli check --impl py --hex 61ff62
python -m fixtures.cli acceptance --impl py
python -m fixtures.cli test-native
```

- **`check`** — palindrome check. **`--impl`** selects the backend (`py`, `cpp`, `c`, `rust`, `cs`, `nodejs`; default `py`). Non-Python backends use thin stdin-JSON adapters in each language tree.
- **`acceptance`** — runs every applicable row in **`acceptance_manifest.json`** by invoking `check` (end-to-end). `applies_to` uses short ids aligned with top-level directories (`py`, `cs`, `nodejs`, …).
- **`test-native`** — runs [`../tools/run_all_tests.py`](../tools/run_all_tests.py) (native unit/integration matrix, timestamped HTML under `reports/<timestamp>/`). A missing toolchain **fails** the run (strict).

**Migration:** the old module was `fixtures.python_cli`; **`cli_manifest.json` has been removed** — CLI behavior is covered by **`acceptance`** + **`acceptance_manifest.json`**.

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
