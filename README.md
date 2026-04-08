# isPalindrome

Multi-language exercise and reference implementations for **ASCII / byte-sequence palindrome** detection with shared **acceptance fixtures** and a **Rust** CLI (**`is_palindrome_cli`**).

## Quick start

- **Spec and rules:** [`SPEC.md`](SPEC.md)  
- **Fixtures & harness:** [`fixtures/README.md`](fixtures/README.md)  
- **Toolchains, dev container, and full test matrix:** [`CONTRIBUTING.md`](CONTRIBUTING.md)

From the **repository root** (Bazel is the supported entry):

```bash
bazel test //...   # canonical: per-language :acceptance_test (shared manifest, in-process)
bazel run //CLI:is_palindrome_cli -- aba
bazel test //fixtures:acceptance_manifest_cli   # optional manual: subprocess CLI + adapters
```

For a **reproducible environment** (compilers and Bazel preinstalled), use the **[Dev Container](CONTRIBUTING.md#fast-path-dev-container-recommended)** (`.devcontainer/`).

## Repository layout

| Location | Contents |
|----------|----------|
| **`src/`** | Language implementations: `c/`, `cpp/`, `cs/`, `nodejs/`, `py/`, `rust/` |
| **`CLI/`** | Canonical **`is_palindrome_cli`** (Rust), at repo root |
| **`fixtures/`** | Shared acceptance data (`acceptance_manifest.json`) and Python CLI harness |
| **`tools/`** | Build/coverage scripts; **`bazel build`** / **`bazel test`** are the primary workflows |
| **`reports/`** | Generated coverage HTML (gitignored); run `bazel run //tools/coverage:html` or `./tools/bazel-coverage.sh` |
| **`docs/`**, **`SPEC.md`**, **`third_party/`** | Documentation and Bazel `*_BUILD` fragments for C/C++ deps |
