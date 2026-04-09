#!/usr/bin/env bash
# First-time container setup: confirm Bazel + host PATH tools (see tools/validate_host_toolchains.sh).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "==> Bazel (via bazelisk at /usr/local/bin/bazel)"
bazel version

echo "==> Host tool PATH check (CI / dev image — not a Bazel test)"
"$ROOT/tools/validate_host_toolchains.sh"

echo "==> post-create done (run 'bazel test //...' for the full matrix)"
