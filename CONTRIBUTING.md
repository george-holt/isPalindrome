# Contributing and development setup

This repository ships **multiple language ports** (Python, Rust, C#, Node.js, C++, C) and one **shared fixture/CLI** layer. You need every toolchain listed below if you want `tools/run_all_tests.py` to go fully green locally. For how tests are structured (file-scoped vs manifest-driven) and **`--verbose` / `VERBOSE=1`**, see [docs/testing.md](docs/testing.md).

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

**Check that you are inside the container:** in the integrated terminal, `cmake --version` should succeed and `uname` should report **Linux** (not Windows).

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

The Dockerfile installs **Python 3.12** (Ubuntu), **Rust (stable)** via rustup, **Node.js 22** (NodeSource), **.NET SDK 8**, **CMake**, **Ninja**, **`build-essential`** (GCC/G++), and **`pkg-config`**. On first create, **[`.devcontainer/post-create.sh`](.devcontainer/post-create.sh)** configures and builds **`cpp/build`** and **`c/build`** so `ctest` works for the matrix.

`PYTHONPATH` is set to the workspace root so `python -m fixtures.cli …` resolves the `fixtures` package without extra steps.

## Prerequisites (local install)

| Tool | Notes |
|------|--------|
| **Python** | 3.10+ recommended; 3.12 matches CI and the devcontainer. |
| **Rust** | `rustup` + stable toolchain; `cargo` / `rustc` on `PATH`. |
| **Node.js** | ≥ 18 per [`nodejs/is-palindrome/package.json`](nodejs/is-palindrome/package.json) (`engines`); CI uses **22**. |
| **.NET SDK** | **8.0** (matches [`cs/IsPalindrome.csproj`](cs/IsPalindrome.csproj)). |
| **CMake** | **≥ 3.24** for [`cpp/CMakeLists.txt`](cpp/CMakeLists.txt); **≥ 3.16** for [`c/CMakeLists.txt`](c/CMakeLists.txt). `ctest` must be on `PATH` (it ships with CMake). |
| **C/C++ compiler** | GCC or Clang on Linux/macOS; **Visual Studio** build tools (or similar) on Windows, or use **WSL** / the **devcontainer**. |

Tests download dependencies via **CMake FetchContent** and **Cargo**/`npm` on first run (network required).

## After cloning

From the **repository root**:

1. **Check tools** (optional but quick):

   ```bash
   python3 tools/check_toolchain.py
   ```

2. **Configure native builds** (required for `cpp_acceptance` / `c_acceptance` in `run_all_tests.py`):

   ```bash
   cmake -S cpp -B cpp/build && cmake --build cpp/build
   cmake -S c -B c/build && cmake --build c/build
   ```

3. **Run the full matrix**:

   ```bash
   python3 tools/run_all_tests.py
   ```

   On success, open the printed path to `reports/<timestamp>/index.html`.

4. **CLI smoke** (see [`fixtures/README.md`](fixtures/README.md)):

   ```bash
   python3 -m fixtures.cli check --impl py aba
   python3 -m fixtures.cli acceptance --impl py
   ```

Use `python` instead of `python3` on Windows if that is how Python is installed.

## CI

GitHub Actions runs the same native matrix (configure C/C++, then `python tools/run_all_tests.py`) on **Ubuntu 24.04** — see [`.github/workflows/ci.yml`](.github/workflows/ci.yml).

## Version pins

- **Rust:** [`rust/is_palindrome/rust-toolchain.toml`](rust/is_palindrome/rust-toolchain.toml) pins the **stable** channel for that crate.
- **Node:** `engines` in `package.json` (see above).
