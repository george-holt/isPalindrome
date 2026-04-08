# Contributing and development setup

This repository ships **multiple language ports** (Python, Rust, C#, Node.js, C++, C) and shared **fixtures**. The **canonical palindrome CLI** is **Rust** (`is_palindrome_cli`); use **Bazel** to build and run it. For how tests are structured (file-scoped vs manifest-driven) and **`--verbose` / `VERBOSE=1`**, see [docs/testing.md](docs/testing.md).

## Fast path: Dev Container (recommended)

This repo’s environment is defined by [`.devcontainer/devcontainer.json`](.devcontainer/devcontainer.json), [`.devcontainer/Dockerfile`](.devcontainer/Dockerfile) (all language toolchains; **no** [Dev Container Features](https://containers.dev/features/) and **no** pulls from **GHCR**), and [`.devcontainer/post-create.sh`](.devcontainer/post-create.sh). There is no separate “devcontainer profile” to pick—opening the folder **in** that container is what selects it.

**Registries:** Only the **base image** comes from **Microsoft Container Registry** (`mcr.microsoft.com/devcontainers/base`). Everything else is installed in the Dockerfile from **Ubuntu `apt`**, [NodeSource](https://github.com/nodesource/distributions) (Node.js), [Microsoft’s apt repo](https://learn.microsoft.com/dotnet/core/install/linux) (.NET SDK), and [rustup](https://rustup.rs/) (Rust). That avoids Cursor/Dev Containers bugs when resolving Features from **`ghcr.io`**.

### 1. One-time host setup

1. **Docker** must be installed and **running** (Docker Desktop on Windows/macOS, or Docker Engine on Linux).
2. **Editor:** [VS Code](https://code.visualstudio.com/) or [Cursor](https://cursor.com/).
3. **Extension:** install **Dev Containers** (VS Code marketplace id: `ms-vscode-remote.remote-containers`). In Cursor, install the same extension from the Extensions view (search “Dev Containers”).

### 2. Start the dev container from the editor (usual workflow)

1. Clone this repository and open the **repository root** folder in VS Code or Cursor (the folder that contains `.devcontainer/`).
2. Open the Command Palette:
   - **Windows / Linux:** `Ctrl+Shift+P`
   - **macOS:** `Cmd+Shift+P`
3. Run exactly this command (type part of it to filter the list):
   - **`Dev Containers: Reopen in Container`**
4. Wait for the image build (first time only) and **`post-create.sh`**. The integrated terminal should be a **Linux** shell inside the container when it finishes.

After a `git pull` that changes `.devcontainer/`, use **`Dev Containers: Rebuild Container`** (same Command Palette) so the container matches the repo.

**Check that you are inside the container:** in the integrated terminal, `bazel version` should succeed and `uname` should report **Linux** (not Windows).

### 3. Optional: command line (no editor attach)

With Docker running and [`@devcontainers/cli`](https://www.npmjs.com/package/@devcontainers/cli) available:

```bash
cd /path/to/isPalindrome    # repository root (contains .devcontainer/)
devcontainer up --workspace-folder .
```

That builds/starts the same container. To get a shell in it (typical follow-up):

```bash
devcontainer exec --workspace-folder . bash
```

Install the CLI once, for example: `npm install -g @devcontainers/cli` (requires Node on the **host**).

### What the container provides

The Dockerfile installs **Python 3.12** (Ubuntu), **Rust (stable)** via rustup, **Node.js 22** (NodeSource), **.NET SDK 8**, **`build-essential`** (GCC/G++), **`pkg-config`**, and **Bazelisk** as `bazel`. On first create, **[`.devcontainer/post-create.sh`](.devcontainer/post-create.sh)** runs a small Bazel smoke test (`//tools:host_toolchains`).

`PYTHONPATH` may be set to the workspace root when running **Python tooling** (e.g. `python -m fixtures.cli acceptance`); it is **not** required for **`bazel run //CLI:is_palindrome_cli`**.

## Prerequisites (local install)

| Tool | Notes |
|------|--------|
| **Bazel** | [Bazelisk](https://github.com/bazelbuild/bazelisk) recommended; see `.bazelversion`. |
| **Python** | 3.10+ for tooling scripts; 3.12 matches CI and the devcontainer. |
| **Rust** | `rustup` + stable toolchain; `cargo` / `rustc` on `PATH`. |
| **Node.js** | ≥ 18 per [`src/nodejs/ispalindrome/package.json`](src/nodejs/ispalindrome/package.json) (`engines`); **Bazel tests** use the pinned toolchain from [rules_nodejs](https://github.com/bazel-contrib/rules_nodejs) (see `MODULE.bazel`). |
| **.NET SDK** | **8.0** (matches [`src/cs/IsPalindrome.csproj`](src/cs/IsPalindrome.csproj)). |
| **C/C++ compiler** | GCC or Clang on Linux/macOS; **Visual Studio** build tools (or similar) on Windows, or use **WSL** / the **devcontainer**. Bazel builds C/C++ via `cc_library` / `cc_test` (see `src/c/BUILD.bazel`, `src/cpp/BUILD.bazel`). |

Tests and builds are **Bazel-first** (`bazel test //...`). Dependencies are fetched by Bazel (C/C++ archives, Node via the lockfile under `src/nodejs/ispalindrome/`, Rust crates via `Cargo.lock` for `rules_rust`) on first run (network required for those fetches).

## After cloning

From the **repository root**:

1. **Palindrome CLI (Rust):**

   ```bash
   bazel run //CLI:is_palindrome_cli -- aba
   ```

2. **Build C/C++ stdin JSON adapters** (required for **`--impl c`** / **`--impl cpp`** in `is_palindrome_cli`; the CLI looks under `bazel-bin/`):

   ```bash
   bazel build //src/c:stdin_json_adapter //src/cpp:stdin_json_adapter
   ```

3. **Full test matrix:**

   ```bash
   bazel test //...
   ```

4. **Manifest acceptance (end-to-end via `is_palindrome_cli`):**

   ```bash
   bazel test //fixtures:acceptance_manifest_cli
   ```

   That target is a **`test_suite`** of six **`sh_test`** shards (`acceptance_manifest_cli_c`, `_cpp`, `_cs`, `_nodejs`, `_py`, `_rust`) so Bazel can run backends in parallel. To run one backend: e.g. `bazel test //fixtures:acceptance_manifest_cli_cs`.

   Optional Python entry (same behavior, requires `PYTHONPATH=.` from repo root):

   ```bash
   PYTHONPATH=. python3 -m fixtures.cli acceptance --impl rust
   ```

Use `python` instead of `python3` on Windows if that is how Python is installed.

## CI

GitHub Actions runs **`bazel test //...`** on **Ubuntu 24.04** — see [`.github/workflows/ci.yml`](.github/workflows/ci.yml).

## Version pins

- **Rust:** [`src/rust/is_palindrome/rust-toolchain.toml`](src/rust/is_palindrome/rust-toolchain.toml) and [`CLI/rust-toolchain.toml`](CLI/rust-toolchain.toml) pin the **stable** channel for the library and CLI crates.
- **Node:** `engines` in `package.json` (see above).
