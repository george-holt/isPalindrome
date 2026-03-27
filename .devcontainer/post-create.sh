#!/usr/bin/env bash
# First-time container setup: native C/C++ builds (needed for ctest in run_all_tests).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "==> CMake: cpp (requires >= 3.24)"
cmake -S cpp -B cpp/build
cmake --build cpp/build

echo "==> CMake: c"
cmake -S c -B c/build
cmake --build c/build

echo "==> Toolchain check"
python3 tools/check_toolchain.py

echo "==> post-create done"
