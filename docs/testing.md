# Testing (polyglot)

## Two kinds of tests

1. **File-scoped tests** — One test function or method per assertion site (e.g. Rust `#[test]`, Python `unittest`, Node `node --test`, .NET xUnit). Runners list **test names** and map them to **source files** via normal tooling.

2. **Manifest-driven tests** — A single program loads [`fixtures/acceptance_manifest.json`](../fixtures/acceptance_manifest.json) and loops over `cases[]`. The outer harness (CTest, `cargo test`, etc.) may show **one** test binary; **traceability** comes from **per-case progress lines** on stderr (`case <id>`) in the C, C++, and `fixtures.cli acceptance` harnesses, plus the manifest row for the spec.

## Running the full matrix

From the repo root (after native builds exist for C/C++):

```bash
python3 tools/run_all_tests.py
```

Extra-verbose runner flags (see `argv_verbose` in [`tools/test_suites.json`](../tools/test_suites.json)):

```bash
python3 tools/run_all_tests.py --verbose
# or
VERBOSE=1 python3 tools/run_all_tests.py
```

Reports land under `reports/<timestamp>/` (HTML index + per-suite logs).
