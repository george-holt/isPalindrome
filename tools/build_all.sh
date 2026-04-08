#!/usr/bin/env bash
# Build C/C++ adapters and manifest tests via Bazel (implementations live under src/).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "==> Bazel: C/C++ adapters + manifest tests"
bazel build //src/c:stdin_json_adapter //src/cpp:stdin_json_adapter

echo "==> build_all: done"
