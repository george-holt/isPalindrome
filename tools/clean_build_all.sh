#!/usr/bin/env bash
# Remove local CMake-era dirs and Node modules under src/, Bazel clean, then tools/build_all.sh.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "==> clean: legacy CMake dirs under src/, node_modules, Bazel output"
rm -rf src/cpp/build src/cpp/out src/c/build src/c/out
rm -rf src/nodejs/ispalindrome/node_modules

bazel clean

echo "==> rebuild"
exec "$ROOT/tools/build_all.sh"
