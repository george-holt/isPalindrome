#!/usr/bin/env bash
# First-time container setup: confirm Bazel + host tools (see //tools:host_toolchains).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "==> Bazel (via bazelisk at /usr/local/bin/bazel)"
bazel version

echo "==> Toolchain smoke test"
bazel test //tools:host_toolchains

echo "==> post-create done (run 'bazel test //...' for the full matrix)"
