# isPalindrome

Multi-language exercise and reference implementations for **ASCII / byte-sequence palindrome** detection with shared **acceptance fixtures** and a single **Python CLI** entry point (`fixtures.cli`).

## Quick start

- **Spec and rules:** [`SPEC.md`](SPEC.md)  
- **Fixtures & CLI usage:** [`fixtures/README.md`](fixtures/README.md)  
- **Toolchains, dev container, and full test matrix:** [`CONTRIBUTING.md`](CONTRIBUTING.md)

Minimal checks from the repo root (with Python on `PATH`):

```bash
python3 -m fixtures.cli acceptance --impl py
python3 tools/check_toolchain.py
python3 tools/run_all_tests.py
```

For a **reproducible environment** (all compilers and `cmake`/`ctest` preinstalled), use the **[Dev Container](CONTRIBUTING.md#fast-path-dev-container-recommended)** (`.devcontainer/`).
